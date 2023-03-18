import sys
import mygfa
import preprocess
from typing import List, Tuple, Dict

def matrix(graph):
	topseg = max([int(i) for i in graph.segments.keys()])
	print(" ".join(str(i) for i in [topseg, topseg, 2*len(graph.links)]))
	_, outs = preprocess.in_out_edges(graph)
	for (seg, neighbors) in outs.items():
		for neighbor in neighbors:
			print(" ".join([seg[0], neighbor[0], "1"]))
			print(" ".join([neighbor[0], seg[0], "1"]))

if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    matrix(graph)
