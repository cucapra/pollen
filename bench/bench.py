try:
    import tomllib
except ImportError:
    import tomli as tomllib
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
        tmp.close()
        subprocess.run(
            ["hyperfine", "-N", "-w", "1", "--export-json", tmp.name] + cmds,
            check=True,
        )
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

    def odgi_convert(self, name):
        """Convert a GFA to odgi's `.og` format."""
        og = graph_path(name, "og")
        if os.path.exists(og):
            self.log.info("og exists for %s", name)
            return

        gfa = graph_path(name, "gfa")
        self.log.info("converting %s to og", name)
        with logtime(self.log):
            subprocess.run([self.odgi, "build", "-g", gfa, "-o", og])

    def flatgfa_convert(self, name):
        """Convert a GFA to the FlatGFA format."""
        flatgfa = graph_path(name, "flatgfa")
        if os.path.exists(flatgfa):
            self.log.info("flatgfa exists for %s", name)
            return

        gfa = graph_path(name, "gfa")
        self.log.info("converting %s to flatgfa", name)
        with logtime(self.log):
            subprocess.run([self.fgfa, "-I", gfa, "-o", flatgfa])

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

    def compare_paths(self, name, tools):
        """Compare odgi and FlatGFA implementations of path-name extraction."""
        commands = {
            "odgi": f'{self.odgi} paths -i {quote(graph_path(name, "og"))} -L',
            "flatgfa": f'{self.fgfa} -i {quote(graph_path(name, "flatgfa"))} paths',
            "slow_odgi": f'{self.slow_odgi} paths {quote(graph_path(name, "gfa"))}',
        }
        commands = {k: commands[k] for k in tools}
        yield from self.compare("paths", name, commands)

    def compare_convert(self, name, tools):
        """Compare conversion time from GFA to specialized file formats."""
        commands = {
            "odgi": f'{self.odgi} build -g {quote(graph_path(name, "gfa"))} -o {quote(graph_path(name, "og"))}',
            "flatgfa": f'{self.fgfa} -I {quote(graph_path(name, "gfa"))} -o {quote(graph_path(name, "flatgfa"))}',
        }
        commands = {k: commands[k] for k in tools}
        yield from self.compare("convert", name, commands)


def run_bench(graph_set, mode, tools, out_csv):
    runner = Runner.default()

    # The input graphs we'll be using to do the comparison.
    graph_names = runner.config["graph_sets"][graph_set]

    # Fetch all the graphs and convert them to both odgi and FlatGFA.
    for graph in graph_names:
        runner.fetch_graph(graph)
        if mode != "convert":
            runner.odgi_convert(graph)
            runner.flatgfa_convert(graph)

    runner.log.debug("writing results to %s", out_csv)
    os.makedirs(os.path.dirname(out_csv), exist_ok=True)
    with open(out_csv, "w") as f:
        writer = csv.DictWriter(f, ["graph", "cmd", "mean", "stddev", "n"])
        writer.writeheader()
        for graph in graph_names:
            match mode:
                case "paths":
                    res = runner.compare_paths(graph, tools)
                case "convert":
                    res = runner.compare_convert(graph, tools)
                case _:
                    assert False, "unknown mode"
            for row in res:
                writer.writerow(row)


def gen_csv_name(graph_set, mode):
    ts = datetime.datetime.now().strftime("%Y-%m-%d-%H-%M-%S.%f")
    return os.path.join(RESULTS_DIR, f"{mode}-{graph_set}-{ts}.csv")


def bench_main():
    parser = argparse.ArgumentParser(description="benchmarks for GFA stuff")
    parser.add_argument(
        "--graph-set", "-g", help="name of input graph set", required=True
    )
    parser.add_argument("--mode", "-m", help="thing to benchmark", required=True)
    parser.add_argument("--tool", "-t", help="test this tool", action="append")
    parser.add_argument("--output", "-o", help="output CSV")

    args = parser.parse_args()

    # Which tools are we comparing?
    tools = args.tool or {
        "paths": ALL_TOOLS,
        "convert": ["odgi", "flatgfa"],
    }[args.mode]
    for tool in tools:
        assert tool in ALL_TOOLS, "unknown tool name"

    run_bench(
        graph_set=args.graph_set,
        mode=args.mode,
        tools=tools,
        out_csv=args.output or gen_csv_name(args.graph_set, args.mode),
    )


if __name__ == "__main__":
    bench_main()
