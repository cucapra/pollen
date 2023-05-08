from mygfa import preprocess


def matrix(graph):
    """Print the graph in sparse matrix format."""

    # Just keeping up with the odgi header format...
    topseg = max([int(i) for i in graph.segments.keys()])
    print(" ".join(str(i) for i in [topseg, topseg, 2 * len(graph.links)]))

    _, outs = preprocess.adjlist(graph)
    for seg, neighbors in outs.items():
        for neighbor in neighbors:
            print(" ".join([seg.name, neighbor.name, "1"]))
            print(" ".join([neighbor.name, seg.name, "1"]))
