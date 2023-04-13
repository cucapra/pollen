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
        for id, seg in enumerate(path.segments):
            crossings[seg.name].append((path.name, id, seg.orientation))

    return crossings

def in_out_edges(graph):
    """
    key: Handle              # my details
    value: list of Handle    # neighbor's details
    We take each step into account, regardless of whether it is on a path.
    We make two such dicts: one for in-edges and one for out-edges
    """
    ins = {}
    outs = {}
    for segment in graph.segments.values():
        ins[mygfa.Handle(segment.name, True)] = []
        ins[mygfa.Handle(segment.name, False)] = []
        outs[mygfa.Handle(segment.name, True)] = []
        outs[mygfa.Handle(segment.name, False)] = []

    for link in graph.links:
        ins[link.to].append(link.from_)
        outs[link.from_].append(link.to)

    return (ins, outs)