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
