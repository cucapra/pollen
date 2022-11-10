'''
This file converts an odgi graph to numerical JSON data that can be used by calyx hardware simulators.
'''

import sys
import argparse
import json
import odgi
import warnings

# Defaults for the maximum possible number of nodes, steps per node, and paths to consider
MAX_NODES=16
MAX_STEPS=15
MAX_PATHS=15

def parse_odgi(filename, subset_paths, pe):
    '''
    Create a calyx node depth input file using the graph in './{filename}' and the paths listed in './{subset_paths}'
    '''

    graph = odgi.graph()
    graph.load(filename)
    
    # Check that the number of paths on the graph does not exceed max_paths
    if graph.get_path_count() > pe.max_paths:
        raise Exception(f'The number of paths in the graph exceeds the maximum number of paths the hardware can process. {graph.get_path_count()} > {pe.max_paths}. Hint: try setting the maximum number of paths manually using the -p flag')

    # Check that the number of nodes on the node does not exceed max_nodess
    if graph.get_node_count() > pe.max_nodes:
        warnings.warn('The number of nodes on the graph exceeds the maximum number of nodes the hardware can process. Hint: try setting the maximum number of nodes manually using the -n flag.')    
    
    # Assign a path_id to each path; the path_ids are not accessible using the
    # default python bindings for odgi
    
    # Obtain a list of path names; a path's index is its id
    paths = []
    graph.for_each_path_handle(lambda h: paths.append(graph.get_path_name(h)))
    # Path name -> path id                                                     
    path_name_to_id = {path:count for count, path in enumerate(paths, start=1)}
    
    data = {}
    for node_id in range(1, pe.max_nodes + 1):
        data.update(pe.parse_node(graph, node_id, path_name_to_id))

    data.update(pe.parse_global_var(graph, subset_paths, path_name_to_id))

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


def get_dimensions(args):
    '''
    Compute the node depth accelerator dimensions from commandline input
    '''
    if args.auto_size:
        filename = args.filename if args.auto_size=='d' else args.auto_size
        max_nodes, max_steps, max_paths = get_maxes(filename)
    else:
        max_nodes, max_steps, max_paths = MAX_NODES, MAX_STEPS, MAX_PATHS
        
    max_nodes = args.max_nodes if args.max_nodes else max_nodes
    max_steps = args.max_steps if args.max_steps else max_steps
    max_paths = args.max_paths if args.max_paths else max_paths

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


def run(args):
    if args.from_verilog or args.from_interp:
        with open(filename, 'r') as fp:
            data = json.load(fp)
        ouput = from_calyx(data, args.from_interp)
    else:
        max_nodes, max_steps, max_paths = get_dimensions(args)

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
