import sys
import random
from . import mygfa


def paths(graph, droprate=0):
    """Just the names of the paths found in this graph.
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


if __name__ == "__main__":
    graph = mygfa.Graph.parse(open(sys.argv[1], "r"))
    paths(graph, int(sys.argv[2]))
