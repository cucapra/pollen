import mygfa
from typing import List, Tuple, Dict


def crush_n_seg(seg):
    """Compact any "runs" of N down to a single N."""
    seq = ""  # the crushed sequence will be built up here
    in_n = False
    for char in seg.seq:
        if char == 'N':
            if in_n:
                continue
            else:
                in_n = True
        else:
            in_n = False
        seq += char
    return mygfa.Segment(seg.name, seq)


def crush_n_graph(graph):
    """Apply the above, indiscriminately, to all nodes"""
    crushed_segments = \
        {name: crush_n_seg(seg)
         for name, seg in graph.segments.items()}
    return mygfa.Graph(crushed_segments, graph.links, graph.paths)


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
