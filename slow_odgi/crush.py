import sys
from . import mygfa


def crush_n_seg(seg):
    """Compact any "runs" of N down to a single N."""
    new_seq = ""
    in_n = False
    for char in seg.seq:
        if char == "N":
            if in_n:
                continue
            else:
                in_n = True
        else:
            in_n = False
        new_seq += char
    return mygfa.Segment(seg.name, new_seq)


def crush_n_graph(graph):
    """Apply the above to all nodes"""
    crushed_segs = {name: crush_n_seg(seg) for name, seg in graph.segments.items()}
    return mygfa.Graph(graph.headers, crushed_segs, graph.links, graph.paths)


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    crushed_graph = crush_n_graph(graph)
    crushed_graph.emit(sys.stdout)
