import sys
from .gfa import Graph


if __name__ == "__main__":
    mygraph = Graph.parse(sys.stdin)
    if len(sys.argv) > 1 and sys.argv[1] == "--nl":
        mygraph.emit(sys.stdout, False)
    else:
        mygraph.emit(sys.stdout)
