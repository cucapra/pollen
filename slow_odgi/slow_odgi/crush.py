import mygfa


def crush_seg(seg: mygfa.Segment) -> mygfa.Segment:
    """Compact any "runs" of N down to a single N."""
    new_seq = ""
    in_n = False
    for char in str(seg.seq):
        if char == "N":
            if in_n:
                continue
            in_n = True
        else:
            in_n = False
        new_seq += char
    return mygfa.Segment(seg.name, mygfa.Strand(new_seq))


def crush(graph: mygfa.Graph) -> mygfa.Graph:
    """Crush all the segments of the graph."""
    crushed_segs = {name: crush_seg(seg) for name, seg in graph.segments.items()}
    return mygfa.Graph(
        graph.headers,
        crushed_segs,
        graph.links,
        mygfa.preprocess.drop_all_overlaps(graph.paths),
        # odgi drops overlaps, so we do too.
    )
