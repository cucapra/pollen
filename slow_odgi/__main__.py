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
    inject_parser.add_argument(
        "-bed",
        nargs="?",
        help="A BED file describing the paths you wish to insert.",
        required=True,
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
    overlap_parser.add_argument(
        "-paths",
        nargs="?",
        help="A BED file describing the paths you wish to query.",
        required=True,
    )

    paths_parser = subparsers.add_parser("paths", help="Lists the paths in the graph.")

    validate_parser = subparsers.add_parser(
        "validate",
        help="Checks whether the links of the graph support its paths.",
    )

    # Add the graph argument to all subparsers.
    # Doing it this way means that the graph argument is sought _after_ the
    # command name.
    for subparser in subparsers.choices.values():
        subparser.add_argument(
            "graph", nargs="?", help="Input GFA file", metavar="GRAPH"
        )

    args = parser.parse_args()

    return parser, args


def parse_bedfile(filename):
    """Parse BED files that describe which paths to insert."""
    bedfile = open(filename, "r")
    return [mygfa.Bed.parse(line) for line in (mygfa.nonblanks(bedfile))]


def parse_paths(filename):
    """Parse path names from a file."""
    return list(mygfa.nonblanks(open(filename, "r")))


def dispatch(args):
    """Parse the graph from filename,
    parse any additional files if needed,
    then dispatch to the appropriate slow-odgi command.
    If the command makes a new graph, emit it to stdout."""
    name_to_func = {
        "chop": lambda x: chop.chop(x, int(args.n)),
        "crush": crush.crush,
        "degree": degree.degree,
        "depth": depth.depth,
        "flatten": lambda x: flatten.flatten(x, f"{args.graph[:-4]}.og"),
        "flip": flip.flip,
        "inject": lambda x: inject.inject(x, parse_bedfile(args.bed)),
        "matrix": matrix.matrix,
        "normalize": normalize.normalize,
        "overlap": lambda x: overlap.overlap(x, parse_paths(args.paths)),
        "paths": paths.paths,
        "validate": validate.validate,
    }
    makes_new_graph = ["chop", "crush", "flip", "inject", "normalize"]
    graph = mygfa.Graph.parse(open(args.graph, "r"))
    ans = name_to_func[args.command](graph)
    if args.command in makes_new_graph:
        ans.emit(sys.stdout)


if __name__ == "__main__":
    parser, args = parse_args()
    if not args.graph:
        parser.print_help()
        exit(-1)
    dispatch(args)
