import gfapy
import sys

def depth(filename):
    g = gfapy.Gfa.from_file(filename)
    depth_map = {}

    for path in g.paths:
        for segment in path.segment_names:
            name = int(segment.name)
            if name in depth_map:
                depth_map[name] += 1
            else:
                depth_map[name] = 1
    sorted_node_depths = sorted(depth_map.items())
    return sorted_node_depths

if __name__ == '__main__':
    sorted_node_depths = depth(sys.argv[1])
    print("#node.id\tdepth")
    for node, depth in sorted_node_depths:
        print(f'{node}\t{depth}')
