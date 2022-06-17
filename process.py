import sys

'''
Reads a graph depth table from the commandline and removes the depth.uniq column
'''
node_depths = sys.stdin.readlines()
for row in node_depths:
    print(row[:row.rfind('\t')])
