import sys
import mygfa


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    graph.emit(sys.stdout, False)
