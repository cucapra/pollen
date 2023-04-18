import argparse
from sys import exit

import pollen.depth.main as depth


def main():

    # Parse commandline input
    parser = argparse.ArgumentParser()

    subparsers = parser.add_subparsers()

    depth_parser = subparsers.add_parser("depth", help="Compute node depth", conflict_handler='resolve')
    depth.config_parser(depth_parser)
    depth_parser.set_defaults(command="depth")
    
    args = parser.parse_args()

    if "command" not in args:
        parser.print_help()
        exit(-1)

    if args.command == "depth":
        depth.run(args)

    else:
        raise Exception('Command not recognized')


if __name__ == '__main__':
    main()
