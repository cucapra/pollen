import sys
import json
import odgi

MAX_NUM_PATHS=12
MAX_NODE_DEPTH=12

def node_depth_odgi(filename, paths_to_consider=[]):
    '''
    Parse an odgi file for the calyx node depth algorithm
    '''
    graph = odgi.graph()
    graph.load(filename)
    shift = graph.min_node_id()
    data = {}

    # Obtain a sorted list of path names. A path's index + 1 is its id
    # Note: path id's are 1 - indexed because 0 indicates the absense of a path
    paths_set = set()
    # Add the name of each path to [paths]
    graph.for_each_path_handle(lambda h: paths_set.add(graph.get_path_name(h)))
    # Convert paths to a sorted list
    paths_list = sorted(paths_set)
    # Path name -> path id
    paths = {path:count for count, path in enumerate(paths_list, start=1)}

    # Initialize the data for each node
    def parse_node(node_h):
        '''
        Get a list of path ids for each step on node_h
        '''
        
        node_id = graph.get_id(node_h)

        path_ids = []

        def parse_step(step_h):
            path_h = graph.get_path(step_h)
            path_id = paths[graph.get_path_name(path_h)]
            path_ids.append(path_id)
            
        graph.for_each_step_on_handle(node_h, parse_step)

        # Pad path_ids with 0s
        path_ids = path_ids + [0] * (MAX_NUM_PATHS + 1 - len(path_ids))
        
        # 'path_ids{id}' is the list of path ids for each step crossing node {id}
        data[f'path_ids{node_id - shift}'] = {
            "data": path_ids,
            "format": {
                "numeric_type": "bitnum",
                "is_signed": False,
                "width": 64
            }
        }

    graph.for_each_handle(parse_node)

    # TODO: pad data with 0s for absent nodes?

    # Initialize the memory for the subset of paths to consider
    consider_paths = [0] * MAX_NUM_PATHS

    if paths_to_consider:
        for path in paths_to_consider:
            path_id = paths[path]
            consider_paths[path_id] = 1

        data['consider_paths'] = {
            "data": consider_paths,
            "format": {
                "numeric_type": "bitnum",
                "is_signed": False,
                "width": 1
            }
        }
    else: # By default, all paths are initialized to 1
        for i in range(len(graph.get_path_count())):
            consider_paths[i] = 1
        
    return data

if __name__ == '__main__':
    '''
    Do the dumbest thing possible and assume the input file (given by commandline)
    is in odgi format and then dump the output to std.out
    '''
    data = node_depth_odgi(sys.argv[1])
    json.dump(data, sys.stdout, indent=2, sort_keys=True)
