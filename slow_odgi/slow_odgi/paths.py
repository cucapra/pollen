import sys
import random
import mygfa


def paths(graph: mygfa.Graph, droprate: int = 0) -> mygfa.Graph:
    """Print the names of the paths found in `graph`.
    The droprate represents the percentage of paths to drop.
    """
    pathnames = list(graph.paths.keys())
    if droprate > 0:
        random.seed(4)
        pathnames[:] = random.sample(
            pathnames, int((100 - droprate) / 100 * len(pathnames))
        )
    for name in pathnames:
        print(name)
    return graph


if __name__ == "__main__":
    paths(mygfa.Graph.parse(open(sys.argv[1], "r", encoding="utf-8")), int(sys.argv[2]))
