import sys
import mygfa


def path_is_rev(path, graph):
    """Is this path more reverse-oriented than it is forward-oriented?"""
    fwd = 0
    rev = 0
    for seg in path.segments:
        length = len(graph.segments[seg.name].seq)
        if seg.orientation:
            fwd += length
        else:
            rev += length
    return rev > fwd


def flip_path(path, graph):
    """Flip the given path, if it is more reverse-oriented than forward."""
    if path_is_rev(path, graph):
        segments = []
        for seg in reversed(path.segments):
            segments.append (mygfa.Handle(seg.name, not seg.orientation))
        return mygfa.Path(f"{path.name}_inv", segments, path.overlaps)
    else:
        return path


def flip_graph(graph):
    """Apply the above to all paths."""
    new_paths = {name: flip_path(path, graph) for name, path in graph.paths.items()}
    return mygfa.Graph(graph.headers, graph.segments, graph.links, new_paths)


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    flipped_graph = flip_graph(graph)
    flipped_graph.emit(sys.stdout)
