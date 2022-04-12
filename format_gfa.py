import json

f = open('k.gfa', 'r')
lines = f.readlines()

s_names = []
s_map = {}
s_counter = 0

p_names = []
p_offset = 0
p_offsets = []
paths = []

for line in lines:
    split = line.split()
    if split[0] == 'S':
        s_name = split[1]
        s_names.append(s_name)
        s_map[s_name] = s_counter
        s_counter += 1
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
with open("k.json", "w") as o:
    json.dump(result_dict, o)