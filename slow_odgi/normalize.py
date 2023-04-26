import sys
import mygfa

if __name__ == "__main__":
    """Accepts a GFA and emits it right out using mygfa's emit().
    This has the effect of normalizing the graph such that its
    entries appear in a stable order:
    headers, then segments, then paths, and then links.
    """
    graph = mygfa.Graph.parse(sys.stdin)
    graph.emit(sys.stdout, "--nl" not in sys.argv[1:])
