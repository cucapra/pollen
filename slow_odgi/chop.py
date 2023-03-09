import sys
import mygfa
import re

limit = 3 # TODO: takes this as a CLI input

def chop_graph(graph):
    new_segments = {}
    new_links = []
    new_paths = {}

    seg_2_start_end = {}
    # Dict[str, Tuple[str, str]]
    # Maps an old segment name to the first and last of its new avatar.
    # For example,
    # S 3 = ATGGCCC was chopped into
    # S 7 = AT
    # S 8 = GG
    # S 9 = CC
    # S 10 = C
    # then seg_2_start_end[3] = (7,10)

    seg_count = 0  # will be used to generate names for the new segments

    for (name, segment) in graph.segments.items():
        chopped_segs = re.findall('...?', segment.seq)
        chopped_segs_with_names = {}
        seg_count_start = seg_count
        for cs in chopped_segs:
            seg_count += 1
            chopped_segs_with_names[(str(seg_count))]=(mygfa.Segment((str(seg_count)), cs))

        # Now let's do some link-work...
        if len (chopped_segs_with_names) > 1:
            # link up all these in a row
            pass

        seg_2_start_end[segment.name] = (str(seg_count_start), str(seg_count))
        # later, if segname linked to something else, then we'll look in this
        # table to see what the corresponding links should be in the new graph

        new_segments = new_segments | chopped_segs_with_names

    return mygfa.Graph(graph.headers, new_segments, new_links, new_paths)


if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    chopped_graph = chop_graph(graph)
    chopped_graph.emit(sys.stdout)
