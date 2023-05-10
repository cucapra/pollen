from typing import List
from mygfa import mygfa


def path_is_rev(path, graph):
    """Is this path more reverse-oriented than it is forward-oriented?"""
    fwd = 0
    rev = 0
    for seg in path.segments:
        length = len(graph.segments[seg.name].seq)
        if seg.ori:
            fwd += length
        else:
            rev += length
    return rev > fwd


def flip_path(path, graph):
    """Flip the given path if it is more reverse- than forward-oriented.
    Return the path, whether this method flipped it or not,
    along with a bool that says whether this method flipped the path."""
    if path_is_rev(path, graph):
        path_segs = []
        for seg in reversed(path.segments):
            path_segs.append(mygfa.Handle(seg.name, not seg.ori))
        return mygfa.Path(f"{path.name}_inv", path_segs, None), True
    else:
        return path.drop_overlaps(), False
        # odgi drops overlaps, so we do too.


def dedup(list: List[mygfa.Link]) -> List[mygfa.Link]:
    new: List[mygfa.Link] = []
    for item in list:
        if item not in new and item.rev() not in new:
            # odgi seems to consider a link's reverse its own duplicate.
            new.append(item)
    return new


def gen_links(paths_dec, pred) -> List[mygfa.Link]:
    """Given a dict of decorated paths and a predicate on path-decorations,
    return a list of links that, when added to the graph,
    would make the predicate-satisfying paths valid.

    The code feels like the spiritual reverse of `validate`,
    and indeed, after this has been run, `validate` will be happy
    with those paths that satisfy the predicate.
    """
    links = []
    # A "no-op" alignment
    alignment = mygfa.Alignment([(0, mygfa.AlignOp("M"))])
    for path, dec in paths_dec.values():
        if not pred(dec):
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


def flip(graph):
    """Flip the paths, and generate new links that make the graph valid."""
    paths_dec = {name: flip_path(p, graph) for name, p in graph.paths.items()}
    # paths_dec is "decorated" with info re:
    # whether a path has just been flipped.
    new_links = gen_links(paths_dec, lambda x: x)
    paths = {name: p for name, (p, _) in paths_dec.items()}
    # Stripping the decoration off paths_dec gives a reasonable
    # Dict[str, Path].
    return mygfa.Graph(
        graph.headers, graph.segments, dedup(graph.links + new_links), paths
    )
