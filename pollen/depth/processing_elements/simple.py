import argparse
import json
import sys
import warnings

from calyx.py_ast import *
import odgi


class DepthPE():

    max_steps: int
    max_paths: int
    ptc_size: int
    path_id_width: int
    depth_width: int
    pids_addr_width: int
    uniq_width: int

    def __init__(self, max_steps, max_paths):
        self.max_steps = max_steps
        self.max_paths = max_paths
        
        self.ptc_size = max_paths + 1
        # number of bits to represent each value
        self.path_id_width = max_paths.bit_length()
        self.depth_width = max_steps.bit_length()
        self.pids_addr_width = max(1, (max_steps - 1).bit_length())
        self.uniq_width = path_id_width
        
    def node_depth_pe(self):
        """
        Return the component representation of a node depth processing element
        """

        stdlib = Stdlib()
        max_steps = self.max_steps
        max_paths = self.max_paths
        ptc_size = self.ptc_size
        path_id_width = self.path_id_width
        depth_width = self.depth_width
        pids_addr_width = self.pids_addr_width
        uniq_width = self.uniq_width

        # Variable identifiers

        # Input and Output ports
        depth_in = CompVar('depth_in')
        depth_write_en = CompVar('depth_write_en')
        depth_out = CompVar('depth_out')
        depth_done = CompVar('depth_done')
        uniq_in = CompVar('uniq_in')
        uniq_write_en = CompVar('uniq_write_en')
        uniq_out = CompVar('uniq_out')
        uniq_done = CompVar('uniq_done')
        # path_id for each step on the node
        pids_addr0 = CompVar('pids_addr0')    
        pids_read_data = CompVar('pids_read_data')
        # paths to consider
        ptc_addr0 = CompVar('ptc_addr0')
        ptc_read_data = CompVar('ptc_read_data')
        # paths on node
        pon_addr0 = CompVar('pon_addr0')
        pon_write_data = CompVar('pon_write_data')
        pon_write_en = CompVar('pon_write_en')
        pon_read_data = CompVar('pon_read_data')
        pon_done = CompVar('pon_done')

        path_id_reg = CompVar('path_id_reg')
        idx = CompVar('idx')
        idx_adder = CompVar('idx_adder')
        idx_neq = CompVar('idx_neq')

        depth_temp = CompVar('depth_temp')
        depth_pad = CompVar('depth_pad')
        depth_adder = CompVar('depth_adder')

        uniq_and = CompVar('uniq_and')
        uniq_and_reg_l = CompVar('uniq_and_reg_l')
        uniq_and_reg_r = CompVar('uniq_and_reg_r')
        uniq_pad = CompVar('uniq_pad')
        uniq_adder = CompVar('uniq_adder')

        uniq_idx = CompVar('uniq_idx')
        uniq_idx_neq = CompVar('uniq_idx_neq')
        uniq_idx_adder = CompVar('uniq_idx_adder')


        # Initialize the cells
        cells = [
            # Idx cells
            Cell(idx, stdlib.register(path_ids_addr_width)),
            Cell(idx_adder, stdlib.op("add", pids_addr_width, signed=False)),
            Cell(idx_neq, stdlib.op("neq", pids_addr_width, signed=False)),

            # Registers
            Cell(path_id_reg, stdlib.register(path_id_width)),
            Cell(uniq_and_reg_l, stdlib.register(1)),
            Cell(uniq_and_reg_r, stdlib.register(1)),

            # Cells for node depth computation
            Cell(depth_temp, stdlib.register(1)),
            Cell(depth_pad, stdlib.pad(1, depth_width)),
            Cell(depth_adder, stdlib.op("add", depth_width, signed=False)),

            # Cells for uniq node depth computation
            Cell(uniq_and, stdlib.op("and", 1, signed=False)),
            Cell(uniq_pad, stdlib.pad(1, uniq_width)),
            Cell(uniq_adder, stdlib.op("add", uniq_width, signed=False)),

            Cell(uniq_idx, stdlib.register(path_ids_width)),
            Cell(uniq_idx_neq, stdlib.op("neq", path_ids_width, signed=False)),
            Cell(
                uniq_idx_adder,
                stdlib.op("sub", path_ids_width, signed=False)
            ),
    ]


        # Initialize the wires
        wires = [
            Group(
                CompVar("init_idx"),
                [
                    Connect(
                        CompPort(idx, "in"),
                        ConstantPort(pids_addr_width, 0)
                    ),
                    Connect(CompPort(idx, "write_en"), ConstantPort(1, 1)),
                    Connect(
                        HolePort(CompVar("init_idx"), "done"),
                        CompPort(idx, "done")
                    )
                ]
            ),
            Group(
                CompVar("load_path_id"),
                [
                    Connect(ThisPort(pids_addr0), CompPort(idx, "out")),
                    Connect(
                        CompPort(path_id_reg, "in"),
                        ThisPort(pids_read_data)
                    ),
                    Connect(
                        CompPort(path_id_reg, "write_en"),
                        ConstantPort(1,1)
                    ),
                    Connect(
                        HolePort(CompVar("load_path_id"), "done"),
                        CompPort(path_id_reg, "done")
                    ),
                ]
            ),
            Group(
                CompVar("inc_idx"),
                [
                    Connect(CompPort(idx_adder, "left"), CompPort(idx, "out")),
                    Connect(
                        CompPort(idx_adder, "right"),
                        ConstantPort(pids_addr_width, 1)),
                    Connect(CompPort(idx, "in"), CompPort(idx_adder, "out")),
                    Connect(CompPort(idx, "write_en"), ConstantPort(1, 1)),
                    Connect(
                        HolePort(CompVar("inc_idx"), "done"),
                        CompPort(idx, "done")
                    )
                ]
            ),
            CombGroup(
                CompVar("compare_idx"),
                [
                    Connect(CompPort(idx_neq, "left"), CompPort(idx, "out")),
                    Connect(
                        CompPort(idx_neq, "right"),
                        ConstantPort(pids_addr_width, max_steps - 1)
                    )
                ]
            ),

            # Node depth wires
            Group(
                CompVar("load_consider_path"),
                [
                    Connect(
                        ThisPort(ptc_addr0),
                        CompPort(path_id_reg, "out")
                    ),
                    Connect(
                        CompPort(depth_temp, "in"),
                        ThisPort(ptc_read_data)
                    ),
                    Connect(
                        CompPort(depth_temp, "write_en"),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        HolePort(CompVar("load_consider_path"), "done"),
                        CompPort(depth_temp, "done")
                    )
                ]
            ),
            Group(
                CompVar("inc_depth"),
                [
                    #If path_id is not 0, add 1 to depth
                    Connect(CompPort(depth_adder, "left"), ThisPort(depth_out)),
                    Connect(
                        CompPort(depth_pad, 'in'),
                        CompPort(depth_temp, 'out')
                    ),
                    Connect(
                        CompPort(depth_adder, "right"),
                        CompPort(depth_pad, 'out')
                    ),
                    Connect(ThisPort(depth_in), CompPort(depth_adder, "out")),
                    Connect(ThisPort(depth_write_en), ConstantPort(1, 1)),
                    Connect(
                        HolePort(CompVar("inc_depth"), "done"),
                        ThisPort(depth_done)
                    )
                ]
            ),

            # Uniq node depth wires
            Group(
                CompVar('init_uniq_idx'),
                [
                    Connect(
                        CompPort(uniq_idx, 'in'),
                        ConstantPort(uniq_width, max_paths)
                    ),
                    Connect(CompPort(uniq_idx, 'write_en'), ConstantPort(1, 1)),
                    Connect(
                        HolePort(CompVar('init_uniq_idx'), 'done'),
                        CompPort(uniq_idx, 'done')
                    )
                ]
            ),
            CombGroup(
                CompVar('compare_uniq_idx'),
                [
                    Connect(
                        CompPort(uniq_idx_neq, 'left'),
                        CompPort(uniq_idx, 'out')
                    ),
                    Connect(
                        CompPort(uniq_idx_neq, 'right'),
                        ConstantPort(path_ids_width, 0)
                    )
                ]
            ),
            Group(
                CompVar('dec_uniq_idx'),
                [
                    Connect(
                        CompPort(uniq_idx_adder, 'left'),
                        CompPort(uniq_idx, 'out')
                    ),
                    Connect(
                        CompPort(uniq_idx_adder, 'right'),
                        ConstantPort(path_ids_width, 1)
                    ),
                    Connect(
                        CompPort(uniq_idx, 'in'),
                        CompPort(uniq_idx_adder, 'out')
                    ),
                    Connect(
                        CompPort(uniq_idx, 'write_en'),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        HolePort(CompVar('dec_uniq_idx'), 'done'),
                        CompPort(uniq_idx, 'done')
                    )
                ]
            ),

            Group(
                CompVar('update_pon'), # update paths_on_node
                [
                    Connect(
                        ThisPort(pon_addr0),
                        CompPort(path_id_reg, "out")
                    ),
                    Connect(
                        ThisPort(pon_write_data),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        ThisPort(pon_write_en),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        HolePort(CompVar("update_pon"), "done"),
                        ThisPort(pon_done)
                    )
                ]
            ),
            Group(
                CompVar("load_and_l"),
                [
                    Connect(
                        ThisPort(pon_addr0),
                        CompPort(uniq_idx, "out")
                    ),
                    Connect(
                        CompPort(uniq_and_reg_l, "in"),
                        ThisPort(pon_read_data)
                    ),
                    Connect(
                        CompPort(uniq_and_reg_l, "write_en"),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        HolePort(CompVar("load_and_l"), "done"),
                        CompPort(uniq_and_reg_l, "done")
                    )
                ]
            ),
            Group(
                CompVar("load_and_r"),
                [
                    Connect(
                        ThisPort(ptc_addr0),
                        CompPort(uniq_idx, "out")
                    ),
                    Connect(
                        CompPort(uniq_and_reg_r, "in"),
                        ThisPort(ptc_read_data)
                    ),
                    Connect(
                        CompPort(uniq_and_reg_r, "write_en"),
                        ConstantPort(1, 1)
                    ),
                    Connect(
                        HolePort(CompVar("load_and_r"), "done"),
                        CompPort(uniq_and_reg_r, "done")
                    )   
                ]
            ),
            Group(
                CompVar("inc_uniq"),
                [
                    Connect(
                        CompPort(uniq_and, "left"),
                        CompPort(uniq_and_reg_l, "out")
                    ),
                    Connect(
                        CompPort(uniq_and, "right"),
                        CompPort(uniq_and_reg_r, "out")
                    ),
                    Connect(CompPort(uniq_adder, "left"), ThisPort(uniq_out)),
                    Connect(
                        CompPort(uniq_pad, 'in'),
                        CompPort(uniq_and, 'out')
                    ),
                    Connect(
                        CompPort(uniq_adder, "right"),
                        CompPort(uniq_pad, 'out')
                    ),
                    Connect(ThisPort(uniq_in), CompPort(uniq_adder, "out")),
                    Connect(ThisPort(uniq_write_en), ConstantPort(1, 1)),
                    Connect(
                        HolePort(CompVar("inc_uniq"), "done"),
                        ThisPort(uniq_done)
                    )
                ]
            ),
        ]

        # Define control flow
        controls = SeqComp([
            Enable("init_idx"),
            ParComp([
                Enable('init_uniq_idx'),
                While(
                    CompPort(idx_neq, "out"),
                    CompVar("compare_idx"),
                    SeqComp([
                        Enable("load_path_id"),
                        ParComp([
                            Enable('inc_idx'),
                            # Depth computation
                            SeqComp([
                                Enable("load_consider_path"),
                                Enable("inc_depth"),
                            ]),
                            # Uniq computation
                            Enable('update_pon')
                        ])
                    ])
                )
            ]),
            Enable("load_path_id"),
            Enable("load_consider_path"),
            Enable("inc_depth"),
            Enable('update_pon'),
            While(
                CompPort(uniq_idx_neq, 'out'),
                CompVar('compare_uniq_idx'),
                SeqComp([
                    ParComp([Enable('load_and_l'), Enable('load_and_r')]),
                    Enable('inc_uniq'),
                    Enable('dec_uniq_idx')    
                ])    
            ),
        ])

        pe_component = Component(
            name="node_depth_pe",
            inputs=[
                PortDef(depth_out, depth_width), PortDef(depth_done, 1),
                PortDef(uniq_out, uniq_width), PortDef(uniq_done, 1),
                PortDef(pids_read_data, path_ids_width),
                PortDef(ptc_read_data, 1),
                PortDef(pon_read_data, 1), PortDef(pon_done, 1)
            ],
            outputs=[
                PortDef(depth_in, depth_width), PortDef(depth_write_en, 1),
                PortDef(uniq_in, uniq_width), PortDef(uniq_write_en, 1),
                PortDef(pids_addr0, pids_addr_width),
                PortDef(ptc_addr0, path_ids_width),
                PortDef(pon_addr0, path_ids_width), PortDef(pon_write_data, 1),
                PortDef(pon_write_en, 1)
            ],
            structs=cells + wires,
            controls=controls,
        )

        return pe_component


    def parse_node(self, graph, node_id, path_name_to_id):
        '''
        Generate input data containing the path ids for each step on node_h, ex.
        {
          "path_ids": {
            "data": [0, 1, 1, 2],
            "format": {
              "numeric_type": "bitnum",
              "is_signed": False,
              "width": 2
            }
          },
          "paths_on_node": {
            "data": [0, 0],
            "format": {
              "numeric_type": "bitnum", 
              "is_signed": False,                                            
              "width": 1
            }
          }
        }
        '''

        # If node_id is out of bounds, return the default value
        if node_id > max_nodes:
            node_data = {
                "path_ids" : {
                    "data": [0] * max_steps,
                    "format": {
                        "numeric_type": "bitnum",
                        "is_signed": False,
                        "width": max_paths.bit_length()
                    }
                },
                "paths_on_node": {
                    "data": [0] * (max_paths + 1)
                    "format": {
                        "numeric_type": "bitnum",
                        "is_signed": False,
                        "width": 1
                    }
                },
            }        
            return node_data
            
        # Check that the number of steps on the node does not exceed max_steps
        if graph.get_step_count(node_h) > max_steps:
            raise Exception(f'The number of paths in the graph exceeds the \
                              maximum number of paths the hardware can process.\
                              {graph.get_step_count(node_h)} > {max_steps}. \
                              Hint: try setting the maximum number of steps \
                              manually using the -e flag.'
                            )

        # Get a list of path ids for each step on node_h.    
        path_ids = []

        def parse_step(step_h):
            path_h = graph.get_path(step_h)
            path_id = path_name_to_id[graph.get_path_name(path_h)]
            path_ids.append(path_id)

        graph.for_each_step_on_handle(node_h, parse_step)

        # Pad path_ids with 0s
        path_ids = path_ids + [0] * (max_steps - len(path_ids))

        paths_to_consider = parse_paths_file(subset_paths, path_name_to_id, max_paths)

        # format the data
        node_data = {
            "path_ids" : {
                "data": path_ids,
                "format": {
                    "numeric_type": "bitnum",
                    "is_signed": False,
                    "width": max_paths.bit_length()
                }
            }
        }

        return node_data


    def parse_global_vars(self, graph, subset_paths, path_name_to_id):
        data = {}
        default_pon = [0] * (max_paths + 1)
        paths_to_consider = self.parse_paths_file(subset_paths, path_name_to_id)
        for i in range(1, self.max_nodes + 1):
            data[f'paths_on_node{i}'] = {
                "data": default_pon,
                "format": {
                    "numeric_type": "bitnum",
                    "is_signed": False,
                    "width": 1
                }
            }
            data[f'paths_to_consider{i}'] = {
                "data": paths_to_consider,
                "format": {
                    "numeric_type": "bitnum",
                    "is_signed": False,
                    "width": 1
                }
            }

        return data

    
    def parse_paths_file(self, filename, path_name_to_id):
        '''
        Return paths_to_consider, a list of length max_paths, where
        paths_to_consider[i] is 1 if i is a path id and we include path i in our
        calculations of node depth
        '''

        if filename is None: # Return the default value     
            paths_to_consider = [1]*(self.ptc_size)
            paths_to_consider[0] = 0
            return paths_to_consider

        with open(filename, 'r') as paths_file:
            text = paths_file.read()
            paths = text.splitlines()

        paths_to_consider = [0] * (self.ptc_size)

        for path_name in paths:
            path_id = path_name_to_id[path_name]
            paths_to_consider[path_id] = 1
            
        return paths_to_consider
