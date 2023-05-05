from mygfa import mygfa
from . import chop, flip


def track_path(graph, bed):
    """Given a BED entry, make a list of the Segments traversed _in full_."""
    walk = 0
    segs_walked = []
    for handle in graph.paths[bed.name].segments:
        length = len(graph.segments[handle.name].seq)
        if walk < bed.lo:
            # Skipping over segments that are not of interest.
            walk = walk + length
            continue
        if walk + length <= bed.hi:
            walk = walk + length
            segs_walked.append(handle)
        else:
            return segs_walked
    return segs_walked  # Given a legal BED, I should never reach this point.


def where_chop(graph, pathname, index):
    """Given a path and an index, find which segment should be chopped.
    We may not need to chop: the index could already be at a seam b/w segments.
    In such case, return None.
    """
    walk = 0
    for handle in graph.paths[pathname].segments:
        if walk == index:
            return None
        length = len(graph.segments[handle.name].seq)
        if walk + length > index:
            return handle.name, index - walk
        walk = walk + length


def chop_if_needed(graph, pathname, index):
    """Modify this graph such that the given index will fall on a segment-seam.
    This involves:
      1. renumbering segments
      2. redoing paths
    But at least we know we'll only ever need to renumber a max of one segment.
    """
    targetpos = where_chop(graph, pathname, index)
    if not targetpos:
        return graph  # We were already on a seam.
    target, pos = targetpos

    segments = {}
    legend = {}  # With plans to reuse `chop_paths`.

    for seg in graph.segments.values():
        segnumber = int(seg.name)
        succname = str(segnumber + 1)
        if segnumber < int(target):  # Keep these verbatim.
            segments[seg.name] = seg
            legend[seg.name] = segnumber, segnumber + 1
        elif seg.name == target:  # Perform one chop.
            segments[seg.name] = mygfa.Segment(target, seg.seq[:pos])
            segments[succname] = mygfa.Segment(succname, seg.seq[pos:])
            legend[seg.name] = segnumber, segnumber + 2
        else:  # Keep the segment as it was, but increment its name.
            segments[succname] = mygfa.Segment(succname, seg.seq)
            legend[seg.name] = segnumber + 1, segnumber + 2

    paths = chop.chop_paths(graph, legend)
    return mygfa.Graph(graph.headers, segments, graph.links, paths)


def inject(graph, p2i):
    """Given a graph and the list of paths to inject, inject those paths."""
    for p in p2i:
        if p.name in graph.paths.keys():  # odgi is silent if path was absent.
            # if flip.path_is_rev(graph.paths[p.name], graph):
            # print(f"Path {p.name} is reverse-oriented.")
            graph = chop_if_needed(chop_if_needed(graph, p.name, p.lo), p.name, p.hi)
            new_path = mygfa.Path(p.new, track_path(graph, p), None)
            graph.paths[p.new] = new_path  # In-place update!
    return graph
