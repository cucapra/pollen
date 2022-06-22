import gfapy
import sys

def depth(filename):
    g = gfapy.Gfa.from_file(filename)
    # Maps [node] to its node depth
    depth_map = {node:0 for node in g.segment_names}
    # Maps [node] to the set of all paths that cross [node]
    path_map = {node:set() for node in g.segment_names}
    
    for path in g.paths:
        
        for node in path.segment_names:
            name = node.name # Node name
            depth_map[name] = depth_map[name] + 1
            path_map[name].add(path)
            
        # the node depth table maps from a node's id to its depth and unique depth
        ndt = {int(node):(depth_map[node], len(path_map[node])) for node in g.segment_names}
    sorted_ndt = sorted(ndt.items())
    return sorted_ndt

if __name__ == '__main__':
    sorted_ndt = depth(sys.argv[1])
    print("#node.id\tdepth\tdepth.uniq")
    for node, (depth, unique) in sorted_ndt:
        print(f'{node}\t{depth}\t{unique}')
