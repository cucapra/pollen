import sys
import mygfa
import chop

def parse_bedfile(bedfile):
  """Parse entries of the form described in `inject_setup`."""
  p2i = [mygfa.Bed.parse(line) for line in (mygfa.nonblanks(bedfile))]
  return p2i


def track_path(graph, bed):
  """Given a BED entry, make a list of the Segments traversed _completely_."""
  walk = 0
  segs_walked : List[Handle] = []
  for handle in graph.paths[bed.name].segments:
    length = len(graph.segments[handle.name].seq)
    if (walk < bed.lo):
      walk = walk + length # Skipping over segments that are not of interest.
      continue
    if (walk + length <= bed.hi):
      walk = walk + length
      segs_walked.append(handle)
    else:
      return segs_walked
  # Given a legal BED request, I should never reach this point.


def where_chop(graph, pathname, index):
  """Given a path and an index, find which segment should be chopped.
  We may not need to chop: the index could already be at a seam b/w segments.
  In such case, return None.
  """
  walk = 0
  for handle in graph.paths[pathname].segments:
    if (walk == index):
      return None
    length = len(graph.segments[handle.name].seq)
    if (walk + length > index):
      return handle.name, index - walk
    walk = walk + length


def chop_if_needed(graph, pathname, index):
  """Modify this graph such that the given index will fall on a segment-seam.
  This involves:
    1. renumbering segments
    2. redoing paths
  But at least we know we'll only ever need to renumber a max of one segment.
  """
  targetpos = where_chop(graph, pathname, index)
  if not targetpos:
    return graph # We were already on a seam.
  targetname, pos = targetpos

  segments = {} # We'll accrue the new segments as we walk over the old ones.
  legend = {} # With plans to reuse `chop`.

  for seg in graph.segments.values():
    segnumber = int(seg.name)
    if (segnumber < int(targetname)): # Keep these verbatim.
      segments[seg.name] = seg
      legend[seg.name] = (segnumber, segnumber + 1)
    elif (targetname == seg.name): # Perform one chop.
      succname = str(segnumber + 1)
      segments[seg.name] = mygfa.Segment(targetname, seg.seq[:pos])
      segments[succname] = mygfa.Segment(succname, seg.seq[pos:])
      legend[seg.name] = (int(targetname), int(succname) + 1)
    else: # Keep the segment as it was, but increment its name.
      succname = str(segnumber + 1)
      segments[succname] = mygfa.Segment(succname, seg.seq)
      legend[seg.name] = (int(succname), int(succname) + 1)

  paths = chop.chop_paths(graph, legend)
  return mygfa.Graph(graph.headers, segments, graph.links, paths)


def inject_paths(graph, p2i):
  """Given a graph and the list of paths to inject, inject those paths."""
  newpaths = {}
  for p in p2i:
    if p.name in graph.paths.keys(): # odgi is silent if name was invalid
      graph = chop_if_needed(chop_if_needed(graph, p.name, p.lo), p.name, p.hi)
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
