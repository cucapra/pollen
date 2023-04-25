import sys
from . import mygfa


def paths(graph):
    """Just the names of the paths found in this graph."""
    for name in graph.paths.keys():
        print(name)
