import argparse
import sys
from mygfa import mygfa

from . import mkjson


def parse_args():
    """Parse command line arguments and run the appropriate subcommand."""
    parser = argparse.ArgumentParser()

    subparsers = parser.add_subparsers(
        title="data-gen commands", metavar="COMMAND", dest="command"
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
    parse any additional files if needed,
    then dispatch to the appropriate slow-odgi command.
    If the command makes a new graph, emit it to stdout."""
    name_to_func = {
        "mkjson": lambda g: mkjson.depth_json(g, args.n, args.e, args.p),
        # "mkjson": mkjson.simple_json,
        # Toggle the two lines on/off to see mkjson emit a simple JSON
        # versus the `node depth`-specific JSON.
    }
    graph = mygfa.Graph.parse(open(args.graph, "r"))
    name_to_func[args.command](graph)


def main():
    parser, args = parse_args()
    if "graph" not in args or not args.graph:
        parser.print_help()
        exit(-1)
    dispatch(args)


if __name__ == "__main__":
    main()
