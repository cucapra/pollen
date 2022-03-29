import gfapy

g = gfapy.Gfa.from_file("k.gfa")
depth_map = {}

for path in g.paths:
    for segment in path.segment_names:
        print(segment)
        name = segment.name
        if name in depth_map:
            depth_map[name] += 1
        else:
            depth_map[name] = 1
print(depth_map)