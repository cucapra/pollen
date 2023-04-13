import sys
import mygfa
import preprocess
from typing import List, Tuple, Dict


def node_degree(graph):
    """The degree of a node is just the cardinality of adjlist for that node."""
    print('\t'.join(["#node.id", "node.degree"]))
    ins, outs = preprocess.adjlist(graph)
    for seg in graph.segments.values():
        seg = seg.name
        out_degree = len(outs[mygfa.Handle(seg, True)]) \
                     + len(outs[mygfa.Handle(seg, False)])
        in_degree = len(ins[mygfa.Handle(seg, True)]) \
                    + len(ins[mygfa.Handle(seg, False)])
        print('\t'.join([seg, str(in_degree + out_degree)]))


if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    node_degree(graph)
