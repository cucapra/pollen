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
    #print(depth_map)

    with open('python_output.txt', 'w') as f: 
        depth_items = depth_map.items()
        for pair in depth_items:
            f.write(f'{pair[0]} {pair[1]}\n')

if __name__ == '__main__':
    depth(sys.argv[1])
