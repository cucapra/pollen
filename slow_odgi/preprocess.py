from typing import List, Tuple, Dict
from . import mygfa


def node_steps(graph):
    """For each segment in the graph,
    list the times the segment was crossed by a path"""
    # segment name, (path name, index on path, direction) list
    crossings: Dict[str, List[Tuple[str, int, bool]]] = {}
    for segname in graph.segments.keys():
        crossings[segname] = []

    for path in graph.paths.values():
        for id, pathseg in enumerate(path.segments):
            crossings[pathseg.name].append((path.name, id, pathseg.orientation))

    return crossings


def adjlist(graph):
    """Construct an adjacency list representation of the graph.
    This is via two dicts having the same type:
    key: Handle              # my details
    value: list of Handle    # neighbors' details
    We take each segment into account, regardless of whether it is on a path.
    We make two such dicts: one for in-edges and one for out-edges
    """
    ins = {}
    outs = {}
    for segname in graph.segments.keys():
        ins[mygfa.Handle(segname, True)] = []
        ins[mygfa.Handle(segname, False)] = []
        outs[mygfa.Handle(segname, True)] = []
        outs[mygfa.Handle(segname, False)] = []

    for link in graph.links:
        ins[link.to].append(link.from_)
        outs[link.from_].append(link.to)

    return (ins, outs)


def pathseq(graph):
    """Given a graph, precompute the _sequence_
    charted by each of the graph's paths.
    """
    ans = {}
    for path in graph.paths:
        ans[path] = "".join(
            graph.segments[seg.name].seq for seg in graph.paths[path].segments
        )
    return ans
