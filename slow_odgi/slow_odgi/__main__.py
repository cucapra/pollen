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
    mkjson,
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
    depth_parser.add_argument(
        "-paths",
        nargs="?",
        help="A file describing the paths you wish to query.",
        required=True,
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
        "matrix", help="Represents the graph as a matrix."
    )

    mkjson_parser = subparsers.add_parser(
        "mkjson", help="Produces a JSON representation of the graph."
    )
    mkjson_parser.add_argument(
        "-n",
        nargs="?",
        const="d",
        help="The max number of nodes.",
        required=False,
    )
    mkjson_parser.add_argument(
        "-e",
        nargs="?",
        const="d",
        help="The max number of steps per node.",
        required=False,
    )
    mkjson_parser.add_argument(
        "-p",
        nargs="?",
        const="d",
        help="The max number of paths.",
        required=False,
    )

    overlap_parser = subparsers.add_parser(
        "overlap",
        help="Queries the graph about which paths overlap with which other paths.",
    )
    overlap_parser.add_argument(
        "-paths",
        nargs="?",
        help="A file describing the paths you wish to query.",
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
        "chop": lambda g: chop.chop(g, int(args.n)),
        "crush": crush.crush,
        "degree": degree.degree,
        "depth": lambda g: depth.depth(g, parse_paths(args.paths)),
        "flatten": lambda g: flatten.flatten(g, f"{args.graph[:-4]}.og"),
        "flip": flip.flip,
        "inject": lambda g: inject.inject(g, parse_bedfile(args.bed)),
        "matrix": matrix.matrix,
        "mkjson": lambda g: mkjson.depth_json(g, args.n, args.e, args.p),
        # "mkjson": mkjson.simple_json,
        # Toggle the two lines on/off to see mkjson emit a simple JSON
        # versus the `node depth`-specific JSON.
        "overlap": lambda g: overlap.overlap(g, parse_paths(args.paths)),
        "paths": paths.paths,
        "validate": validate.validate,
    }
    makes_new_graph = ["chop", "crush", "flip", "inject"]
    no_links = ["chop", "inject"]
    graph = mygfa.Graph.parse(open(args.graph, "r"))
    ans = name_to_func[args.command](graph)
    # print(f"I sent you off to {args.command} with args {args}")
    if args.command in makes_new_graph:
        ans.emit(sys.stdout, args.command not in no_links)


def main():
    parser, args = parse_args()
    if "graph" not in args or not args.graph:
        parser.print_help()
        exit(-1)
    dispatch(args)


if __name__ == "__main__":
    main()
