import tomllib
import os
import subprocess

BASE = os.path.dirname(__file__)
GRAPHS_TOML = os.path.join(BASE, "graphs.toml")
GRAPHS_DIR = os.path.join(BASE, "graphs")

SOME_GRAPHS = ['test.lpa', 'test.chr6c4']


def fetch_file(name, url):
    os.makedirs(GRAPHS_DIR, exist_ok=True)
    dest = os.path.join(GRAPHS_DIR, f'{name}.gfa')
    # If the file exists, don't re-download.
    if os.path.exists(dest):
        return
    subprocess.run(['curl', '-L', '-o', dest, url], check=True)


def fetch_graphs(graphs, names):
    for graph_name in names:
        suite, key = graph_name.split('.')
        url = graphs[suite][key]
        fetch_file(graph_name, url)


def bench_main():
    with open(GRAPHS_TOML, 'rb') as f:
        graphs = tomllib.load(f)
    fetch_graphs(graphs, SOME_GRAPHS)


if __name__ == "__main__":
    bench_main()
