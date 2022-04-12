import json

with open('k.json') as f:
    g = json.load(f)

p_offsets = g['path offsets']
paths = g['paths']
s_names = g['segment names']
s_map = {}
s_counter = 0

result_map = {}

for s in s_names:
    s_map[s_counter] = s

for i in range(len(p_offsets) - 1):
    for j in range(p_offsets[i], p_offsets[i + 1]):
        index = paths[j]
        if index in result_map:
            result_map[index] += 1
        else:
            result_map[index] = 1

depth_map = {}
for k, v in result_map.items():
    depth_map[s_names[k]] = v

print(depth_map)