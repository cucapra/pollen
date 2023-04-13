import sys
import mygfa

# Note that I needed the whole graph just to be able to
# access the lengths of segments.
# A possible avenue for precomputation?
def path_is_rev(path, graph):
    """Is this path more reverse-oriented than it is forward-oriented?"""
    fwd = 0
    rev = 0
    for seg in path.segments:
        length = len (graph.segments[seg.name].seq)
        if seg.orientation:
            fwd += length
        else:
            rev += length
    return rev > fwd

def flip_path(path, graph):
    if path_is_rev(path, graph):
        segments = []
        for seg in reversed(path.segments):
            segments.append (mygfa.Handle(seg.name, not seg.orientation))
        return mygfa.Path(path.name+"_inv", segments, path.overlaps)
    else:
        return path

def flip_graph(graph):
    """Apply the above, indiscriminately, to all paths"""
    new_paths = \
        {name: flip_path(path, graph) for name, path in graph.paths.items()}
    return mygfa.Graph(graph.headers, graph.segments, graph.links, new_paths)

if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    flipped_graph = flip_graph(graph)
    flipped_graph.emit(sys.stdout)
