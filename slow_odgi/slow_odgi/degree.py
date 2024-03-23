import mygfa


def degree(graph: mygfa.Graph) -> mygfa.Graph:
    """The degree of a node is just the cardinality of adjlist for that node."""
    print("\t".join(["#node.id", "node.degree"]))
    ins, outs = mygfa.preprocess.adjlist(graph)
    for seg in graph.segments.values():
        segname = seg.name
        out_degree = len(outs[mygfa.Handle(segname, True)]) + len(
            outs[mygfa.Handle(segname, False)]
        )
        in_degree = len(ins[mygfa.Handle(segname, True)]) + len(
            ins[mygfa.Handle(segname, False)]
        )
        print("\t".join([segname, str(in_degree + out_degree)]))
    return graph
