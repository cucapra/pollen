import sys
import mygfa


def print_paths(graph):
    """Just the names of the paths found in this graph."""
    for name in graph.paths.keys():
        print(name)


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    print_paths(graph)
