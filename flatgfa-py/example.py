import flatgfa
from collections import Counter

# graph = flatgfa.parse("../tests/k.gfa")
graph = flatgfa.parse("../tests/basic/ex1.gfa")
# depths = Counter()
# for path in graph.paths:
#     for step in path:
#         depths[step.segment.id] += 1

# print("#node.id\tdepth")
# for seg in graph.segments:
#     print("{}\t{}".format(seg.name, depths[seg.id]))

graph.depth()