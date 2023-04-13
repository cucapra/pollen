import sys
import mygfa
import random

def drop_some_lines(graph):
    random.seed(4)
    links = list(sorted(graph.links))
    links[:] = random.sample(links, int(0.1 * len(links)))
    return mygfa.Graph(graph.headers, graph.segments, links, graph.paths)

if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    newgraph = drop_some_lines(graph)
    newgraph.emit(sys.stdout)
