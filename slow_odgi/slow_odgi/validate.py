import mygfa
import mygfa.preprocess


def validate(graph: mygfa.Graph) -> mygfa.Graph:
    """Does the underlying set of Links support the paths that the graph has?"""
    _, outs = mygfa.preprocess.adjlist(graph)

    for path in graph.paths.values():
        length = len(path.segments)
        if length < 2:
            continue  # Success: done with this path.
        for i in range(length - 1):
            seg_from = path.segments[i]
            seg_to = path.segments[i + 1]
            if (
                seg_to not in outs[seg_from]
                and seg_from.rev() not in outs[seg_to.rev()]
            ):
                print(
                    f"[odgi::validate] error: the path {path.name} "
                    "does not respect the graph topology: the link "
                    f"{seg_from},{seg_to} is missing."
                )
    return graph
