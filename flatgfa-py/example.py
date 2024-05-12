import flatgfa

g = flatgfa.parse("../bench/graphs/test.k.gfa")
print(g.segments[2])
for seg in g.segments:
    print(seg.name, seg.id, seg.sequence())

g = flatgfa.load("../bench/graphs/test.k.flatgfa")
print(len(g.segments))
for path in g.paths:
    print(path, path.name)
    print(len(path))
    print(','.join(
        '{}{}'.format(
            s.segment.name,
            '+' if s.is_forward else '-'
        )
        for s in path
    ))
for link in g.links:
    print(link, link.from_, link.to)

print(g.paths.find(b"x"))
print(g.segments.find(2))

print(g)
g.write_gfa("temp.gfa")
g.write_flatgfa("temp.flatgfa")
