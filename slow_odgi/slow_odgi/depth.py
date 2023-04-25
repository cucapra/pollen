import sys
from . import mygfa, preprocess


def depth(graph):
    """The depth of a node is the cardinality of node_step for that node."""
    print("\t".join(["#node.id", "depth", "depth.uniq"]))
    for seg, crossings in preprocess.node_steps(graph).items():
        # For depth.uniq, we need to know how many unique path-names there are.
        uniq_path_names = set(c[0] for c in crossings)
        print("\t".join([seg, str(len(crossings)), str(len(uniq_path_names))]))
