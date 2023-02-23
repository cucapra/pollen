import sys
import mygfa
import preprocess


def node_depth(graph):
    # The depth of a node is just the cardinality of node_step for that node
    print('\t'.join(["node.id", "depth", "depth.uniq"]))
    for (seg, crossings) in preprocess.node_steps(graph).items():
        # for depth.uniq,
        # we just need to know how many unique path-names there are
        uniq_path_names = set(c[0] for c in crossings)
        print('\t'.join([seg, str(len(crossings)), str(len(uniq_path_names))]))


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    node_depth(graph)
