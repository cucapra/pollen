'''
This file converts an odgi graph to numerical JSON data that can be used by calyx hardware simulators.
'''

import sys
import argparse
import json
import odgi

# Defaults for the maximum possible number of nodes, steps per node, and paths to consider
MAX_NODES=16
MAX_STEPS=15
MAX_PATHS=15

def parse_odgi(filename, subset_paths, max_nodes, max_steps, max_paths):
    '''
    Create a calyx node depth input file using the graph in './{filename}' and the paths listed in './{subset_paths}'
    '''

    graph = odgi.graph()
    graph.load(filename)
    
    # Check that the number of paths on the graph does not exceed max_paths
    if graph.get_path_count() > max_paths:
        raise Exception(f'The number of paths in the graph exceeds the maximum number of paths the hardware can process. {graph.get_path_count()} > {args.max_paths}. Hint: try setting the maximum number of paths manually using the -p flag')

    
    # Assign a path_id to each path; the path_ids are not accessible using the
    # default python bindings for odgi
    
    # Obtain a list of path names; a path's index is its id
    paths = []
    graph.for_each_path_handle(lambda h: paths.append(graph.get_path_name(h)))
    
    # Path name -> path id                                                     
    path_name_to_id = {path:count for count, path in enumerate(paths, start=1)}

    
    paths_to_consider = parse_paths_file(subset_paths, path_name_to_id, max_paths)

    data = parse_steps_on_nodes(graph, path_name_to_id, max_nodes, max_steps, max_paths)

    for i in range(1, max_nodes + 1):
        data[f'paths_to_consider{i}'] = {
            "data": paths_to_consider,
            "format": {
                "numeric_type": "bitnum",
                "is_signed": False,
                "width": 1
            }
        }

    return data
    

def parse_steps_on_nodes(graph, path_name_to_id, max_nodes=MAX_NODES, max_steps=MAX_STEPS, max_paths=MAX_PATHS):
    '''
    Generate input data containing the path ids for each step on each node in the graph, e.g.
    {path_ids1: 
        "data": [0, 1, 1, 2],
            "format": {
                "numeric_type": "bitnum",8kklkskl
                "is_signed": False,
                "width": 2
            }
    }
    '''

    num_nodes = graph.get_node_count()
    
    # Check that the number of steps on the node does not exceed max_steps
    if num_nodes > max_nodes:
        raise Exception(f'The number of nodes in the graph exceeds the maximum number of nodes the hardware can process. Hint: try setting the maximum number of nodes manually using the -n flag.')
    
    data = {}
    width = max_paths.bit_length()

    # Initialize the data for each node
    def parse_node(node_h):
        '''
        Get a list of path ids for each step on node_h.
        '''

        # Check that the number of steps on the node does not exceed max_steps
        if graph.get_step_count(node_h) > max_steps:
            raise Exception(f'The number of paths in the graph exceeds the maximum number of paths the hardware can process. {graph.get_step_count(node_h)} > {max_steps}. Hint: try setting the maximum number of steps manually using the -e flag.')
        
        path_ids = []

        def parse_step(step_h):
            path_h = graph.get_path(step_h)
            path_id = path_name_to_id[graph.get_path_name(path_h)]
            path_ids.append(path_id)
            
        graph.for_each_step_on_handle(node_h, parse_step)

        # Pad path_ids with 0s
        path_ids = path_ids + [0] * (max_steps - len(path_ids))
        
        # 'path_ids{id}' is the list of path ids for each step crossing node {id}
        node_id = graph.get_id(node_h)
        data[f'path_ids{node_id}'] = {
            "data": path_ids,
            "format": {
                "numeric_type": "bitnum",
                "is_signed": False,
                "width": width
            }
        }

    graph.for_each_handle(parse_node)

    default_steps = [0] * max_steps
    
    while num_nodes < max_nodes:
        num_nodes += 1
        data[f'path_ids{num_nodes}'] = {
            "data": default_steps,
            "format": {
                "numeric_type": "bitnum",
                "is_signed": False,
                "width": width
            }
        }

    data['depth_output'] = {
        "data": [0] * max_nodes,
        "format": {
            "numeric_type": "bitnum",
            "is_signed": False,
            "width": max_steps.bit_length()
        }
    }

    data['uniq_output'] = {
        "data": [0] * max_nodes,
        "format": {
            "numeric_type": "bitnum",
            "is_signed": False,
            "width": max_paths.bit_length()
        }
    }
        
    return data


def parse_paths_file(filename, path_name_to_id, max_paths=MAX_PATHS):
    '''
    Return paths_to_consider, a list of length max_paths, where 
    paths_to_consider[i] is 1 if i is a path id and we include path i in our
    calculations of node depth
    '''
    
    if filename is None: # Return the default value
        paths_to_consider = [1]*(max_paths + 1)
        paths_to_consider[0] = 0
        return paths_to_consider

    with open(filename, 'r') as paths_file:
        text = paths_file.read()
        paths = text.splitlines()

    paths_to_consider = [0] * (max_paths + 1)
        
    for path_name in paths:
        path_id = path_name_to_id[path_name]
        paths_to_consider[path_id] = 1

    return paths_to_consider


def get_maxes(filename):
    
    graph = odgi.graph()
    graph.load(filename)
    
    max_nodes = graph.get_node_count()
    max_steps = 0
    max_paths = graph.get_path_count()

    def update_max_steps(node_h):
        nonlocal max_steps
        num_steps = graph.get_step_count(node_h)
        if num_steps > max_steps:
            max_steps = num_steps

    graph.for_each_handle(update_max_steps)

    return max_nodes, max_steps, max_paths


def from_calyx(calyx_out, from_interp, max_nodes=None):
    '''
    Parse a calyx output file to the odgi format
    '''

    if from_interp:
        depths = calyx_out['main']['depth_output']
        uniqs = calyx_out['main']['uniq_output']
    else:
        depths = calyx_out['memories']['depth_output']
        uniqs = calyx_out['memories']['uniq_output']

    if not max_nodes:
        max_nodes = len(depths)

    header = "#node.id\tdepth\tdepth.uniq"
    rows = '\n'.join([f'{i + 1}\t{depths[i]}\t{uniqs[i]}' for i in range(max_nodes)])
    return '\n'.join([header, rows])


def config_parser(parser):
    
    parser.add_argument(
        'filename',
        help='The file to be parsed. If the -d and -i flags are not specified, this must be an odgi file.'
    )
    parser.add_argument(
        '-s',
        '--subset-paths',
        help='Specify a file containing a subset of all paths in the graph. See the odgi documentation for more details.'
    )
    parser.add_argument(
        '-v',
        '--from-verilog',
        action='store_true',
        help='Specify that the given file is a calyx data file to be converted to the odgi ouput format.'
    )
    parser.add_argument(
        '-i',
        '--from-interp',
        action='store_true',
        help='Specify that the given file is a calyx interpreter output file to be converted to the odgi output format.'
    )
    parser.add_argument(
        '-a',
        '--auto-size',
        nargs='?',
        const='d',
        help='Provide an odgi file that will be used to calculate the hardware dimensions. If the flag is specified with no argument, use the file to be parsed. Specified hardware dimensions take precedence.'
    )
    parser.add_argument(
        '-n',
        '--max-nodes',
        type=int,
        default=MAX_NODES,
        help='Specify the maximum number of nodes that the hardware can support.'
    )
    parser.add_argument(
        '-e',
        '--max-steps',
        type=int,
        default=MAX_STEPS,
        help='Specify the maximum number of steps per node that the hardware can support.')
    parser.add_argument(
        '-p',
        '--max-paths',
        type=int,
        default=MAX_PATHS,
        help='Specify the maximum number of paths that the hardware can support.'
    )
    parser.add_argument(
        '-o',
        '--out',
        help='Specify the output file. If not specified, will dump to stdout.'
    )

def run(args):
    if args.from_verilog or args.from_interp:
        with open(filename, 'r') as fp:
            data = json.load(fp)
        ouput = from_calyx(data, args.from_interp)
    else:
        if args.auto_size:
            filename = args.filename if args.auto_size=='d' else args.auto_size
            max_nodes, max_steps, max_paths = get_maxes(filename)
        else:
            max_nodes = args.max_nodes
            max_steps = args.max_steps
            max_paths = args.max_paths

        data = parse_odgi(args.filename, args.subset_paths, max_nodes, max_steps, max_paths)
        output = json.dumps(data, indent=2, sort_keys=True) 

    if args.out:
        with open(args.out, 'w') as out_file:
            out_file.write(output)
    else:
        print(output)
        

if __name__ == '__main__':
    
    # Parse commandline arguments                                              
    parser = argparse.ArgumentParser()
    config_parser(parser)
    args = parser.parse_args()
    run(args)
