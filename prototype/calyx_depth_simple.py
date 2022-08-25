from calyx.py_ast import *
import argparse

MAX_NODES=16
MAX_STEPS=15
MAX_PATHS=15

def node_depth(max_nodes=MAX_NODES, max_steps=MAX_STEPS, max_paths=MAX_PATHS):

    stdlib = Stdlib()
    
    # Variable identifiers
    path_ids = CompVar('path_ids') # path_id for each step on the node
    paths_to_consider = CompVar('paths_to_consider') 
    paths_on_node = CompVar('paths_on_node') # computed by depth.uniq
    depth_output = CompVar('depth_output')
    
    path_id_reg = CompVar('path_id_reg')
    idx = CompVar("idx")
    idx_adder = CompVar("idx_adder")
    idx_neq = CompVar("idx_neq")
    
    depth = CompVar('depth')
    depth_temp = CompVar('depth_temp')
    depth_adder = CompVar('depth_adder')
    
    uniq = CompVar('node_uniq')
    uniq_and = CompVar('uniq_and')
    uniq_and_reg_l = CompVar('uniq_and_reg_l')
    uniq_and_reg_r = CompVar('uniq_and_reg_r')
    uniq_adder = CompVar('uniq_adders')
    
    uniq_idx = CompVar('uniq_idx')
    uniq_idx_adder = CompVar('uniq_idx_adder')
    

    # Initialize the cells
    ptc_size = max_paths + 1
    path_id_width = max_paths.bit_length()
    depth_width = max_steps.bit_length()
    steps_width = (max_steps - 1).bit_length()
    
    cells = [
        # Memory cells for path_ids and paths_on_node
        Cell(path_ids, stdlib.mem_d1(path_id_width, max_steps, steps_width), is_external=True),
        Cell(paths_to_consider, stdlib.mem_d1(1, ptc_size, path_id_width), is_external=True),
        Cell(paths_on_node, stdlib.mem_d1(1, ptc_size, path_id_width)),
        Cell(depth_output, stdlib.mem_d1(depth_width, 1, 1), is_external=True),

        # Idx cells
        Cell(idx, stdlib.register(steps_width)),
        Cell(idx_adder, stdlib.op("add", steps_width, signed=True)),
        Cell(idx_neq, stdlib.op("neq", steps_width, signed=False)),

        # Registers
        Cell(path_id_reg, stdlib.register(path_id_width)),
        Cell(uniq_and_reg_l, stdlib.register(1)),
        Cell(uniq_and_reg_r, stdlib.register(1)),
        
        # Cells for node depth computation
        Cell(depth, stdlib.register(depth_width)),
        Cell(depth_temp, stdlib.register(1)),
        Cell(depth_adder, stdlib.op("add", depth_width, signed=False)),
        
        # Cells for uniq node depth computation
        Cell(uniq, stdlib.register(depth_width)),
        Cell(uniq_and, stdlib.op("and", 1, signed=False)),
        Cell(uniq_adder, stdlib.op("add", depth_width, signed=False))
    ]

    
    # Initialize the wires
    wires = [
        Group(
            CompVar("init_idx"),
            [
                Connect(ConstantPort(steps_width, 0), CompPort(idx, "in")),
                Connect(ConstantPort(1, 1), CompPort(idx, "write_en")),
                Connect(CompPort(idx, "done"), HolePort(CompVar("init_idx"), "done"))
            ]
        ),
        
        Group(
            CompVar("load_path_id"),
            [
                Connect(CompPort(idx, "out"), CompPort(path_ids, "addr0")),
                Connect(CompPort(path_ids, "read_data"), CompPort(path_id_reg, "in")),
                Connect(ConstantPort(1,1), CompPort(path_id_reg, "write_en")),
                Connect(CompPort(path_id_reg, "done"), HolePort(CompVar("load_path_id"), "done")),
            ]
        ),
            
        Group(
            CompVar("inc_idx"),
            [
                Connect(CompPort(idx, "out"), CompPort(idx_adder, "left")),
                Connect(ConstantPort(steps_width, 1), CompPort(idx_adder, "right")),
                Connect(CompPort(idx_adder, "out"), CompPort(idx, "in")),
                Connect(ConstantPort(1, 1), CompPort(idx, "write_en")),
                Connect(CompPort(idx, "done"), HolePort(CompVar("inc_idx"), "done"))
            ]
        ),

        CombGroup(
            CompVar("compare_idx"),
            [
                Connect(CompPort(idx, "out"), CompPort(idx_neq, "left")),
                Connect(ConstantPort(steps_width, max_steps - 1), CompPort(idx_neq, "right"))
            ]
        ),

        Group(
            CompVar("load_consider_path"),
            [
                Connect(CompPort(path_id_reg, "out"), CompPort(paths_to_consider, "addr0")),
                Connect(CompPort(paths_to_consider, "read_data"), CompPort(depth_temp, "in")),
                Connect(ConstantPort(1, 1), CompPort(depth_temp, "write_en")),
                Connect(CompPort(depth_temp, "done"), HolePort(CompVar("load_consider_path"), "done"))
            ]
        ),
            
        Group(
            CompVar("inc_depth"),
            [
                #If path_id is not 0, add 1 to depth
                Connect(CompPort(depth, "out"), CompPort(depth_adder, "left")),
                Connect(ConstantPort(depth_width, 1), CompPort(depth_adder, "right")),
                Connect(CompPort(depth_adder, "out"), CompPort(depth, "in")),
                Connect(ConstantPort(1, 1), CompPort(depth, "write_en")),
                Connect(CompPort(depth, "done"), HolePort(CompVar("inc_depth"), "done"))
            ]
        ),

        Group(
            CompVar("no_inc_depth"),
            [
                Connect(ConstantPort(1, 0), CompPort(depth_temp, "in")),
                Connect(ConstantPort(1, 1), CompPort(depth_temp, "write_en")),
                Connect(CompPort(depth_temp, "done"), HolePort(CompVar("no_inc_depth"), "done"))
            ]
        ),

        Group(
            CompVar('write_depth'),
            [
                Connect(ConstantPort(1, 0), CompPort(depth_output, "addr0")),
                Connect(CompPort(depth, 'out'), CompPort(depth_output, 'write_data')),
                Connect(ConstantPort(1, 1), CompPort(depth_output, 'write_en')),
                Connect(CompPort(depth_output, 'done'), HolePort(CompVar('write_depth'), 'done'))
            ]
        )
    ]
    

    # Define control flow
    controls = SeqComp([
        Enable("init_idx"),
        While(
            CompPort(idx_neq, "out"),
            CompVar("compare_idx"),
            SeqComp([
                Enable("load_path_id"),
                ParComp([
                    Enable('inc_idx'),
                    SeqComp([
                        Enable("load_consider_path"),
                        If(
                            CompPort(depth_temp, "out"),
                            None,
                            Enable("inc_depth"),
                            Enable("no_inc_depth")  
                        )
                    ])
                ])
            ])
        ),
        Enable("load_path_id"),
        Enable("load_consider_path"),
        If(
            CompPort(depth_temp, "out"),
            None,
            Enable("inc_depth"),
            Enable("no_inc_depth")  
        ),
        Enable('write_depth')
    ])
        
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
        # 
                        

    main_component = Component(
        name="main",
        inputs=[],
        outputs=[],
        structs=cells + wires,
        controls=controls,
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
    parser.add_argument('-n', '--max-nodes', type=int, default=MAX_NODES, help='Specify the maximum number of nodes that the hardware can support.')
    parser.add_argument('-e', '--max-steps', type=int, default=MAX_STEPS, help='Specify the maximum number of steps per node that the hardware can support.')
    parser.add_argument('-p', '--max-paths', type=int, default=MAX_PATHS, help='Specify the maximum number of paths that the hardware can support.')
    parser.add_argument('-o', '--out', help='Specify the output file. If not specified, will dump to stdout.')

    args = parser.parse_args()


    # Generate calyx code
    program = node_depth(args.max_nodes, args.max_steps, args.max_paths)

    # Emit the code
    if (args.out):
        with open(args.out, 'w') as out_file:
            out_file.write(program.doc())
    else:
        program.emit()            
