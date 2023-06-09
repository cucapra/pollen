from typing import Dict, Tuple
from mygfa import mygfa


def chop_segs(
    graph: mygfa.Graph, choplength: int
) -> Tuple[Dict[str, mygfa.Segment], mygfa.LegendType]:
    """Chop all the sequences of the graph into length n or lower."""

    legend: mygfa.LegendType = {}
    # If a segment is chopped, its sequence will be spread out over
    # up among a series of contiguous new segments.

    # While not important for segment-chopping itself, it will serve us well to
    # maintain a dict that bookkeeps this chopping.

    # For example, if
    #     S 3 = ATGGCCC
    # gets chopped into
    #     S 7 = AT
    #     S 8 = GG
    #     S 9 = CC
    #     S 10 = C
    # then legend[3] = (7,11).

    # Later, if 3+ occurs in a path, we will replace it with 7+,8+,9+,10+.
    # If 3- occurs in a path, we will replace it with 10-,9-,8-,7-.

    seg_count = 1  # To generate names for the new segments.
    new_segs: Dict[str, mygfa.Segment] = {}

    for segment in graph.segments.values():
        chopped_segs = {}
        chopped_seqs = segment.seq.chop(choplength)
        seg_count_start = seg_count
        for chopped_seg in chopped_seqs:  # Going from seqs to segs.
            seg_name = str(seg_count)
            chopped_segs[seg_name] = mygfa.Segment(seg_name, chopped_seg)
            seg_count += 1
        legend[segment.name] = (seg_count_start, seg_count)
        new_segs = new_segs | chopped_segs

    return new_segs, legend


def chop_paths(graph: mygfa.Graph, legend: mygfa.LegendType) -> Dict[str, mygfa.Path]:
    """With the legend computed as above, this step is easy."""
    new_paths = {}
    for path in graph.paths.values():
        new_p_segs = []
        for handle in path.segments:
            ori = handle.ori
            fst, snd = legend[handle.name]
            segments = [mygfa.Handle(str(s), ori) for s in range(fst, snd)]
            new_p_segs += segments if ori else list(reversed(segments))
        new_paths[path.name] = mygfa.Path(path.name, new_p_segs, None)
        # odgi drops overlaps, so we do too.
    return new_paths


def chop(graph: mygfa.Graph, choplength: int) -> mygfa.Graph:
    """Chop segments and regenerate paths."""
    new_segments, legend = chop_segs(graph, choplength)
    new_paths = chop_paths(graph, legend)
    return mygfa.Graph(graph.headers, new_segments, [], new_paths)
    # The blank list is because we are choosing to drop links for now.
