import json

with open('k.json') as f:
    g = json.load(f)

p_offsets = g['path offsets']
paths = g['paths']
s_names = g['segment names']
s_map = {}
s_counter = 0

depth_map = {}

for s in s_names:
    s_map[s_counter] = s

for i in range(len(p_offsets) - 1):
    for j in range(p_offsets[i], p_offsets[i + 1]):
        name = s_names[paths[j]]
        if name in depth_map:
            depth_map[name] += 1
        else:
            depth_map[name] = 1

print(depth_map)