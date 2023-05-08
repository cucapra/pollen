import sys
import random
from mygfa import mygfa


def drop_some_links(graph):
    """Given a graph, (pseudo)randomly drop 90% of the Links of the graph.
    This serves as a starting point from which to test `validate`.
    """
    random.seed(4)
    links = list(sorted(graph.links))
    links[:] = random.sample(links, int(0.1 * len(links)))
    return mygfa.Graph(graph.headers, graph.segments, links, graph.paths)


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    newgraph = drop_some_links(graph)
    newgraph.emit(sys.stdout)
