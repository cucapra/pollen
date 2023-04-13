import sys
import mygfa


def chop_segs(graph, n):
    """Chop all the sequences of the graph into length n or lower."""

    legend: Dict[str, Tuple[str, str]] = {}
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
    then seg_2_start_end[3] = (7,11).

    Later, if 3+ occurs in a path, we will replace it with 7+,8+,9+,10+.
    If 3- occurs in a path, we will replace it with 10-,9-,8-,7-.
    """

    seg_count = 1  # To generate names for the new segments.
    new_segs = {}

    for segment in graph.segments.values():
        chopped_segs = {}
        seq = segment.seq
        chopped_seqs = [seq[i:i+n] for i in range(0, len(seq), n)]
        seg_count_start = seg_count
        for cs in chopped_seqs:
            # Going from seqs to segs.
            seg_name = str(seg_count)
            chopped_segs[seg_name]=(mygfa.Segment(seg_name, cs))
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
            r = legend[seg.name]
            segments = [mygfa.Handle(str(s), o) for s in range(r[0], r[1])]
            new_p_segs += segments if o else list(reversed(segments))
        new_paths[path.name] = mygfa.Path(path.name, new_p_segs, path.overlaps)
        # For now we handle overlaps very sloppily.
    return new_paths


def chop_graph(graph):
    new_segments, legend = chop_segs(graph, n)
    new_paths = chop_paths(graph, legend)
    return mygfa.Graph(graph.headers, new_segments, [], new_paths)
    # The blank list is because we are choosing to drop links for now.


if __name__ == "__main__":
    if len(sys.argv) > 1:
        n = int(sys.argv[1])
        graph = mygfa.Graph.parse(sys.stdin)
        chopped_graph = chop_graph(graph)
        chopped_graph.emit(sys.stdout, False)
    else:
        print ("Pass the chop-size as a CLI")
