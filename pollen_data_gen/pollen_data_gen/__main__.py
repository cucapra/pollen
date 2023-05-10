"""The pollen_data_gen command line interface."""

import argparse
from mygfa import mygfa

from . import depth, simple


def parse_args():
    """Parse command line arguments and run the appropriate subcommand."""
    parser = argparse.ArgumentParser()

    subparsers = parser.add_subparsers(
        title="pollen-data-gen commands", metavar="COMMAND", dest="command"
    )

    _ = subparsers.add_parser("simple", help="Produces a simple JSON of the graph.")

    depth_parser = subparsers.add_parser(
        "depth", help="Produces a `depth`-specific JSON of the graph."
    )
    depth_parser.add_argument(
        "-n",
        nargs="?",
        const="d",
        help="The max number of nodes.",
        required=False,
    )
    depth_parser.add_argument(
        "-e",
        nargs="?",
        const="d",
        help="The max number of steps per node.",
        required=False,
    )
    depth_parser.add_argument(
        "-p",
        nargs="?",
        const="d",
        help="The max number of paths.",
        required=False,
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


def dispatch(args):
    """Parse the graph from filename,
    then dispatch to the appropriate pollen_data_gen command.
    """
    name_to_func = {
        "depth": lambda g: depth.depth(g, args.n, args.e, args.p),
        "simple": simple.simple,
    }
    graph = mygfa.Graph.parse(open(args.graph, "r"))
    name_to_func[args.command](graph)


def main():
    """Parse command line arguments and run the appropriate subcommand."""
    parser, args = parse_args()
    if "graph" not in args or not args.graph:
        parser.print_help()
        exit(-1)
    dispatch(args)


if __name__ == "__main__":
    main()
