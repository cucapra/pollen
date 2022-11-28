""" A node degree computation for .og files implemented using odgi's
Python bindings. While this implementation reuses odgi's data structures, it
does not reuse its node degree computation algorithm and instead implements
it from scratch.

The documentation for the odgi module can be found at 
https://odgi.readthedocs.io/en/latest/rst/binding/glossary.html 
"""

import argparse
import sys
import odgi

def get_degree_table(graph):
    ''' 
    Input: an odgi.graph object
    Output: the node degree table, a dictionary that maps from a node's id to its (degree, uniq_degree),
        where degree is the total number of times each path in subset_paths crosses the node,
        and uniq_degree is the number of paths in subset_paths which cross the node
    Note: if subset_paths is empty, consider all paths when computing node degree
    '''

    ndt = dict() # node degree table map from node.id -> (node.degree)

    # Compute the node degree and unique degree
    def get_node_degree(handle):
        '''
        Input: [handle] is an odgi.handle object which represents a node
        Inserts node.degree and node.uniq into ndt for the node associated with
            [handle]
        '''

        # Note: a node can have multiple handles, but only one id
        node_id = graph.get_id(handle)
        degree = 0

        def sum_edges(handle):
            nonlocal degree
            degree += 1
        graph.follow_edges(handle, False, sum_edges)
        graph.follow_edges(handle, True, sum_edges)

        ndt[node_id] = ndt.get(node_id, 0) + degree
        
    graph.for_each_handle(get_node_degree)
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
    parser.add_argument('filename', help='A .og file representing a pangenome whose node degree we want to calculate')
    args = parser.parse_args()

    graph = odgi.graph()
    graph.load(args.filename)
    
    # Get the node degrees for all nodes in the graph
    ndt = get_degree_table(graph)

    # Print the ndt to the standard output
    print("#node.id\tdegree")
    for id, degree in sorted(ndt.items()):
        print(f'{id}\t{degree}')

'''
import odgi
import python_degree
graph = odgi.graph()
name = './processing-elements/test/k.og'
graph.load(name)
python_degree.get_degree_table(graph)
'''