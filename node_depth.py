import sys
import mygfa


def node_depth(graph):
    # The depth of a node is just the cardinality of node_step for that node
    for (segment, crossings) in mygfa.node_steps(graph).items():
        print('\t'.join([segment, str(len(crossings))]))


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    node_depth(graph)
