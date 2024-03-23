from typing import List, Optional
import mygfa
import mygfa.preprocess


def depth(graph: mygfa.Graph, inputpaths: Optional[List[str]]) -> mygfa.Graph:
    """The depth of a node is the cardinality of node_step for that node."""
    print("\t".join(["#node.id", "depth", "depth.uniq"]))
    for seg, crossings in mygfa.preprocess.node_steps(graph).items():
        # Each crossing is a (path name, index on path, direction) tuple.
        # We only want to count crossings that are on input paths.
        crossings = [c for c in crossings if inputpaths is None or c[0] in inputpaths]
        # For depth.uniq, we need to know how many unique path-names there are.
        uniq_path_names = set(c[0] for c in crossings)
        print("\t".join([seg, str(len(crossings)), str(len(uniq_path_names))]))
    return graph
