import flatgfa

graph = flatgfa.parse("name-of-GFA-file")
gaf = "name-of-GAF-file"
gaf_parser = graph.all_reads(gaf)
for lines in gaf_parser:
    print(lines.name)
    print(lines.sequence())
    print(lines.segment_ranges())
    for element in lines:
        print(element.handle)
        print(element.range)
