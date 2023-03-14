import sys
import mygfa

# Note: I needed the whole graph just to be able to
# access the lengths of segments.
# Would have been avoided if:
# a) paths actually carried Segments (instead of names of segments)
# b) some precomputation had given me this info in a lookup table
def path_is_rev(path, graph):
    """Is this path more reverse-oriented than it is forward-oriented?"""
    fwd = 0
    rev = 0
    for (segname, orientation) in path.segments:
        length = len (graph.segments[segname].seq)
        if orientation:
            fwd += length
        else:
            rev += length
    return rev > fwd

def straighten_path(path):
    segments = []
    for (segname, orientation) in reversed(path.segments):
        segments.append ((segname, not orientation))
    return mygfa.Path(path.name+"_inv", segments, path.overlaps)

def flip_path(path, graph):
    if path_is_rev(path, graph):
        return straighten_path(path)
    else:
        return path

def flip_graph(graph):
    """Apply the above, indiscriminately, to all paths"""
    flipped_paths = \
        {name: flip_path(path, graph)
         for name, path in graph.paths.items()}
    return mygfa.Graph(graph.headers, graph.segments, graph.links, flipped_paths)


if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    flipped_graph = flip_graph(graph)
    flipped_graph.emit(sys.stdout)
