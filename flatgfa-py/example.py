import flatgfa
g = flatgfa.parse("../tests/k.gfa")
for seg in g.segments:
    print(seg.name, seg.sequence())
