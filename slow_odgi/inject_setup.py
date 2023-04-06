import sys
import mygfa
import random

def print_bed(graph, outfile):
  for path in graph.paths.values():
    length = mygfa.Path.seqlen(path)
    for i in range(random.randint(0,5)):
      r1 = random.randint(0, length)
      r2 = random.randint(0, length)
      lo = min(r1, r2)
      hi = max(r1, r2)
      print ("\t".join([path.name, str(lo), str(hi), path.name+"_"+str(i)]), \
        file=outfile)

if __name__ == "__main__":
    if len(sys.argv) > 1:
      graph = mygfa.Graph.parse(open("../test/"+sys.argv[1],'r'))
      outfile = open("../test/"+sys.argv[1][:-4]+".bed", 'w')
      print_bed(graph, outfile)