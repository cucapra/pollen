import sys
import mygfa
import random

def pop50percent(l):
    origlen = len(l)
    while len(l) > .5 * origlen:
        l.pop(random.randrange(len(l)))
    return l

def print_some_paths(graph):
  print("\n".join(pop50percent(list(graph.paths.keys()))))

if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    print_some_paths(graph)