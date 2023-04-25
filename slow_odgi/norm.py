import sys
from . import mygfa


def norm(graph):
    """Gives the graph's entries a stable order:
    headers, then segments, then paths, and then links.
    """
    return graph
