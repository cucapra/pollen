import mygfa
from typing import List, Tuple, Dict


def node_steps(graph):
    """For each segment in the graph,
       list the times the segment was crossed by a path"""
    # segment name, (path name, index on path, direction) list
    crossings: Dict[str, List[Tuple[str, int, bool]]] = {}
    for segment in graph.segments.values():
        crossings[segment.name] = []

    for path in graph.paths.values():
        for id, (seg_name, seg_orient) in enumerate(path.segments):
            crossings[seg_name].append((path.name, id, seg_orient))

    return crossings

def in_out(graph):
    """
    key: segment name i.e. me
    value: list of (segment name i.e. you,
                    edge_is_from_me_to_you)

    We take each step into account, regardless of whether it is on a path.
    We could add a further, optional, item to the key with which to indicate
    whether the link is on a path.
    """
    in_out: Dict[str, List[Tuple[str, bool]]] = {}
    for segment in graph.segments.values():
        in_out[segment.name] = []

    for link in graph.links:
        in_out[link.from_].append((link.to, True))
        in_out[link.to].append((link.from_, False))

    return in_out