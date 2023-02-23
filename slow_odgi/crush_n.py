import sys
import mygfa


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


if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    crushed_graph = crush_n_graph(graph)
    crushed_graph.emit(sys.stdout)
