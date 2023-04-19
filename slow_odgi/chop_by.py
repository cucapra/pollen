import sys
import mygfa
import random


def print_n(graph):
    """Prints out a reasonable parameter to pass to `chop`.
    Specifically: a random number between 2 and the length of the longest seq.
    """
    random.seed(4)
    longest_seq = max([len(s.seq) for s in graph.segments.values()])
    print(random.randint(2, longest_seq))


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    print_n(graph)
