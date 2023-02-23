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
