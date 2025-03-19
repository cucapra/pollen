import flatgfa
from collections import Counter


# graph = flatgfa.parse("../tests/k.gfa")
# depths = Counter()
# for path in graph.paths:
#     for step in path:
#         depths[step.segment.id] += 1

# print("#node.id\tdepth")
# for seg in graph.segments:
#     print("{}\t{}".format(seg.name, depths[seg.id]))

# graph = flatgfa.load("chr22.flatgfa")
graph = flatgfa.load("chr6.flatgfa")
print(graph.size)
# graph.print_gaf_lookup("ch6.gaf")

# event = graph.test_gaf("ch6.gaf")
# print(event.handle)
# print(event.range)

for read in graph.test_gaf("ch6.gaf"):
    for event in read:
        print(event.handle)
        print(event.range)
        print(event.get_seq())