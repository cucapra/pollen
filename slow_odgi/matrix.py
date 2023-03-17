import sys
import mygfa
import preprocess
from typing import List, Tuple, Dict

def matrix(graph):
	topseg = max([int(i) for i in graph.segments.keys()])
	print(" ".join(str(i) for i in [topseg, topseg, 2*len(graph.links)]))
	for (seg, neighbors) in preprocess.in_out(graph).items():
		dst_neighbors = filter (lambda x: x[1], neighbors)
		for dst_neighbor in dst_neighbors:
				print(" ".join([seg, dst_neighbor[0], "1"]))
				print(" ".join([dst_neighbor[0], seg, "1"]))

if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    matrix(graph)
