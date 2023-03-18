import sys
import mygfa
import random

def pop90percent(l):
    origlen = len(l)
    while len(l) > .1 * origlen:
        l.pop(random.randrange(len(l)))

def drop_some_lines(graph):
    links = sorted(graph.links, key=mygfa.Link.cmp)
    pop90percent(links)
    return mygfa.Graph(graph.headers, graph.segments, links, graph.paths)

if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    newgraph = drop_some_lines(graph)
    newgraph.emit(sys.stdout)
