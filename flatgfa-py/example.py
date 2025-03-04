import flatgfa
from collections import Counter

graph = flatgfa.load("chr6.flatgfa")
print(graph.size)
# graph.print_gaf_lookup("ch6.gaf")

for read in graph.test_gaf("ch6.gaf"):
    for event in read:
        print(event.handle)
        print(event.range)
        print(event.get_seq(graph))