import argparse
import importlib
from math import ceil
from os.path import abspath, split, splitext
import subprocess
import sys

from calyx.py_ast import *
from pollen.depth import parse_data


def node_depth(max_nodes, max_steps, max_paths, pe_component, num_pes=None):

    num_pes = num_pes if num_pes else max_nodes

    stdlib = Stdlib()
    
    # Variable identifiers
    depth_output = CompVar('depth_output')
    uniq_output = CompVar('uniq_output')
    ptc_idx = CompVar('ptc_idx') # For copying ptc[0] for other pes
    ptc_neq = CompVar('ptc_neq') # To check if ptc_idx neq 0, for while loop
    ptc_idx_dec = CompVar('ptc_idx_dec') # Decrement ptc_idx

    depth = [] # registers for storing depth
    uniq = []
    path_ids = [] # path_id for each step on the node
    ptc = [] # paths to consider. duplicated, each node gets its own array
    
    pe = [] #processing elements
    
    for i in range(1, max_nodes + 1):
        depth.append(CompVar(f'depth{i}'))
        uniq.append(CompVar(f'uniq{i}'))
        path_ids.append(CompVar(f'path_ids{i}'))

    for i in range(1, num_pes + 1):
        pe.append(CompVar(f'pe{i}'))
        ptc.append(CompVar(f'paths_to_consider{i}'))
        

    # Initialize the cells
    ptc_size = max_paths + 1
    path_id_width = max_paths.bit_length()
    depth_width = max_steps.bit_length() # number of bits to represent depth
    uniq_width = path_id_width # number of bits to represent uniq depth
    steps_width = max((max_steps - 1).bit_length(), 1)
    node_width = max((max_nodes - 1).bit_length(), 1)
    
    cells = [
        # External memory cells for the output
        Cell(
            depth_output,
            stdlib.mem_d1(depth_width, max_nodes, node_width),
            is_external=True
        ),
        Cell(
            uniq_output,
            stdlib.mem_d1(uniq_width, max_nodes, node_width),
            is_external=True
        ),
        # Metadata cells
        Cell(
            ptc[0],
            stdlib.mem_d1(1, ptc_size, path_id_width),
            is_external=True
        ),
        Cell(ptc_idx, stdlib.register(path_id_width)),
        Cell(ptc_neq, stdlib.op('neq', path_id_width, signed=False)),
        Cell(ptc_idx_dec, stdlib.op('sub', path_id_width, signed=False))
    ]

    for i in range(max_nodes):
        cells.extend([
            Cell(depth[i], stdlib.register(depth_width)),
            Cell(uniq[i], stdlib.register(uniq_width)),
            Cell(
                path_ids[i],
                stdlib.mem_d1(path_id_width, max_steps, steps_width),
                is_external=True
            )
        ])

    for i in range(num_pes):
        cells.append(Cell(pe[i], CompInst('node_depth_pe', [])))

    for i in range(1, num_pes):
        cells.append(Cell(ptc[i], stdlib.mem_d1(1, ptc_size, path_id_width)))
    
    
    # Initialize the wires
    wires = []

    for i in range(max_nodes):
        wires.extend([
            Group(
                CompVar(f'store_depth{i}'),
                [
                    Connect(
                        CompPort(depth_output, "addr0"),
                        ConstantPort(node_width, i)
                    ),
                    Connect(
                        CompPort(depth_output, 'write_data'),
                        CompPort(depth[i], 'out')
                    ),
                    Connect(
                        CompPort(depth_output, 'write_en'),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        HolePort(CompVar(f'store_depth{i}'), 'done'),
                        CompPort(depth_output, 'done')
                    )
                ]
            ),
            Group(
                CompVar(f"store_uniq{i}"),
                [
                    Connect(
                        CompPort(uniq_output, 'addr0'),
                        ConstantPort(node_width, i)
                    ),
                    Connect(
                        CompPort(uniq_output, 'write_data'),
                        CompPort(uniq[i], 'out')
                    ),
                    Connect(
                        CompPort(uniq_output, 'write_en'),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        HolePort(CompVar(f'store_uniq{i}'), 'done'),
                        CompPort(uniq_output, 'done')
                    )
                ]
            ),
        ])

    # Initialize metadata
    wires.extend([
        Group(
            CompVar('init_ptc_idx'),
            [
                Connect(
                    CompPort(ptc_idx, 'in'),
                    ConstantPort(path_id_width, max_paths)
                ),
                Connect(
                    CompPort(ptc_idx, 'write_en'),
                    ConstantPort(1, 1)
                ),
                Connect(
                    HolePort(CompVar('init_ptc_idx'), 'done'),
                    CompPort(ptc_idx, 'done')
                )
            ]
        ),
        CombGroup(
            CompVar('compare_ptc_idx'),
            [
                Connect(CompPort(ptc_neq, 'left'), CompPort(ptc_idx, 'out')),
                Connect(
                    CompPort(ptc_neq, 'right'),
                    ConstantPort(path_id_width, 0)
                ),
            ]
        ),
        Group(
            CompVar('dec_ptc_idx'),
            [
                Connect(
                    CompPort(ptc_idx_dec, 'left'),
                    CompPort(ptc_idx, 'out')
                ),
                Connect(
                    CompPort(ptc_idx_dec, 'right'),
                    ConstantPort(path_id_width, 1)
                ),
                Connect(
                    CompPort(ptc_idx, 'in'),
                    CompPort(ptc_idx_dec, 'out'),
                ),
                Connect(
                    CompPort(ptc_idx, 'write_en'),
                    ConstantPort(1, 1)
                ),
                Connect(
                    HolePort(CompVar('dec_ptc_idx'), 'done'),
                    CompPort(ptc_idx, 'done')
                )
            ]
        )
    ])

    cpy_elt_connections = [
        Connect(
            CompPort(ptc[0], 'addr0'),
            CompPort(ptc_idx, 'out')
        )
    ]
    for i in range(1, num_pes):
        cpy_elt_connections.extend([
            Connect(
                CompPort(ptc[i], 'addr0'),
                CompPort(ptc_idx, 'out')
            ),
            Connect(
                CompPort(ptc[i], 'write_data'),
                CompPort(ptc[0], 'read_data')
            ),
            Connect(
                CompPort(ptc[i], 'write_en'),
                ConstantPort(1, 1)
            )
        ])
    # WARNING VERY DUMB DO NOT TRY THIS AT HOME
    cpy_elt_guard_expr = Atom(CompPort(ptc[1], 'done'))
    for i in range(2, num_pes - 1):
        cpy_elt_guard_expr = And(cpy_elt_guard_expr, CompPort(ptc[i], 'done'))
        
    wires.append(
        Group(
            CompVar('cpy_elt'),
            cpy_elt_connections + [
                Connect(
                    HolePort(CompVar('cpy_elt'), 'done'),
                    CompPort(ptc[num_pes - 1], 'done'),
                    guard=cpy_elt_guard_expr
                )
            ]
        )
    )

        
    # Define control flow
    
    # Initialize paths_to_consider for each pe
    controls = [
        Enable('init_ptc_idx'),
        While(
            CompPort(ptc_neq, 'out'),
            CompVar('compare_ptc_idx'),
            SeqComp([Enable('cpy_elt'), Enable('dec_ptc_idx')])
        ),
        Enable('cpy_elt')
    ]

    # Compute depth and uniq depth
    pe_controls = []
    for i in range(num_pes):
        pe_i_controls = []
        for j in range(i, max_nodes, num_pes):
            in_connects = [
                ('pids_read_data', CompPort(path_ids[j], 'read_data')),
                ('ptc_read_data', CompPort(ptc[i], 'read_data')),
                ('depth_out', CompPort(depth[j], 'out')),
                ('depth_done', CompPort(depth[j], 'done')),
                ('uniq_out', CompPort(uniq[j], 'out')),
                ('uniq_done', CompPort(uniq[j], 'done')),
            ]

            out_connects = [
                ('pids_addr0', CompPort(path_ids[j], 'addr0')),
                ('ptc_addr0', CompPort(ptc[i], 'addr0')),
                ('depth_in', CompPort(depth[j], 'in')),
                ('depth_write_en', CompPort(depth[j], 'write_en')),
                ('uniq_in', CompPort(uniq[j], 'in')),
                ('uniq_write_en', CompPort(uniq[j], 'write_en')),
            ]
            
            pe_i_controls.append(
                Invoke(id=pe[i],
                       in_connects=in_connects,
                       out_connects=out_connects,
                )
            )
        pe_controls.append(SeqComp(pe_i_controls))
    controls.extend([ParComp(pe_controls)])
                
    for i in range(max_nodes):
        controls.append(
            ParComp([
                Enable(f'store_uniq{i}'),
                Enable(f'store_depth{i}')
            ])
        )
    
    main_component = Component(
        name="main",
        inputs=[],
        outputs=[],
        structs=cells + wires,
        controls=SeqComp(controls),
    )

    # Create the Calyx program.
    program = Program(
        imports=[
            Import("primitives/core.futil"),
            Import("primitives/binary_operators.futil")
        ],
        components=[main_component, pe_component]
    )

    return program


def config_parser(parser):
    
    parser.add_argument(
        '-a',
        '--auto-size',
        help='Provide an odgi file that will be used to calculate the hardware dimensions.'
    )
    parser.add_argument(
        '-n',
        '--max-nodes',
        type=int,
        help='Specify the maximum number of nodes that the hardware can support.'
    )
    parser.add_argument(
        '-e',
        '--max-steps',
        type=int,
        help='Specify the maximum number of steps per node that the hardware can support.'
    )
    parser.add_argument(
        '-p',
        '--max-paths',
        type=int,
        help='Specify the maximum number of paths that the hardware can support.'
    )

    parser.add_argument(
        '--pe',
        '--processing-element',
        default='default',
        help='Provide a file containing the method node_depth_pe which outputs a node depth processing element.'
    )
    parser.add_argument(
        '--num-pes',
        type=int,
        help='Specify the number of processing elements that the generated hardware will use.'
    )
    
    parser.add_argument(
        '-o',
        '--out',
        help='Specify the output file. If not specified, will dump to stdout.'
    )


def run(args):

    # Import the processing element generator
    if args.pe == 'default':
        pe_module = importlib.import_module('pollen.depth.processing_elements.simple')
    else:
        pe_path, pe_file = split(args.pe)
        pe_abspath = abspath(pe_path)
        pe_basename = splitext(pe_file)[0]
        # Add pe generator path to PYTHONPATH
        sys.path.append(pe_abspath)
        pe_module = importlib.import_module(pe_basename)

    max_nodes, max_steps, max_paths = parse_data.get_dimensions(args)        
    pe_component = pe_module.node_depth_pe(max_steps, max_paths)
        
    program = node_depth(max_nodes, max_steps, max_paths,
                         pe_component, args.num_pes)
    output = program.doc()

    # Ouput the program
    if (args.out):
        with open(args.out, 'w') as out_file:
            out_file.write(output)
    else:
        print(output)

        
if __name__ == '__main__':

    # Parse commandline input
    parser = argparse.ArgumentParser()
    config_parser(parser)
    args = parser.parse_args()

    run(args)
