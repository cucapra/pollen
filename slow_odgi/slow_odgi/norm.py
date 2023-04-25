import sys
from . import mygfa


def norm(graph):
    """Gives the graph's entries a stable order:
    headers, then segments, then paths, and then links.
    """
    return graph


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    newgraph = norm(graph)
    newgraph.emit(sys.stdout, "--nl" not in sys.argv[1:])
