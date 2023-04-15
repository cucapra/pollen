import sys
import mygfa
import preprocess


def validate(graph):
    """Does the underlying set of Links support the paths that the graph has?"""
    _, outs = preprocess.adjlist(graph)

    for path in graph.paths.values():
        length = len(path.segments)
        if length < 2:
            continue # Success: done with this path.
        else:
            for i in range(length-1):
                seg_from = path.segments[i]
                seg_to = path.segments[i+1]
                if seg_to not in outs[seg_from] and \
                    seg_from.rev() not in outs[seg_to.rev()]:
                    print(f"[odgi::validate] error: the path {path.name} "\
                           "does not respect the graph topology: the link "\
                           f"{seg_from},{seg_to} is missing.")


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    validate(graph)
