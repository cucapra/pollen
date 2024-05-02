import flatgfa

g = flatgfa.parse("../bench/graphs/test.k.gfa")
print(g.segments[2])
for seg in g.segments:
    print(seg.name, seg.id, seg.sequence())

g = flatgfa.load("../bench/graphs/test.k.flatgfa")
print(len(g.segments))
