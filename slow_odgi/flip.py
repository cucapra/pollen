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
    """Flip the given path if it is more reverse- than forward-oriented."""
    if path_is_rev(path, graph):
        path_segs = []
        for seg in reversed(path.segments):
            path_segs.append(mygfa.Handle(seg.name, not seg.orientation))
        return mygfa.Path(f"{path.name}_inv", path_segs, path.overlaps)
    else:
        return path


def gen_links(paths, prop):
    """Given a list of paths and a proposition on paths,
    return a list of links that, when added to the graph,
    would make the proposition-satisfying paths valid.

    Feels like the moral reverse of `validate`.
    """
    links = []
    for path in paths.values():
        if not prop(path):
            continue
        # Below be the paths of interest.
        length = len(path.segments)
        if length < 2:
            continue  # Success: done with this path.
        else:
            for i in range(length - 1):
                from_ = path.segments[i]
                to = path.segments[i + 1]
                links.append(mygfa.Link(from_, to, mygfa.Alignment([])))
    return links


def flip_graph(graph):
    """Apply the above to all paths."""
    new_paths = {name: flip_path(p, graph) for name, p in graph.paths.items()}
    new_links = graph.links + gen_links(new_paths, lambda x: x.name.endswith("_inv"))
    return mygfa.Graph(graph.headers, graph.segments, new_links, new_paths)


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    flipped_graph = flip_graph(graph)
    flipped_graph.emit(sys.stdout)
