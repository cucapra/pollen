import sys
import parse_gfa

def depth(filename):
    '''
    Calculates the depth of each node by counting how many times a node appears
    in graph
    '''
    g = parse_gfa.parse_file_path_only(filename)
    depth_map = {}

    for path in g.paths:
        for segment in path.segment_names:
            name = int(segment.name)
            if name in depth_map:
                depth_map[name] += 1
            else:
                depth_map[name] = 1
    sorted_depth_items = sorted(depth_map.items())

    with open('python_output.txt', 'w') as f: 
        for pair in sorted_depth_items:
            f.write(f'{pair[0]} {pair[1]}\n')

if __name__ == '__main__':
    depth(sys.argv[1])
