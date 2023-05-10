import sys
import random
from mygfa import mygfa, preprocess


def print_bed(graph):
    """Creates a reasonable query for `inject`.
    Each entry of the output is a BED where:
      `name` is the name of an existing path.
      `lo`/`hi` are the start/end points that we should walk over; lo <= hi.
      `new` is the name of the path we wish to create.
    """
    random.seed(4)
    for path in graph.paths.values():
        length = len(preprocess.pathseq(graph)[path.name])
        for i in range(random.randint(0, 5)):
            low = random.randint(0, length - 1)
            high = random.randint(low + 1, length)
            bed = mygfa.Bed(path.name, low, high, f"{path.name}_{i}")
            print(bed)


if __name__ == "__main__":
    mygraph = mygfa.Graph.parse(sys.stdin)
    print_bed(mygraph)
