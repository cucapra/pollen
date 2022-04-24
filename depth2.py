import json
import sys
import format_gfa
import os

def depth2(filename):
    with open(filename) as f:
        g = json.load(f)
    p_offsets = g['path offsets']
    paths = g['paths']
    s_names = g['segment names']

    counts = [0] * len(s_names)

    for i in range(len(p_offsets) - 1):
        for j in range(p_offsets[i], p_offsets[i + 1]):
            index = paths[j]
            counts[index] += 1

    depth_map = {}
    for i in range(len(counts)):
        depth_map[s_names[i]] = counts[i]

    print(depth_map)

def depth2_nojson(filename):
    format_gfa.generate_json(filename)
    json_file = os.path.splitext(filename)[0] + '.json'
    depth2(json_file)

if __name__ == '__main__':
    filename = sys.argv[1]
    ext = os.path.splitext(filename)[1]
    if ext == '.json':
        depth2(filename)
    elif ext == '.gfa':
        depth2_nojson(filename)
    else:
        raise Exception("Invalid file format.")