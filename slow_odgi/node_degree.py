import sys
import mygfa
import preprocess
from typing import List, Tuple, Dict

def node_degree(graph):
    # The degree of a node is just the cardinality of in_out for that node
    print('\t'.join(["#node.id", "node.degree"]))
    for (seg, in_out) in preprocess.in_out(graph).items():
        print('\t'.join([seg, str(len(in_out))]))


if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    node_degree(graph)
