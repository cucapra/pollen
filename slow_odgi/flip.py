import sys
from typing import List
from . import mygfa


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


def dedup(list: List[mygfa.Link]) -> List[mygfa.Link]:
    new = []
    for item in list:
        if item not in new and item.rev() not in new:
            # odgi seems to consider a link's reverse its own duplicate.
            new.append(item)
    return new


def gen_links(paths, pred) -> List[mygfa.Link]:
    """Given a list of paths and a predicate on paths,
    return a list of links that, when added to the graph,
    would make the proposition-satisfying paths valid.

    The code feels like the spiritual reverse of `validate`,
    and indeed, after this has been run, `validate` will be happy
    with those paths that satisfy the predicate.
    """
    links = []
    alignment = mygfa.Alignment([(0, mygfa.AlignOp("M"))])  # A "no-op" alignment
    for path in paths.values():
        if not pred(path):
            continue
        # Below be the paths of interest.
        length = len(path.segments)
        if length < 2:
            continue  # Success: done with this path.
        else:
            for i in range(length - 1):
                from_ = path.segments[i]
                to = path.segments[i + 1]
                links.append(mygfa.Link(from_, to, alignment))
    return links


def flip_graph(graph):
    paths = {name: flip_path(p, graph) for name, p in graph.paths.items()}
    new_links = gen_links(paths, lambda x: x.name.endswith("_inv"))
    return mygfa.Graph(
        graph.headers, graph.segments, dedup(graph.links + new_links), paths
    )


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    flipped_graph = flip_graph(graph)
    flipped_graph.emit(sys.stdout)
