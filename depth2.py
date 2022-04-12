import json

with open('k.json') as f:
    g = json.load(f)

p_offsets = g['path offsets']
paths = g['paths']
s_names = g['segment names']
s_counter = 0

counts = [0] * len(s_names)

for i in range(len(p_offsets) - 1):
    for j in range(p_offsets[i], p_offsets[i + 1]):
        index = paths[j]
        counts[index] += 1

depth_map = {}
for i in range(len(counts)):
    depth_map[s_names[i]] = counts[i]

print(depth_map)