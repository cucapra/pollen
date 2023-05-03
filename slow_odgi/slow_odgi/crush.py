from mygfa import mygfa, preprocess


def crush_seg(seg):
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


def crush(graph):
    """Crush all the segments of the graph."""
    crushed_segs = {name: crush_seg(seg) for name, seg in graph.segments.items()}
    return mygfa.Graph(
        graph.headers,
        crushed_segs,
        graph.links,
        preprocess.drop_all_overlaps(graph.paths),
        # odgi drops overlaps, so we do too.
    )
