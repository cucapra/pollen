import sys
import mygfa
import preprocess
from typing import List, Tuple, Dict

def sego_str(seg):
    return seg[0] + ("+" if seg[1] else "-")

def validate(graph):
    _, outs = preprocess.in_out_edges(graph)

    for path in graph.paths.values():
        length = len(path.segments)
        if length < 2:
            continue # success: done with this path
        else:
            for i in range(length-1):
                seg_from = path.segments[i]
                seg_to = path.segments[i+1]
                if seg_to not in outs[seg_from]:
                    print(f"[odgi::validate] error: the path {path.name} does not respect the graph topology: the link {sego_str(seg_from)},{sego_str(seg_to)} is missing.")

if __name__ == "__main__":
    name = sys.stdin
    graph = mygfa.Graph.parse(sys.stdin)
    validate(graph)
