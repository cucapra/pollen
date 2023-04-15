import sys
import mygfa
from typing import NamedTuple

# TODO: lift so that inject_setup also uses this.
class P2I(NamedTuple):
	src: str
	lo: int
	hi: int
	new: str


def parse_bed(bedfile):
	"""Parse entries of the form described in `inject_setup`."""
	p2i: List[P2I] = []
	for line in (mygfa.nonblanks(bedfile)):
		src, lo, hi, new = line.split("\t")
		p2i.append(P2I(src, int(lo), int(hi), new))
	return p2i


def inject_paths(graph, p2i):
	"""Given a graph and the list of paths to inject, inject those paths."""
	newpaths = {}
	for p in p2i:
		if p.src in graph.paths.keys(): # odgi fails silently if src was invalid
			newpaths[p.new] = mygfa.Path(p.new, [], None)
	paths = graph.paths | newpaths
	return mygfa.Graph(graph.headers, graph.segments, graph.links, paths)


if __name__ == "__main__":
	if len(sys.argv) > 1 and sys.argv[1].endswith(".bed"):
		paths_to_inject = parse_bed(open(sys.argv[1], 'r'))
		graph = mygfa.Graph.parse(sys.stdin)
		graph_inj = inject_paths(graph, paths_to_inject)
		graph_inj.emit(sys.stdout, False)
	else:
		print("Please provide a .bed file as a command line argument.")
