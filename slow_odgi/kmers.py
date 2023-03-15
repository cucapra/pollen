import sys
import mygfa

def flip_graph(graph):
    """Apply the above, indiscriminately, to all paths"""
    flipped_paths = \
        {name: flip_path(path, graph)
         for name, path in graph.paths.items()}
    return mygfa.Graph(graph.headers, graph.segments, graph.links, flipped_paths)


def get_path2seq(graph):
    """Populates a Dict[str, str]
    That maps a path's name to its aggregated sequence.

    Say we have:
    S 1: AAT
    S 2: GCC
    S 3: GT
    P x: 2+,1+
    P y: 1+,2+,3-

    Then
    path2seq["x"] = GCCAAT
    path2seq["y"] = AATGCCAC (because AC is the complement of GT)
    """
    path2seq = {}
    for path in graph.paths.values():
        aggregated_seg = ""
        for (segname, o) in path.segments:
            segment = graph.segments[segname]
            aggregated_seg += segment.seq if o else mygfa.Segment.rev_comp(segment)
        path2seq[path.name] = aggregated_seg
    return path2seq

def print_kmers(graph):
    print(get_path2seq(graph))

if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    print_kmers(graph)
