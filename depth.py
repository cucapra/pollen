import sys
import parse_gfa

def depth(filename):
    '''
    Calculates the depth of each node by counting how many times a node appears
    in graph
    '''
    g = parse_gfa.format_file_path_only(filename)
    depth_map = {}
    for segment in g:
        if segment in depth_map:
            depth_map[segment] += 1
        else:
            depth_map[segment] = 1
    print(depth_map)


if __name__ == '__main__':
    depth(sys.argv[1])
