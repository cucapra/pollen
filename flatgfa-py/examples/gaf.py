import flatgfa

graph = flatgfa.parse("your-gfa-file")
gaf = "your-gaf-gile"
gaf_parser = graph.load_gaf(gaf)
for read in gaf_parser:
    print(read.name)
    print(read.sequence())
    print(read.segment_ranges())
    for element in read.chunks:
        print(element.handle)
        print(element.range)
