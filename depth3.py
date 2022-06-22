import sys
import odgi

def get_depth_table(graph, subset_paths=[]):
    ''' 
    Input: an odgi.graph object
    Output: a dictionary that maps from a node_id to (node.depth, node.uniq),
        where node.depth is the total node depth, and node.uniq is the number of
        paths which cross the node
    '''

    ndt = dict()
    
    if subset_paths: # Consder only paths specified, if specified
        paths_to_consider = set(subset_paths)
    else: # By default, we want to consider all paths
        paths_to_consider = set()
        graph.for_each_path_handle(lambda h: paths_to_consider.add(graph.get_path_name(h)))
    
    def get_node_depth(handle):
        '''
        Input: [handle] is an odgi.handle object representing a node
        Inserts node.depth and node.uniq into ndt for the node associated with
            [handle]
        '''
        paths = set()
        depth = [0]
        
        def for_step(step):
            path_h = graph.get_path_handle_of_step(step)
            if graph.get_path_name(path_h) in paths_to_consider:
                paths.add(path_h)
                depth[0] = depth[0] + 1

        graph.for_each_step_on_handle(handle, for_step)

        node_id = graph.get_id(handle)
        ndt[node_id] = (depth[0], len(paths))
        
    graph.for_each_handle(get_node_depth)
    return ndt

if __name__ == '__main__':
    graph = odgi.graph()
    graph.load(sys.argv[1])
    ndt = get_depth_table(graph)
    print("#node.id\tdepth\tdepth.uniq")
    for id, (depth, uniq) in sorted(ndt.items()):
        print(f'{id}\t{depth}\t{uniq}')
