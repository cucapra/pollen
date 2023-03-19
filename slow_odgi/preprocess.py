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

def in_out_edges(graph):
    """
    key: (segment name, orientation)              # my details
    value: list of (segment name, orientation)    # neighbor's details
    We take each step into account, regardless of whether it is on a path.
    We make two such dicts: one for out-edges and one for in-edges
    """
    ins: Dict[Tuple[str, bool], List[Tuple[str, bool]]] = {}
    outs: Dict[Tuple[str, bool], List[Tuple[str, bool]]] = {}
    for segment in graph.segments.values():
        ins[(segment.name, True)] = []
        ins[(segment.name, False)] = []
        outs[(segment.name, True)] = []
        outs[(segment.name, False)] = []

    for link in graph.links:
        ins[(link.to, link.to_orient)].append((link.from_, link.from_orient))
        outs[(link.from_, link.from_orient)].append((link.to, link.to_orient))

    return (ins, outs)