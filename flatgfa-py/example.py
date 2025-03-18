import flatgfa
from collections import Counter

# graph = flatgfa.parse("../tests/k.gfa")
graph = flatgfa.parse("/Users/hangji/Desktop/pollen/flatgfa-py/chr6.gfa")
gaf = "/Users/hangji/Desktop/pollen/flatgfa-py/chr6gaf.gaf"
gaf_parser = graph.load_gaf(gaf)

for read in gaf_parser:
    print(read.name)
    print(read.get_sequence())
    print(read.get_seg())
    for element in read.chunk_list:
        print(element.handle)
        print(element.range)
          