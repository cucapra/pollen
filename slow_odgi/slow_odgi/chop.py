from typing import Dict, Tuple
from mygfa import mygfa


def chop_segs(graph, n):
    """Chop all the sequences of the graph into length n or lower."""

    legend: Dict[str, Tuple[int, int]] = {}
    """If a segment is chopped, its sequence will be spread out over
    up among a series of contiguous new segments.

    While not important for segment-chopping itself, it will serve us well to
    maintain a dict that bookkeeps this chopping.

    For example, if
        S 3 = ATGGCCC
    gets chopped into
        S 7 = AT
        S 8 = GG
        S 9 = CC
        S 10 = C
    then legend[3] = (7,11).

    Later, if 3+ occurs in a path, we will replace it with 7+,8+,9+,10+.
    If 3- occurs in a path, we will replace it with 10-,9-,8-,7-.
    """

    seg_count = 1  # To generate names for the new segments.
    new_segs = {}

    for segment in graph.segments.values():
        chopped_segs = {}
        seq = segment.seq
        chopped_seqs = [seq[i : i + n] for i in range(0, len(seq), n)]
        seg_count_start = seg_count
        for cs in chopped_seqs:  # Going from seqs to segs.
            seg_name = str(seg_count)
            chopped_segs[seg_name] = mygfa.Segment(seg_name, cs)
            seg_count += 1
        legend[segment.name] = (seg_count_start, seg_count)
        new_segs = new_segs | chopped_segs

    return new_segs, legend


def chop_paths(graph, legend):
    """With the legend computed as above, this step is easy."""
    new_paths = {}
    for path in graph.paths.values():
        new_p_segs = []
        for seg in path.segments:
            o = seg.orientation
            a, b = legend[seg.name]
            segments = [mygfa.Handle(str(s), o) for s in range(a, b)]
            new_p_segs += segments if o else list(reversed(segments))
        new_paths[path.name] = mygfa.Path(path.name, new_p_segs, None)
        # odgi drops overlaps, so we do too.
    return new_paths


def chop(graph, n):
    """Chop segments and regenerate paths."""
    new_segments, legend = chop_segs(graph, n)
    new_paths = chop_paths(graph, legend)
    return mygfa.Graph(graph.headers, new_segments, [], new_paths)
    # The blank list is because we are choosing to drop links for now.
