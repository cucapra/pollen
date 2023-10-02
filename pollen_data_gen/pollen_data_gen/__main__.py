import sys
import argparse
from mygfa import mygfa

from . import depth, simple


def parse_args() -> tuple[argparse.ArgumentParser, argparse.Namespace]:
    """Parse command line arguments and run the appropriate subcommand."""
    parser = argparse.ArgumentParser()

    subparsers = parser.add_subparsers(
        title="pollen-data-gen commands", metavar="COMMAND", dest="command"
    )

    simple_parser = subparsers.add_parser(
        "simple", help="Produces a simple JSON serialization of the graph."
    )
    simple_parser.add_argument(
        "-n",
        default="d",
        help="The max number of nodes.",
        required=False,
    )
    simple_parser.add_argument(
        "-e",
        default="d",
        help="The max number of steps per node.",
        required=False,
    )
    simple_parser.add_argument(
        "-p",
        default="d",
        help="The max number of paths.",
        required=False,
    )

    _ = subparsers.add_parser(
        "roundtrip",
        help="Checks that we can serialize the deserilize the graph losslessly.",
    )

    depth_parser = subparsers.add_parser(
        "depth", help="Produces a `depth`-specific JSON of the graph."
    )
    depth_parser.add_argument(
        "-n",
        default="d",
        help="The max number of nodes.",
        required=False,
    )
    depth_parser.add_argument(
        "-e",
        default="d",
        help="The max number of steps per node.",
        required=False,
    )
    depth_parser.add_argument(
        "-p",
        default="d",
        help="The max number of paths.",
        required=False,
    )
    depth_parser.add_argument(
        "-s",
        "--subset-paths",
        help="A file where each line is a path of the graph to consider when calculating node depth",
        required=False
    )

    # Add the graph argument to all subparsers.
    # Doing it this way means that the graph argument is sought _after_ the
    # command name.
    for subparser in subparsers.choices.values():
        subparser.add_argument(
            "graph", help="Input GFA file", metavar="GRAPH"
        )

    args = parser.parse_args()

    return parser, args


def parse_subset_paths(filename):
    """
    Return a list of the names of paths in [filename]
    """

    if filename is None:  # Return the default value
        return []

    with open(filename, "r", encoding="utf-8") as paths_file:
        text = paths_file.read()
        return text.splitlines()


def dispatch(args: argparse.Namespace) -> None:
    """Parse the graph from filename,
    then dispatch to the appropriate pollen_data_gen command.
    """
    name_to_func = {
        "depth": lambda g: depth.depth_stdout(g, args.n, args.e, args.p, parse_subset_paths(args.subset_paths)),
        "simple": lambda g: simple.dump(g, sys.stdout, args.n, args.e, args.p),
        "roundtrip": simple.roundtrip_test,
    }
    graph = mygfa.Graph.parse(open(args.graph, "r", encoding="utf-8"))
    name_to_func[args.command](graph)


def main() -> None:
    """Parse command line arguments and run the appropriate subcommand."""
    parser, arguments = parse_args()
    if "graph" not in arguments or not arguments.graph:
        parser.print_help()
        exit(-1)
    dispatch(arguments)


if __name__ == "__main__":
    main()
