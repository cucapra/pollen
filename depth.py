import argparse
import sys
import odgi
''' 
The documentation for the odgi module can be found at 
https://odgi.readthedocs.io/en/latest/rst/binding/glossary.html 
'''

def get_depth_table(graph, subset_paths=None):
    ''' 
    Input: an odgi.graph object
    Output: the node depth table, a dictionary that maps from a node's id to it's (depth, uniq_depth),
        where depth is the total number of times each path in subset_paths crosses the node,
        and uniq_depth is the number of paths in subset_paths which cross the node
    Node: if subset_paths is empty, consider all paths when computing node depth
    '''

    ndt = dict() # node depth table map from node.id -> (node.depth, node.uniq_depth)

    # Compute the set of paths we want to consider
    if subset_paths is None: # By default, we want to consider all paths
        paths_to_consider = set()
        # Adds the name of each path in graph to paths_to_consider
        graph.for_each_path_handle(lambda h: paths_to_consider.add(graph.get_path_name(h)))
    else: # Consider only paths specified, if a subet of paths is specified
        paths_to_consider = set(subset_paths)

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

def parse_paths_file(filename):
    ''' Parse a file which contains the name of a path on each line. '''

    if filename is None: # Return the default value
        return None
    
    with open(filename, 'r') as paths_file:
        text = paths_file.read()
        paths = text.splitlines()
    return paths

if __name__ == '__main__':
    # Parse commandline arguments
    parser = argparse.ArgumentParser()
    parser.add_argument('filename', help='A .og file representing a pangenome whose node depth we want to calculate')
    parser.add_argument('-s', '--subset-paths', help='Specify a file containing a subset of all paths in the graph. See the odgi documentation for more details')
    args = parser.parse_args()

    graph = odgi.graph()
    graph.load(args.filename)

    # Get the set of all paths specified in the file give
    subset_paths = parse_paths_file(args.subset_paths)
    
    # Get the node depths for all nodes in the graph
    ndt = get_depth_table(graph, subset_paths)

    # Print the ndt to the standard output
    print("#node.id\tdepth\tdepth.uniq")
    for id, (depth, uniq) in sorted(ndt.items()):
        print(f'{id}\t{depth}\t{uniq}')
