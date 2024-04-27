import sys
import mygfa


def paths(graph: mygfa.Graph) -> mygfa.Graph:
    """Print the names of the paths found in `graph`."""
    pathnames = graph.paths.keys()
    print("\n".join(pathnames))
    return graph


if __name__ == "__main__":
    paths(mygfa.Graph.parse(open(sys.argv[1], "r", encoding="utf-8")))
