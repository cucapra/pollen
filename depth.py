import sys
import odgi
''' 
The documentation for the odgi module can be found at 
https://odgi.readthedocs.io/en/latest/rst/binding/glossary.html 
'''

def get_depth_table(graph, subset_paths=[]):
    ''' 
    Input: an odgi.graph object
    Output: the node depth table, a dictionary that maps from a node's id to it's (depth, uniq_depth),
        where depth is the total number of times each path in subset_paths crosses the node,
        and uniq_depth is the number of paths in subset_paths which cross the node
    Node: if subset_paths is empty, consider all paths when computing node depth
    '''

    ndt = dict() # node depth table map from node.id -> (node.depth, node.uniq_depth)

    # Compute the set of paths we want to consider
    if subset_paths: # Consder only paths specified, if specified
        paths_to_consider = set(subset_paths)
    else: # By default, we want to consider all paths
        paths_to_consider = set()
        graph.for_each_path_handle(lambda h: paths_to_consider.add(graph.get_path_name(h)))

    # Compute the node depth and unique depth
    def get_node_depth(handle):
        '''
        Input: [handle] is an odgi.handle object which representing a node
        Inserts node.depth and node.uniq into ndt for the node associated with
            [handle]
        '''

        # Note: a node can have multiple handles, but only one id
        node_id = graph.get_id(handle)
        if node_id in ndt:
            print("doing something")
            return
        
        paths = set()
        depth = [0] # depth[0] is the node depth

        # For a given path step, update the node depth and set of paths which cross the node
        def for_step(step):
            path_h = graph.get_path_handle_of_step(step)
            path = graph.get_path_name(path_h) # The name of the path associated with path_h
            if path in paths_to_consider:
                paths.add(path)
                depth[0] = depth[0] + 1

        graph.for_each_step_on_handle(handle, for_step)

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
