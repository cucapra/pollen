import json
import sys
import parse_gfa
import os

def depth2(g):
    '''
    Calculates the depth based on the flat representation of a .gfa file
    '''
    p_offsets = g['path offsets']
    paths = g['paths']
    s_names = g['segment names']

    counts = [0] * len(s_names) # list to keep track of the count of each segment

    # finds the count for each node
    for i in range(len(p_offsets) - 1):
        for j in range(p_offsets[i], p_offsets[i + 1]):
            index = paths[j]
            counts[index] += 1

    # gets the name for each segment and gets its respective count
    depth_map = {} # maps segment names ot their counts
    for i in range(len(counts)):
        depth_map[s_names[i]] = counts[i]

    print(depth_map)

def depth2_nojson(filename):
    '''
    Parses the given .gfa and calculates the depth of each node.
    '''
    depth2(parse_gfa.parse_file(filename))

def depth2_withjson(filename):
    '''
    Parses the given .json and calculates the depth of each node.
    '''
    with open(filename) as f:
        g = json.load(f)
    depth2(g)

if __name__ == '__main__':
    filename = sys.argv[1]
    ext = os.path.splitext(filename)[1]
    if ext == '.json':
        depth2_withjson(filename)
    elif ext == '.gfa':
        depth2_nojson(filename)
    else:
        raise Exception("Invalid file format.")