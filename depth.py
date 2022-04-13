import gfapy
import sys


def depth(filename):
    g = gfapy.Gfa.from_file(filename)
    depth_map = {}

    for path in g.paths:
        for segment in path.segment_names:
            name = segment.name
            if name in depth_map:
                depth_map[name] += 1
            else:
                depth_map[name] = 1
    print(depth_map)


if __name__ == '__main__':
    depth(sys.argv[1])
