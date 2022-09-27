from calyx.py_ast import *
from parse_data import get_maxes
import argparse


MAX_NODES=16
MAX_STEPS=15
MAX_PATHS=15

def node_depth(max_nodes=MAX_NODES, max_steps=MAX_STEPS, max_paths=MAX_PATHS):

    stdlib = Stdlib()
    
    # Variable identifiers
    depth_output = CompVar('depth_output')
    uniq_output = CompVar('uniq_output')

    path_ids = [] # path_id for each step on the node
    paths_on_node = [] # computed by depth.uniq
    paths_to_consider = [] # duplicated, each node gets its own array

    path_id_reg = []
    idx = []
    idx_adder = []
    idx_neq = []
    
    depth = []
    depth_temp = []
    depth_pad = []
    depth_adder = []
    
    uniq = []
    uniq_and = []
    uniq_and_reg_l = []
    uniq_and_reg_r = []
    uniq_pad = []
    uniq_adder = []
    
    uniq_idx = []
    uniq_idx_neq = []
    uniq_idx_adder = []

    
    for i in range(1, max_nodes + 1):
        path_ids.append(CompVar(f'path_ids{i}'))
        paths_on_node.append(CompVar(f'paths_on_node{i}'))
        paths_to_consider.append(CompVar(f'paths_to_consider{i}')) 

        path_id_reg.append(CompVar(f'path_id_reg{i}'))
        idx.append(CompVar(f'idx{i}'))
        idx_adder.append(CompVar(f'idx_adder{i}'))
        idx_neq.append(CompVar(f'idx_neq{i}'))

        depth.append(CompVar(f'depth{i}'))
        depth_temp.append(CompVar(f'depth_temp{i}'))
        depth_pad.append(CompVar(f'depth_pad{i}'))
        depth_adder.append(CompVar(f'depth_depth{i}'))

        uniq.append(CompVar(f'uniq{i}'))
        uniq_and.append(CompVar(f'uniq_and{i}'))
        uniq_and_reg_l.append(CompVar(f'uniq_and_reg_l{i}'))
        uniq_and_reg_r.append(CompVar(f'uniq_and_reg_r{i}'))
        uniq_pad.append(CompVar(f'uniq_pad{i}'))
        uniq_adder.append(CompVar(f'uniq_adder{i}'))

        uniq_idx.append(CompVar(f'uniq_idx{i}'))
        uniq_idx_neq.append(CompVar(f'uniq_idx_neq{i}'))
        uniq_idx_adder.append(CompVar(f'uniq_idx_adder{i}'))


    # Initialize the cells
    ptc_size = max_paths + 1
    path_id_width = max_paths.bit_length()
    depth_width = max_steps.bit_length() # number of bits to represent depth
    uniq_width = path_id_width # number of bits to represent uniq depth
    steps_width = max((max_steps - 1).bit_length(), 1)
    node_width = max((max_nodes - 1).bit_length(), 1)
    
    cells = [
        # External memory cells for the output
        Cell(depth_output, stdlib.mem_d1(depth_width, max_nodes, node_width), is_external=True),
        Cell(uniq_output, stdlib.mem_d1(uniq_width, max_nodes, node_width), is_external=True)
    ]

    for i in range(max_nodes):
        cells.extend([
            Cell(path_ids[i], stdlib.mem_d1(path_id_width, max_steps, steps_width), is_external=True),
            Cell(paths_on_node[i], stdlib.mem_d1(1, ptc_size, path_id_width)),
            Cell(paths_to_consider[i], stdlib.mem_d1(1, ptc_size, path_id_width), is_external=True),
            
            # Idx cells
            Cell(path_id_reg[i], stdlib.register(path_id_width)),            
            Cell(idx[i], stdlib.register(steps_width)),
            Cell(idx_adder[i], stdlib.op("add", steps_width, signed=False)),
            Cell(idx_neq[i], stdlib.op("neq", steps_width, signed=False)),

            # Cells for node depth computation
            Cell(depth[i], stdlib.register(depth_width)),
            Cell(depth_temp[i], stdlib.register(1)),
            Cell(depth_pad[i], stdlib.pad(1, depth_width)),
            Cell(depth_adder[i], stdlib.op("add", depth_width, signed=False)),

            # Cells for uniq node depth computation
            Cell(uniq[i], stdlib.register(uniq_width)),
            Cell(uniq_and[i], stdlib.op("and", 1, signed=False)),
            Cell(uniq_and_reg_l[i], stdlib.register(1)),
            Cell(uniq_and_reg_r[i], stdlib.register(1)),
            Cell(uniq_pad[i], stdlib.pad(1, uniq_width)),
            Cell(uniq_adder[i], stdlib.op("add", uniq_width, signed=False)),

            Cell(uniq_idx[i], stdlib.register(path_id_width)),
            Cell(uniq_idx_neq[i], stdlib.op("neq", path_id_width, signed=False)),
            Cell(uniq_idx_adder[i], stdlib.op("sub", path_id_width, signed=False))
        ])
    
    # Initialize the wires
    wires = []

    for i in range(max_nodes):
        wires.extend([
            Group(
                CompVar(f'init_idx{i}'),
                [
                    Connect(CompPort(idx[i], "in"), ConstantPort(steps_width, 0)),
                    Connect(CompPort(idx[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"init_idx{i}"), "done"), CompPort(idx[i], "done"))
                ]
            ),

            Group(
                CompVar(f"load_path_id{i}"),
                [
                    Connect(CompPort(path_ids[i], "addr0"), CompPort(idx[i], "out")),
                    Connect(CompPort(path_id_reg[i], "in"), CompPort(path_ids[i], "read_data")),
                    Connect(CompPort(path_id_reg[i], "write_en"), ConstantPort(1,1)),
                    Connect(HolePort(CompVar(f"load_path_id{i}"), "done"), CompPort(path_id_reg[i], "done")),
                ]
            ),

            Group(
                CompVar(f"inc_idx{i}"),
                [
                    Connect(CompPort(idx_adder[i], "left"), CompPort(idx[i], "out")),
                    Connect(CompPort(idx_adder[i], "right"), ConstantPort(steps_width, 1)),
                    Connect(CompPort(idx[i], "in"), CompPort(idx_adder[i], "out")),
                    Connect(CompPort(idx[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"inc_idx{i}"), "done"), CompPort(idx[i], "done"))
                ]
            ),

            CombGroup(
                CompVar(f"compare_idx{i}"),
                [
                    Connect(CompPort(idx_neq[i], "left"), CompPort(idx[i], "out")),
                    Connect(CompPort(idx_neq[i], "right"), ConstantPort(steps_width, max_steps - 1))
                ]
            ),

            # Node depth wires
            Group(
                CompVar(f"load_consider_path{i}"),
                [
                    Connect(CompPort(paths_to_consider[i], "addr0"), CompPort(path_id_reg[i], "out")),
                    Connect(CompPort(depth_temp[i], "in"), CompPort(paths_to_consider[i], "read_data")),
                    Connect(CompPort(depth_temp[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"load_consider_path{i}"), "done"), CompPort(depth_temp[i], "done"))
                ]
            ),

            Group(
                CompVar(f"inc_depth{i}"),
                [
                    #If path_id is not 0, add 1 to depth
                    Connect(CompPort(depth_adder[i], "left"), CompPort(depth[i], "out")),
                    Connect(CompPort(depth_pad[i], 'in'), CompPort(depth_temp[i], 'out')),
                    Connect(CompPort(depth_adder[i], "right"), CompPort(depth_pad[i], 'out')),
                    Connect(CompPort(depth[i], "in"), CompPort(depth_adder[i], "out")),
                    Connect(CompPort(depth[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"inc_depth{i}"), "done"), CompPort(depth[i], "done"))
                ]
            ),

            Group(
                CompVar(f'store_depth{i}'),
                [
                    Connect(CompPort(depth_output, "addr0"), ConstantPort(node_width, i)),
                    Connect(CompPort(depth_output, 'write_data'), CompPort(depth[i], 'out')),
                    Connect(CompPort(depth_output, 'write_en'), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f'store_depth{i}'), 'done'), CompPort(depth_output, 'done'))
                ]
            ),


            # Uniq node depth wires
            Group(
                CompVar(f'init_uniq_idx{i}'),
                [
                    Connect(CompPort(uniq_idx[i], 'in'), ConstantPort(uniq_width, max_paths)),
                    Connect(CompPort(uniq_idx[i], 'write_en'), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f'init_uniq_idx{i}'), 'done'), CompPort(uniq_idx[i], 'done'))
                ]
            ),

            CombGroup(
                CompVar(f'compare_uniq_idx{i}'),
                [
                    Connect(CompPort(uniq_idx_neq[i], 'left'), CompPort(uniq_idx[i], 'out')),
                    Connect(CompPort(uniq_idx_neq[i], 'right'), ConstantPort(path_id_width, 0))
                ]
            ),

            Group(
                CompVar(f'dec_uniq_idx{i}'),
                [
                    Connect(CompPort(uniq_idx_adder[i], 'left'), CompPort(uniq_idx[i], 'out')),
                    Connect(CompPort(uniq_idx_adder[i], 'right'), ConstantPort(path_id_width, 1)),
                    Connect(CompPort(uniq_idx[i], 'in'), CompPort(uniq_idx_adder[i], 'out')),
                    Connect(CompPort(uniq_idx[i], 'write_en'), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f'dec_uniq_idx{i}'), 'done'), CompPort(uniq_idx[i], 'done'))
                ]
            ),


            Group(
                CompVar(f'update_pon{i}'), # update paths_on_node
                [
                    Connect(CompPort(paths_on_node[i], "addr0"), CompPort(path_id_reg[i], "out")),
                    Connect(CompPort(paths_on_node[i], "write_data"), ConstantPort(1, 1)),
                    Connect(CompPort(paths_on_node[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"update_pon{i}"), "done"), CompPort(paths_on_node[i], "done"))
                ]
            ),

            Group(
                CompVar(f"load_and_l{i}"),
                [
                    Connect(CompPort(paths_on_node[i], "addr0"), CompPort(uniq_idx[i], "out")),
                    Connect(CompPort(uniq_and_reg_l[i], "in"), CompPort(paths_on_node[i], "read_data")),
                    Connect(CompPort(uniq_and_reg_l[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"load_and_l{i}"), "done"), CompPort(uniq_and_reg_l[i], "done"))
                ]
            ),

            Group(
                CompVar(f"load_and_r{i}"),
                [
                    Connect(CompPort(paths_to_consider[i], "addr0"), CompPort(uniq_idx[i], "out")),
                    Connect(CompPort(uniq_and_reg_r[i], "in"), CompPort(paths_to_consider[i], "read_data")),
                    Connect(CompPort(uniq_and_reg_r[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"load_and_r{i}"), "done"), CompPort(uniq_and_reg_r[i], "done"))    
                ]
            ),

            Group(
                CompVar(f"inc_uniq{i}"),
                [
                    Connect(CompPort(uniq_and[i], "left"), CompPort(uniq_and_reg_l[i], "out")),
                    Connect(CompPort(uniq_and[i], "right"), CompPort(uniq_and_reg_r[i], "out")), 
                    Connect(CompPort(uniq_adder[i], "left"), CompPort(uniq[i], "out")),
                    Connect(CompPort(uniq_pad[i], 'in'), CompPort(uniq_and[i], 'out')),
                    Connect(CompPort(uniq_adder[i], "right"), CompPort(uniq_pad[i], 'out')),
                    Connect(CompPort(uniq[i], "in"), CompPort(uniq_adder[i], "out")),
                    Connect(CompPort(uniq[i], "write_en"), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f"inc_uniq{i}"), "done"), CompPort(uniq[i], "done"))
                ]
            ),

            Group(
                CompVar(f"store_uniq{i}"),
                [
                    Connect(CompPort(uniq_output, 'addr0'), ConstantPort(node_width, i)),
                    Connect(CompPort(uniq_output, 'write_data'), CompPort(uniq[i], 'out')),
                    Connect(CompPort(uniq_output, 'write_en'), ConstantPort(1, 1)),
                    Connect(HolePort(CompVar(f'store_uniq{i}'), 'done'), CompPort(uniq_output, 'done'))
                ]
            )
        ])
    

    # Define control flow
    controls = []
    for i in range(max_nodes):
        controls.append(
            SeqComp([
                Enable(f"init_idx{i}"),
                ParComp([
                    Enable(f'init_uniq_idx{i}'),
                    While(
                        CompPort(idx_neq[i], "out"),
                        CompVar(f"compare_idx{i}"),
                        SeqComp([
                            Enable(f"load_path_id{i}"),
                            ParComp([
                                Enable(f'inc_idx{i}'),
                                # Depth computation
                                SeqComp([
                                    Enable(f"load_consider_path{i}"),
                                    Enable(f"inc_depth{i}"),
                                ]),
                                # Uniq computation
                                Enable(f'update_pon{i}')
                            ])
                        ])
                    )
                ]),
                Enable(f"load_path_id{i}"),
                Enable(f"load_consider_path{i}"),
                Enable(f"inc_depth{i}"),
                Enable(f'update_pon{i}'),
                While(
                    CompPort(uniq_idx_neq[i], 'out'),
                    CompVar(f'compare_uniq_idx{i}'),
                    SeqComp([
                        ParComp([Enable(f'load_and_l{i}'), Enable(f'load_and_r{i}')]),
                        Enable(f'inc_uniq{i}'),
                        Enable(f'dec_uniq_idx{i}')    
                    ])    
                )
            ])
        )

    controls = [ParComp(controls)]

    for i in range(max_nodes):
        controls.append(
            ParComp([
                Enable(f'store_uniq{i}'),
                Enable(f'store_depth{i}')
            ])
        )
    
        
    # Node depth
    # Get the path_id
    # If path_id neq 0, add 1 to depth

    # Uniq node depth
    # For each step:
        # Get the path_id
        # set paths_on_node[node][path_id] to 1
    # sum paths_on_node[node] AND paths_to_consider
    
    # Control flow
    # In parallel: for each node
    # In parallel: compute node depth and uniq depth
        # Node depth sequence:
        #     1) Get path_id
        #     2) compute path_id neq 0
        #     3) add 1 to depth if path_id neq 0

        # Uniq depth sequence:
        #     1) Get path_id
        #     2) Set paths_on_node[node][path_id] to 1
        #     3) uniq = sum(paths_on_node[node] & paths_to_consider)
                        

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
        components=[main_component]
    )

    return program

            
if __name__ == '__main__':

    # Parse commandline input
    parser = argparse.ArgumentParser()
    parser.add_argument('-a', '--auto-size', help='Provide an odgi file that will be used to calculate the hardware dimensions.')
    parser.add_argument('-n', '--max-nodes', type=int, default=MAX_NODES, help='Specify the maximum number of nodes that the hardware can support.')
    parser.add_argument('-e', '--max-steps', type=int, default=MAX_STEPS, help='Specify the maximum number of steps per node that the hardware can support.')
    parser.add_argument('-p', '--max-paths', type=int, default=MAX_PATHS, help='Specify the maximum number of paths that the hardware can support.')
    parser.add_argument('-o', '--out', help='Specify the output file. If not specified, will dump to stdout.')

    args = parser.parse_args()


    if args.auto_size:
        max_nodes, max_steps, max_paths = get_maxes(args.auto_size)
        program = node_depth(max_nodes, max_steps, max_paths)
    else:
        # Generate calyx code
        program = node_depth(args.max_nodes, args.max_steps, args.max_paths)

        
    # Emit the code
    if (args.out):
        with open(args.out, 'w') as out_file:
            out_file.write(program.doc())
    else:
        program.emit()
