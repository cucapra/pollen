from calyx.py_ast import *

MAX_NODES=16
MAX_STEPS=16
MAX_PATHS=15

def node_depth(max_nodes=MAX_NODES, max_steps=MAX_STEPS, max_paths=MAX_PATHS):
    # Variable identifiers
    null_path_id = CompVar('null_path_id') # Default when no path present
    one = CompVar('one') # Value to be added to depth and uniq depth

    paths_to_consider = CompVar('paths_to_consider')
    path_id_regs = []
    
    path_ids = [] # path_ids[i] represents a list of path ids corresponding to each step on node i
    paths_on_node = []
    
    node_depth = []
    depth_neq = []
    depth_adder = []
    
    node_uniq = []
    uniq_and = []
    uniq_and_reg_l = []
    uniq_and_reg_r = []
    uniq_adders = []
    
    for i in range(1, max_nodes + 1):
        path_ids.append(CompVar(f'path_ids{i}'))
        paths_on_node.append(CompVar(f'paths_on_node{i}'))
        path_id_reg.append(CompVar('path_id_reg{i}'))
    
        node_depth.append(CompVar(f'depth{i}'))
        depth_neq.append(CompVar(f'neq{i}'))
        depth_adder.append(CompVar(f'addd{i}'))
    
        node_uniq.append(CompVar(f'uniq{i}'))
        uniq_and.append(CompVar(f'and{i}'))
        uniq_and_reg_l.append(CompVar('uniq_and_reg_l{i}'))
        uniq_and_reg_r.append(CompVar('uniq_and_reg_r{i}'))
        uniq_adder.append(CompVar(f'addu{i}'))

    # Initialize the cells
    ptc_size = max_paths + 1
    path_id_width = max_paths.bit_width()
    depth_size = max_paths.bit_width()
    
    cells = [
        Cells(null_path_id, stdlib.constant(path_id_size, 0)),
        Cells(one, stdlib.constant(depth_size, 1)),
        Cell(paths_to_consider, stdlib.mem_d1(1, ptc_size, path_id_width))
    ]
    
    for i in range(max_nodes):
        # Memory cells for path_ids and paths_on_node
        cells.append(
            Cell(path_ids[i], stdlib.mem_d1(path_id_width, max_steps, max_steps.bit_width()))
        )
        cells.append(
            Cell(paths_on_node[i], stdlib.mem_d1(1, ptc_size, path_id_size))
        )

        # Registers
        cells.append(
            Cell(path_id_regs[i], stdlib.register(path_id_size))
        )
        cells.append(
            Cell(uniq_and_reg_l[i], stdlib.register(1))
        )
        cells.append(
            Cell(uniq_and_reg_r[i], stdlib.register(1))
        )
        
        # Cells for node depth computation
        cells.append(
            Cell(node_depth[i], stdlib.register(depth_size))
        )
        cells.append(
            Cell(depth_neq[i], stdlib.op("neq", path_id_size, signed=False))
        )
        cells.append(
            Cell(depth_adder[i], stdlib.op("add", depth_size, signed=False))
        )
        
        # Cells for uniq node depth computation
        cells.append(
            Cell(node_uniq[i], stdlib.register(depth_size))
        )
        cells.append(
            Cell(uniq_and[i], stdlib.op("and", 1, signed=False))
        )
        cells.append(
            Cell(uniq_adder[i], stdlib.op("add", depth_size, signed=False))
        )

    # Initialize the wires

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

    # Emit the code.
    program.emit()
            
if __name__ == '__main__':

    # Parse commandline input


    # Generate calyx code
    
