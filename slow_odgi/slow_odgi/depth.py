from . import preprocess


def depth(graph, inputpaths):
    """The depth of a node is the cardinality of node_step for that node."""
    print("\t".join(["#node.id", "depth", "depth.uniq"]))
    for seg, crossings in preprocess.node_steps(graph).items():
        # Each crossing is a (path name, index on path, direction) tuple.
        # We only want to count crossings that are on input paths.
        crossings = list(filter(lambda c: c[0] in inputpaths, crossings))
        # For depth.uniq, we need to know how many unique path-names there are.
        uniq_path_names = set(c[0] for c in crossings)
        print("\t".join([seg, str(len(crossings)), str(len(uniq_path_names))]))
