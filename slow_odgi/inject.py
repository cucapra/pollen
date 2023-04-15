import sys
import mygfa


def parse_bedfile(bedfile):
  """Parse entries of the form described in `inject_setup`."""
  p2i = [mygfa.Bed.parse(line) for line in (mygfa.nonblanks(bedfile))]
  return p2i


def track_path(graph, bed):
  """Given a BED entry, make a list of the Segments traversed _completely_."""
  lo, hi = bed.lo, bed.hi
  segs_walked : List[Handle] = []
  for handle in graph.paths[bed.name].segments:
    length = len(graph.segments[handle.name].seq)
    if (lo + length <= hi):
      segs_walked.append(handle)
      lo = lo + length
    else:
      return segs_walked
  # Given a legal BED request, I should never reach this point.


def inject_paths(graph, p2i):
  """Given a graph and the list of paths to inject, inject those paths."""
  newpaths = {}
  for p in p2i:
    if p.name in graph.paths.keys(): # odgi is silent if name was invalid
      new_segs = track_path(graph, p)
      newpaths[p.new] = mygfa.Path(p.new, new_segs, None)
  paths = graph.paths | newpaths
  return mygfa.Graph(graph.headers, graph.segments, graph.links, paths)


if __name__ == "__main__":
  if len(sys.argv) > 1 and sys.argv[1].endswith(".bed"):
    paths_to_inject = parse_bedfile(open(sys.argv[1], 'r'))
    graph = mygfa.Graph.parse(sys.stdin)
    graph_inj = inject_paths(graph, paths_to_inject)
    graph_inj.emit(sys.stdout, False)
  else:
    print("Please provide a .bed file as a command line argument.")
