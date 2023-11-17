import argparse
import sys
import io
from typing import Dict, Tuple, List, Optional
from collections.abc import Callable
from mygfa import mygfa

from . import (
    chop,
    crush,
    degree,
    depth,
    flatten,
    flip,
    inject,
    matrix,
    overlap,
    paths,
    proofs,
    validate,
    norm,
    inject_setup,
    validate_setup,
)


def parse_args() -> Tuple[argparse.ArgumentParser, argparse.Namespace]:
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

    subparsers.add_parser(
        "crush",
        help="Replaces consecutive instances of `N` with a single `N`.",
    )

    subparsers.add_parser(
        "degree", help="Generates a table summarizing each segment's degree."
    )

    depth_parser = subparsers.add_parser(
        "depth", help="Generates a table summarizing each segment's depth."
    )
    depth_parser.add_argument(
        "--paths",
        help="A file describing the paths you wish to query.",
        required=False,
    )

    subparsers.add_parser(
        "flatten",
        help="Converts the graph into FASTA + BED representation.",
    )

    subparsers.add_parser(
        "flip",
        help="Flips any paths that step more backward than forward.",
    )

    inject_parser = subparsers.add_parser(
        "inject", help="Adds new paths, as specified, to the graph."
    )
    inject_parser.add_argument(
        "--bed",
        nargs="?",
        help="A BED file describing the paths you wish to insert.",
        required=True,
    )

    subparsers.add_parser("matrix", help="Represents the graph as a matrix.")

    overlap_parser = subparsers.add_parser(
        "overlap",
        help="Queries the graph about which paths overlap with which other paths.",
    )
    overlap_parser.add_argument(
        "--paths",
        nargs="?",
        help="A file describing the paths you wish to query.",
        required=True,
    )

    paths_parser = subparsers.add_parser("paths", help="Lists the paths in the graph.")
    paths_parser.add_argument(
        "--drop",
        type=int,
        default=0,
        help="Randomly drop a percentage of the paths.",
        metavar="PCT",
    )

    subparsers.add_parser(
        "validate",
        help="Checks whether the links of the graph support its paths.",
    )

    norm_parser = subparsers.add_parser(
        "norm",
        help="Print a graph unmodified, normalizing its representation.",
    )
    norm_parser.add_argument(
        "--nl",
        action="store_true",
        help="Don't include links.",
    )

    # "Hidden" commands for testing only
    subparsers.add_parser("inject_setup")
    subparsers.add_parser("validate_setup")

    # Add the graph argument to all subparsers.
    # Doing it this way means that the graph argument is sought _after_ the
    # command name.
    for subparser in subparsers.choices.values():
        subparser.add_argument(
            "graph", nargs="?", help="Input GFA file", metavar="GRAPH"
        )

    args = parser.parse_args()

    return parser, args


def parse_bedfile(filename: str) -> List[mygfa.Bed]:
    """Parse BED files that describe which paths to insert."""
    bedfile = open(filename, "r", encoding="utf-8")
    return [mygfa.Bed.parse(line) for line in (mygfa.nonblanks(bedfile))]


def parse_paths(filename: Optional[str]) -> List[str]:
    """Parse path names from a file."""
    if filename:
        return list(mygfa.nonblanks(open(filename, "r", encoding="utf-8")))
    else:
        return None


def dispatch(args: argparse.Namespace) -> None:
    """Parse the graph from filename,
    parse any additional files if needed,
    then dispatch to the appropriate slow-odgi command.
    If the command makes a new graph, emit it to stdout."""

    # Functions that produce a new graph.
    transformer_funcs: Dict[str, Callable[[mygfa.Graph], mygfa.Graph]] = {
        "chop": lambda g: chop.chop(g, int(args.n)),
        "crush": crush.crush,
        "flip": flip.flip,
        "inject": lambda g: inject.inject(g, parse_bedfile(args.bed)),
        "norm": norm.norm,
        "validate_setup": validate_setup.drop_some_links,
    }

    # Other functions, which typically print their own output.
    other_funcs: Dict[str, Callable[[mygfa.Graph], object]] = {
        "degree": degree.degree,
        "depth": lambda g: depth.depth(g, parse_paths(args.paths)),
        "flatten": lambda g: flatten.flatten(g, f"{args.graph[:-4]}.og"),
        "matrix": matrix.matrix,
        "overlap": lambda g: overlap.overlap(g, parse_paths(args.paths)),
        "paths": lambda g: paths.paths(g, args.drop),
        "validate": validate.validate,
        "inject_setup": inject_setup.print_bed,
    }

    show_no_links = ["chop", "inject"]
    constructive_changes = ["chop", "inject"]
    # These commands only add to the graph, so we'll assert "logically_le".

    # Parse the input graph, which comes from either a filename argument or
    # stdin (if the filename is unspecified).
    if args.graph:
        in_file = open(args.graph, "r", encoding="utf-8")
    else:
        in_file = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8")
    graph = mygfa.Graph.parse(in_file)

    # Run the appropriate command on the input graph.
    if args.command in transformer_funcs:
        out_graph = transformer_funcs[args.command](graph)
        out_graph.emit(
            sys.stdout, args.command not in show_no_links and not vars(args).get("nl")
        )
        if args.command in constructive_changes:
            assert proofs.logically_le(graph, out_graph)
    elif args.command in other_funcs:
        other_funcs[args.command](graph)
    else:
        assert False


def main() -> None:
    """Parse command line arguments and run the appropriate subcommand."""
    parser, args = parse_args()
    dispatch(args)


if __name__ == "__main__":
    main()
