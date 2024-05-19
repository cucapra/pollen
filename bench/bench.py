try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type:ignore
import os
import subprocess
from subprocess import PIPE
from shlex import quote
import json
import tempfile
from dataclasses import dataclass
import csv
import argparse
import datetime
import logging
from contextlib import contextmanager
import time
import platform

BASE = os.path.dirname(__file__)
GRAPHS_TOML = os.path.join(BASE, "graphs.toml")
CONFIG_TOML = os.path.join(BASE, "config.toml")
GRAPHS_DIR = os.path.join(BASE, "graphs")
RESULTS_DIR = os.path.join(BASE, "results")
ALL_TOOLS = ["slow_odgi", "odgi", "flatgfa"]
DECOMPRESS = {
    ".gz": ["gunzip"],
    ".zst": ["zstd", "-d"],
}


def check_wait(popen):
    err = popen.wait()
    if err:
        raise subprocess.CalledProcessError(err, popen.args)


@contextmanager
def logtime(log):
    start = time.time()
    yield
    dur = time.time() - start
    log.info("done in %.1f seconds", dur)


@dataclass(frozen=True)
class HyperfineResult:
    command: str
    mean: float
    stddev: float
    median: float
    min: float
    max: float
    count: float

    @classmethod
    def from_json(cls, obj):
        return cls(
            command=obj["command"],
            mean=obj["mean"],
            stddev=obj["stddev"],
            median=obj["median"],
            min=obj["min"],
            max=obj["max"],
            count=len(obj["times"]),
        )


def hyperfine(cmds):
    """Run Hyperfine to compare the commands."""
    with tempfile.NamedTemporaryFile(delete=False) as tmp:
        hf_cmd = [
            "hyperfine",
            "--export-json",
            tmp.name,
            "--shell=none",
            "--warmup=1",
            "--min-runs=3",
            "--max-runs=16",
        ]
        hf_cmd += cmds

        tmp.close()
        subprocess.run(hf_cmd, check=True)
        with open(tmp.name, "rb") as f:
            data = json.load(f)
            return [HyperfineResult.from_json(r) for r in data["results"]]
        os.unlink(tmp.name)


def graph_path(name, ext):
    return os.path.join(GRAPHS_DIR, f"{name}.{ext}")


def fetch_file(dest, url):
    os.makedirs(GRAPHS_DIR, exist_ok=True)

    _, ext = os.path.splitext(url)
    if ext in DECOMPRESS:
        # Decompress the file while downloading.
        with open(dest, "wb") as f:
            curl = subprocess.Popen(["curl", "-L", url], stdout=PIPE)
            decomp = subprocess.Popen(DECOMPRESS[ext], stdin=curl.stdout, stdout=f)
            assert curl.stdout is not None
            curl.stdout.close()
            check_wait(decomp)
    else:
        # Just fetch the raw file.
        subprocess.run(["curl", "-L", "-o", dest, url], check=True)


class Runner:
    def __init__(self, graphs, config):
        self.graphs = graphs
        self.config = config

        # Some shorthands for tool paths.
        self.odgi = config["tools"]["odgi"]
        self.fgfa = config["tools"]["fgfa"]
        self.slow_odgi = config["tools"]["slow_odgi"]

        self.log = logging.getLogger("pollen-bench")
        self.log.addHandler(logging.StreamHandler())
        self.log.setLevel(logging.DEBUG)

    @classmethod
    def default(cls):
        with open(GRAPHS_TOML, "rb") as f:
            graphs = tomllib.load(f)
        with open(CONFIG_TOML, "rb") as f:
            config = tomllib.load(f)
        return cls(graphs, config)

    def _cmd_vals(self, graph):
        """Get a dictionary of values to use when formatting a command."""
        return {
            "files": {
                "gfa": quote(graph_path(graph, "gfa")),
                "og": quote(graph_path(graph, "og")),
                "flatgfa": quote(graph_path(graph, "flatgfa")),
            },
            **self.config["tools"],
        }

    def fetch_graph(self, name):
        """Fetch a single graph, given by its <suite>.<graph> name."""
        suite, key = name.split(".")
        url = self.graphs[suite][key]
        dest = graph_path(name, "gfa")

        # If the file exists, don't re-download.
        if os.path.exists(dest):
            self.log.info("gfa already fetched for %s", name)
            return

        self.log.info("fetching graph %s", name)
        fetch_file(dest, url)

    def convert(self, graph, tool, ext):
        """Convert a graph to a new format, unless the file already exists."""
        dest = graph_path(graph, ext)
        if os.path.exists(dest):
            self.log.info("%s already exists for %s", ext, graph)
            return

        self.log.info("converting %s to %s", graph, ext)
        cmd = self.config["modes"]["convert"]["cmd"][tool].format(
            **self._cmd_vals(graph)
        )
        with logtime(self.log):
            subprocess.run(cmd, shell=True)

    def prepare_files(self, graph, mode, tools):
        """Ensure that all the input files are ready for a benchmarking run.

        We first fetch the graph. Then, if the mode requires it, we convert the graph to the
        necessary formats for `tools`. Each step is skipped if the files already exist.
        """
        self.fetch_graph(graph)
        if self.config["modes"][mode].get("convert", True):
            for tool in tools:
                match tool:
                    case "odgi":
                        self.convert(graph, "odgi", "og")
                    case "flatgfa":
                        self.convert(graph, "flatgfa", "flatgfa")

    def compare(self, mode, graph, commands):
        """Run a Hyperfine comparison and produce CSV lines for the results.

        `commands` is a dict mapping tool names to command strings.
        """
        self.log.info("comparing %s for %s", mode, " ".join(commands.keys()))
        with logtime(self.log):
            results = hyperfine(list(commands.values()))
        for cmd, res in zip(commands.keys(), results):
            yield {
                "cmd": cmd,
                "mean": res.mean,
                "stddev": res.stddev,
                "graph": graph,
                "n": res.count,
            }

    def compare_mode(self, mode, graph, tools):
        """Compare a mode across several tools for a single graph."""
        mode_info = self.config["modes"][mode]
        subst = self._cmd_vals(graph)
        commands = {
            k: v.format(**subst) for k, v in mode_info["cmd"].items() if k in tools
        }
        yield from self.compare(mode, graph, commands)


def run_bench(graph_set, mode, tools, out_csv):
    runner = Runner.default()

    # The input graphs we'll be using to do the comparison.
    graph_names = runner.config["graph_sets"][graph_set]

    # Which tools are we comparing?
    tools = tools or list(runner.config["modes"][mode]["cmd"].keys())
    for tool in tools:
        assert tool in ALL_TOOLS, "unknown tool name"

    # Fetch all the graphs and convert them to both odgi and FlatGFA.
    for graph in graph_names:
        runner.prepare_files(graph, mode, tools)

    runner.log.debug("writing results to %s", out_csv)
    os.makedirs(os.path.dirname(out_csv), exist_ok=True)
    with open(out_csv, "w") as f:
        writer = csv.DictWriter(f, ["graph", "cmd", "mean", "stddev", "n"])
        writer.writeheader()
        for graph in graph_names:
            assert mode in runner.config["modes"], "unknown mode"
            res = runner.compare_mode(mode, graph, tools)
            for row in res:
                writer.writerow(row)


def gen_csv_name(graph_set, mode):
    host = platform.node().split(".")[0]
    ts = datetime.datetime.now().strftime("%Y-%m-%d-%H-%M-%S.%f")
    return os.path.join(RESULTS_DIR, f"{mode}-{graph_set}-{host}-{ts}.csv")


def bench_main():
    parser = argparse.ArgumentParser(description="benchmarks for GFA stuff")
    parser.add_argument(
        "--graph-set", "-g", help="name of input graph set", required=True
    )
    parser.add_argument("--mode", "-m", help="thing to benchmark", required=True)
    parser.add_argument("--tool", "-t", help="test this tool", action="append")
    parser.add_argument("--output", "-o", help="output CSV")

    args = parser.parse_args()

    run_bench(
        graph_set=args.graph_set,
        mode=args.mode,
        tools=args.tool,
        out_csv=args.output or gen_csv_name(args.graph_set, args.mode),
    )


if __name__ == "__main__":
    bench_main()
