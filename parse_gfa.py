import json
import os

def parse_file(filename):
    '''
    Parses a .gfa into a flat file format dictionary containing the
    segment names, path names, nodes in the path, and offsets for where
    the paths begin and end.
    '''
    f = open(filename, 'r')
    lines = f.readlines()

    s_names = [] # list of segment names
    s_map = {} # maps segment names to their index
    s_counter = 0 # the index of the segment in the list

    p_names = [] # list of path names
    paths = [] # list of nodes in a path
    p_offset = 0 # the offset corresponding to the range of a path in the path array
    p_offsets = [] # list of path offsets for each path

    for line in lines:
        split = line.split()
        # processes a segment in the file
        if split[0] == 'S':
            s_name = split[1]
            s_names.append(s_name)
            s_map[s_name] = s_counter
            s_counter += 1
        
        # proceses a path in the file
        if split[0] == 'P':
            p_names.append(split[1])
            path_segments = split[2].split(',')
            p_offsets.append(p_offset)
            for segment in path_segments:
                segment = segment.rstrip('+-')
                paths.append(s_map[segment])
                p_offset += 1

    p_offsets.append(len(paths))

    result_dict = {"segment names": s_names, "paths": paths, 
    "path offsets": p_offsets, "path names": p_names}
    return result_dict

def parse_file_path_only(filename):
    '''
    Parses a .gfa and returns a list containing all nodes in every path.
    '''
    f = open(filename, 'r')
    lines = f.readlines()
    all_paths = []

    for line in lines:
        split = line.split()
        if split[0] == 'P':
            path_segments = split[2].split(',')
            for segment in path_segments:
                segment = segment.rstrip('+-')
                all_paths.append(segment)
    
    return all_paths

def generate_json(filename):
    '''
    Parses a .gfa file and writes the flat file information to a .json file
    '''
    result_dict = parse_file(filename)
    json_file = os.path.splitext(filename)[0] + '.json'
    with open(json_file, "w") as o:
        json.dump(result_dict, o)