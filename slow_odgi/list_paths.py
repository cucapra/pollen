import sys
import mygfa

def print_paths(graph):
  for name in graph.paths.keys():
    print (name)

if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    print_paths(graph)

