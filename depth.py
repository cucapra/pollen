""" A node depth computation for .og files implemented using odgi's
Python bindings. While this implementation reuses odgi's data structures, it
does not reuse its node depth computation algorithm and instead implements
it from scratch.

The documentation for the odgi module can be found at 
https://odgi.readthedocs.io/en/latest/rst/binding/glossary.html 
"""

import argparse
import sys
import odgi

def get_depth_table(graph, subset_paths=None):
    ''' 
    Input: an odgi.graph object
    Output: the node depth table, a dictionary that maps from a node's id to its (depth, uniq_depth),
        where depth is the total number of times each path in subset_paths crosses the node,
        and uniq_depth is the number of paths in subset_paths which cross the node
    Note: if subset_paths is empty, consider all paths when computing node depth
    '''

    ndt = dict() # node depth table map from node.id -> (node.depth, node.uniq_depth)

    # Compute the node depth and unique depth
    def get_node_depth(handle):
        '''
        Input: [handle] is an odgi.handle object which represents a node
        Inserts node.depth and node.uniq into ndt for the node associated with
            [handle]
        '''

        # Note: a node can have multiple handles, but only one id
        node_id = graph.get_id(handle)
        
        paths = set()
        depth = 0 # depth[0] is the node depth

        # For a given path step, update the node depth and set of paths which cross the node
        def for_step(step):
            path_h = graph.get_path_handle_of_step(step)
            path = graph.get_path_name(path_h) # The name of the path associated with path_h
            if not subset_paths or path in subset_paths:
                paths.add(path)
                nonlocal depth # modify the 'depth' variable in the outer scope
                depth += 1

        graph.for_each_step_on_handle(handle, for_step)

        ndt[node_id] = (depth, len(paths))
        
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
