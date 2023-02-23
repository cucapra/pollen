import sys
import mygfa
from typing import List, Tuple, Dict


def preprocess(graph):
    """Arguably a general cousin of node_step

    key: segment name - me
    value: list of (segment name - you, edge_is_from_me_to_you)

    We take each step into account, regardless of whether it is on a path.
    We could add a further, optional, item to the key with which to indicate
    whether the link is on a path.
    """
    in_out: Dict[str, List[Tuple[str, bool]]] = {}
    for segment in graph.segments.values():
        in_out[segment.name] = []

    for link in graph.links:
        in_out[link.from_].append((link.to, True))
        # Can add this symmetric information separately...
        in_out[link.to].append((link.from_, False))

    return in_out


def node_degree(graph):
    # The degree of a node is just the cardinality of in_out for that node
    print('\t'.join(["#node.id", "node.degree"]))
    for (seg, in_out) in preprocess(graph).items():
        print('\t'.join([seg, str(len(in_out))]))


if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    node_degree(graph)
