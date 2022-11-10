    def parse_step(step_h):
        path_h = graph.get_path(step_h)
        path_id = path_name_to_id[graph.get_path_name(path_h)]
        path_ids.append(path_id)
            
    graph.for_each_step_on_handle(node_h, parse_step)

    # Pad path_ids with 0s
    path_ids = path_ids + [0] * (max_steps - len(path_ids))
    
    # format the data
    node_data = {
        "path_ids" : { 
            "data": path_ids,
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
