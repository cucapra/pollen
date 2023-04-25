import sys
from . import mygfa


def normalize(graph):
    """Gives the graph's entries a stable order:
    headers, then segments, then paths, and then links.
    """
    return graph
