import json
import sys


def format_graph_depth_table(node_depths):
    '''
    Reads a graph depth table from the commandline and removes the depth.uniq column
    '''
    for row in node_depths:
        print(row[:row.rfind('\t')])

def format_json_data(node_depths, mem='segments0'):
    '''
    Takes a json data file (calyx output) and formats it as above
    '''
    depths = node_depths['memories'][mem]
    print('#node.id\tdepth')
    for i in range(len(depths)):
        print(f'{i+1}\t{depths[i]}')
    
if __name__ == '__main__':
    '''
    Take a commandline arg, gdt or json, to specify which file to convert
    '''
    format = sys.argv[1]
    if format == "gdt":
        format_graph_depth_table(sys.stdin.readlines())
    elif format == "json":
        data = json.load(sys.stdin)
        format_json_data(data)
