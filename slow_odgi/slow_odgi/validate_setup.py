import sys
import random
import mygfa


def drop_some_links(graph: mygfa.Graph) -> mygfa.Graph:
    """Given a graph, (pseudo)randomly drop 90% of the Links of the graph.
    This serves as a starting point from which to test `validate`.
    """
    random.seed(4)
    links = list(sorted(graph.links))
    links[:] = random.sample(links, int(0.1 * len(links)))
    return mygfa.Graph(graph.headers, graph.segments, links, graph.paths)


if __name__ == "__main__":
    newgraph = drop_some_links(mygfa.Graph.parse(sys.stdin))
    newgraph.emit(sys.stdout)
