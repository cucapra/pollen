import argparse


def main():
    """Parse command line arguments and run the appropriate subcommand."""
    parser = argparse.ArgumentParser()
    parser.add_argument("graph", nargs="?", help="Input GFA file")

    subparsers = parser.add_subparsers(title="slow-odgi commands")

    # Command `chop`
    chop_parser = subparsers.add_parser(
        "chop",
        help="Shortens segments' sequences to a given maximum length.",
    )
    chop_parser.add_argument(
        "-n", nargs="?", const="d", help="The max segment size desired after chopping."
    )
    chop_parser.set_defaults(command="chop")

    # Command `crush`
    crush_parser = subparsers.add_parser(
        "crush",
        help="Replaces consecutive instances of `N` with a single `N`.",
    )
    crush_parser.set_defaults(command="crush")

    # Command `degree`
    degree_parser = subparsers.add_parser(
        "degree", help="Generates a table summarizing each segment's degree."
    )
    degree_parser.set_defaults(command="degree")

    # Command `depth`
    depth_parser = subparsers.add_parser(
        "depth", help="Generates a table summarizing each segment's depth."
    )
    depth_parser.set_defaults(command="depth")

    # Command `flatten`
    flatten_parser = subparsers.add_parser(
        "flatten",
        help="Converts the graph into FASTA + BED representation.",
    )
    flatten_parser.set_defaults(command="flatten")

    # Command `flip`
    flip_parser = subparsers.add_parser(
        "flip",
        help="Flips any paths that step more backward than forward.",
    )
    flip_parser.set_defaults(command="flip")

    # Command `inject`
    inject_parser = subparsers.add_parser(
        "inject", help="Adds new paths, as specified, to the graph."
    )
    inject_parser.set_defaults(command="inject")

    # Command `matrix`
    matrix_parser = subparsers.add_parser(
        "matrix", help="Represents the graph as a matrix. ."
    )
    matrix_parser.set_defaults(command="matrix")

    # Command `normalize`
    normalize_parser = subparsers.add_parser(
        "normalize", help="Runs a normalization pass over the graph."
    )
    normalize_parser.set_defaults(command="normalize")

    # Command `overlap`
    overlap_parser = subparsers.add_parser(
        "overlap",
        help="Queries the graph about which paths overlap with which other paths.",
    )
    overlap_parser.set_defaults(command="overlap")

    # Command `paths`
    paths_parser = subparsers.add_parser("paths", help="Lists the paths in the graph.")
    paths_parser.set_defaults(command="paths")

    # Command `validate`
    validate_parser = subparsers.add_parser(
        "validate",
        help="Checks whether the links of the graph support its paths.",
    )
    validate_parser.set_defaults(command="validate")

    args = parser.parse_args()

    if "command" not in args:
        parser.print_help()
        exit(-1)

    print(f"Pretend I ran {args.command} on graph {args.graph}.")


if __name__ == "__main__":
    main()
