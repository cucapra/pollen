import sys
import mygfa
import random
import preprocess


def print_bed(graph):
  # random.seed(4)
  for path in graph.paths.values():
    length = len(preprocess.pathseq(graph)[path.name])
    for i in range(random.randint(0,5)):
      r1 = random.randint(0, length)
      r2 = random.randint(0, length)
      lo = str(min(r1, r2))
      hi = str(max(r1, r2))
      print ("\t".join([path.name, lo, hi, f"{path.name}_{i}"]))


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    print_bed(graph)