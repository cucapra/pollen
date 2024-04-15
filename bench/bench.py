import tomllib
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

BASE = os.path.dirname(__file__)
GRAPHS_TOML = os.path.join(BASE, "graphs.toml")
CONFIG_TOML = os.path.join(BASE, "config.toml")
GRAPHS_DIR = os.path.join(BASE, "graphs")

SOME_GRAPHS = ['test.lpa', 'test.chr6c4', 'hprc.chrM']
ODGI = 'odgi'
FGFA = 'fgfa'


def check_wait(popen):
    err = popen.wait()
    if err:
        raise subprocess.CalledProcessError(err, popen.args)


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
            command=obj['command'],
            mean=obj['mean'],
            stddev=obj['stddev'],
            median=obj['median'],
            min=obj['min'],
            max=obj['max'],
            count=len(obj['times']),
        )


def hyperfine(cmds):
    """Run Hyperfine to compare the commands."""
    with tempfile.NamedTemporaryFile(delete_on_close=False) as tmp:
        tmp.close()
        subprocess.run(
            ['hyperfine', '-N', '--export-json', tmp.name] + cmds,
            check=True,
        )
        with open(tmp.name, 'rb') as f:
            data = json.load(f)
            return [HyperfineResult.from_json(r) for r in data['results']]


def graph_path(name, ext):
    return os.path.join(GRAPHS_DIR, f'{name}.{ext}')


def fetch_file(name, url):
    os.makedirs(GRAPHS_DIR, exist_ok=True)
    dest = graph_path(name, 'gfa')
    # If the file exists, don't re-download.
    if os.path.exists(dest):
        return

    if url.endswith('.gz'):
        # Decompress the file while downloading.
        with open(dest, 'wb') as f:
            curl = subprocess.Popen(['curl', '-L', url], stdout=PIPE)
            gunzip = subprocess.Popen(['gunzip'], stdin=curl.stdout, stdout=f)
            curl.stdout.close()
            check_wait(gunzip)
    else:
        # Just fetch the raw file.
        subprocess.run(['curl', '-L', '-o', dest, url], check=True)


def fetch_graphs(graphs, names):
    for graph_name in names:
        suite, key = graph_name.split('.')
        url = graphs[suite][key]
        fetch_file(graph_name, url)


def odgi_convert(name):
    """Convert a GFA to odgi's `.og` format."""
    og = graph_path(name, 'og')
    if os.path.exists(og):
        return

    gfa = graph_path(name, 'gfa')
    subprocess.run([ODGI, 'build', '-g', gfa, '-o', og])


def flatgfa_convert(name):
    """Convert a GFA to the FlatGFA format."""
    flatgfa = graph_path(name, 'flatgfa')
    if os.path.exists(flatgfa):
        return

    gfa = graph_path(name, 'gfa')
    subprocess.run([FGFA, '-I', gfa, '-o', flatgfa])


def compare_paths(name):
    """Compare odgi and FlatGFA implementations of path-name extraction.
    """
    odgi_cmd = f'odgi paths -i {quote(graph_path(name, "og"))} -L'
    fgfa_cmd = f'fgfa -i {quote(graph_path(name, "flatgfa"))} paths'
    slow_cmd = f'slow_odgi paths {quote(graph_path(name, "gfa"))}'

    results = hyperfine([slow_cmd, odgi_cmd, fgfa_cmd])
    names = ['slow_odgi paths', 'odgi paths', 'fgfa paths']
    for cmd, res in zip(names, results):
        yield {
            'cmd': cmd,
            'mean': res.mean,
            'stddev': res.stddev,
            'graph': name,
            'n': res.count,
        }


def run_bench(graph_set, mode, out_csv):
    with open(GRAPHS_TOML, 'rb') as f:
        graphs = tomllib.load(f)
    with open(CONFIG_TOML, 'rb') as f:
        config = tomllib.load(f)

    assert mode == 'paths'
    graph_names = config['graph_sets'][graph_set]

    # Fetch all the graphs and convert them to both odgi and FlatGFA.
    fetch_graphs(graphs, graph_names)
    for graph in graph_names:
        odgi_convert(graph)
        flatgfa_convert(graph)

    with open(out_csv, 'w') as f:
        writer = csv.DictWriter(f, ['graph', 'cmd', 'mean', 'stddev', 'n'])
        writer.writeheader()
        for graph in graph_names:
            for row in compare_paths(graph):
                writer.writerow(row)


def gen_csv_name(graph_set, mode):
    ts = datetime.datetime.now().strftime('%Y-%m-%d-%H-%M-%S.%f')
    return f'{mode}-{graph_set}-{ts}.csv'


def bench_main():
    parser = argparse.ArgumentParser(description='benchmarks for GFA stuff')
    parser.add_argument('--graph-set', '-g', help='name of input graph set',
                        required=True)
    parser.add_argument('--mode', '-m', help='thing to benchmark',
                        required=True)
    parser.add_argument('--output', '-o', help='output CSV')
    args = parser.parse_args()
    run_bench(
        graph_set=args.graph_set,
        mode=args.mode,
        out_csv=args.output or gen_csv_name(args.graph_set, args.mode),
    )


if __name__ == "__main__":
    bench_main()
