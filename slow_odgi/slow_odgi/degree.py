from mygfa import mygfa, preprocess


def degree(graph):
    """The degree of a node is just the cardinality of adjlist for that node."""
    print("\t".join(["#node.id", "node.degree"]))
    ins, outs = preprocess.adjlist(graph)
    for seg in graph.segments.values():
        seg = seg.name
        out_degree = len(outs[mygfa.Handle(seg, True)]) + len(
            outs[mygfa.Handle(seg, False)]
        )
        in_degree = len(ins[mygfa.Handle(seg, True)]) + len(
            ins[mygfa.Handle(seg, False)]
        )
        print("\t".join([seg, str(in_degree + out_degree)]))
    return graph
