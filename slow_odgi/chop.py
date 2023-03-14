import sys
import mygfa

n = 3 # TODO: take this as a CLI input

def chop_graph(graph):
    new_segments = {}
    new_links = []
    new_paths = {}

    seg_2_start_end = {}
    """Dict[str, Tuple[str, str]]
    Maps an old segment name to the first and last of its new avatar.

    For example, if
    ...
    S 3 = ATGGCCC
    ...
    was chopped into
    ...
    S 7 = AT
    S 8 = GG
    S 9 = CC
    S 10 = C
    ...
    then seg_2_start_end[3] = (7,11).
    Later, if 3+ occurs in a path, we will replace it with 7+,8+,9+,10+.
    If 3- occurs in a path, we will replace it with 10-,9-,8-,7-.
    """

    seg_count = 1  # will be used to generate names for the new segments

    for (name, segment) in graph.segments.items():
        seq = segment.seq
        chopped_seqs = [seq[i:i+n] for i in range(0, len(seq), n)]
        chopped_segments = {}
        seg_count_start = seg_count
        for cs in chopped_seqs:
            seg_name = str(seg_count)
            chopped_segments[seg_name]=(mygfa.Segment(seg_name, cs))
            seg_count += 1
        seg_2_start_end[segment.name] = (seg_count_start, seg_count)

        new_segments = new_segments | chopped_segments

    for path in graph.paths.values():
        new_path_segments = []
        for (segname, o) in path.segments:
            r = seg_2_start_end[segname]
            if o: # forward direction
                new_path_segments += [(s,o) for s in range(r[0], r[1])]
            else: # reverse direction
                new_path_segments += [(s,o) for s in range(r[1], r[0])]
        new_paths[path.name] = mygfa.Path(path.name, new_path_segments, path.overlaps)

    return mygfa.Graph(graph.headers, new_segments, new_links, new_paths)


if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    chopped_graph = chop_graph(graph)
    chopped_graph.emit(sys.stdout)
