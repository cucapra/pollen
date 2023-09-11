import sys
from mygfa import mygfa


def norm(graph: mygfa.Graph) -> mygfa.Graph:
    """Gives the graph's entries a stable order:
    headers, then segments, then paths, and then links.
    """
    return graph


if __name__ == "__main__":
    newgraph = norm(mygfa.Graph.parse(sys.stdin))
    newgraph.emit(sys.stdout, "--nl" not in sys.argv[1:])
