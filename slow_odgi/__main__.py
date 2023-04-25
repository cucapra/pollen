import argparse
import sys
from . import (
    chop,
    crush,
    degree,
    depth,
    flatten,
    flip,
    inject,
    matrix,
    normalize,
    mygfa,
    overlap,
    paths,
    validate,
)


def parse_args():
    """Parse command line arguments and run the appropriate subcommand."""
    parser = argparse.ArgumentParser()

    parser.add_argument("graph", nargs="?", help="Input GFA file", metavar="GRAPH")

    subparsers = parser.add_subparsers(
        title="slow-odgi commands", metavar="COMMAND", dest="command"
    )

    chop_parser = subparsers.add_parser(
        "chop",
        help="Shortens segments' sequences to a given maximum length.",
    )
    chop_parser.add_argument(
        "-n",
        nargs="?",
        const="d",
        help="The max segment size desired after chopping.",
        required=True,
    )

    crush_parser = subparsers.add_parser(
        "crush",
        help="Replaces consecutive instances of `N` with a single `N`.",
    )

    degree_parser = subparsers.add_parser(
        "degree", help="Generates a table summarizing each segment's degree."
    )

    depth_parser = subparsers.add_parser(
        "depth", help="Generates a table summarizing each segment's depth."
    )

    flatten_parser = subparsers.add_parser(
        "flatten",
        help="Converts the graph into FASTA + BED representation.",
    )

    flip_parser = subparsers.add_parser(
        "flip",
        help="Flips any paths that step more backward than forward.",
    )

    inject_parser = subparsers.add_parser(
        "inject", help="Adds new paths, as specified, to the graph."
    )

    matrix_parser = subparsers.add_parser(
        "matrix", help="Represents the graph as a matrix. ."
    )

    normalize_parser = subparsers.add_parser(
        "normalize", help="Runs a normalization pass over the graph."
    )

    overlap_parser = subparsers.add_parser(
        "overlap",
        help="Queries the graph about which paths overlap with which other paths.",
    )

    paths_parser = subparsers.add_parser("paths", help="Lists the paths in the graph.")

    validate_parser = subparsers.add_parser(
        "validate",
        help="Checks whether the links of the graph support its paths.",
    )

    args = parser.parse_args()

    return args


def dispatch(args):
    """Parse the graph from filename, then dispatch to the appropriate subcommand."""
    name_to_func = {
        "chop": lambda x: chop.chop_graph(x, args.n),
        "crush": crush.crush_n_graph,
        "degree": degree.node_degree,
        "depth": depth.node_depth,
        # "flatten": flatten
        "flip": flip.flip_graph,
        # "inject": inject
        "matrix": matrix.matrix,
        # "normalize": normalize,
        # "overlap": overlap,
        "paths": paths.print_paths,
        "validate": validate.validate,
    }
    graph = mygfa.Graph.parse(open(args.graph, "r"))
    ans = name_to_func[args.command](graph)
    ans.emit(sys.stdout)


if __name__ == "__main__":
    args = parse_args()
    dispatch(args)
