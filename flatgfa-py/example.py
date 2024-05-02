import flatgfa
g = flatgfa.parse("../tests/k.gfa")
print(g.segments[2])
for seg in g.segments:
    print(seg.name, seg.id, seg.sequence())
