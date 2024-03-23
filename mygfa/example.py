import sys
import mygfa


def print_depth(graph):
    # Count the number of times that any path passes through a segment.
    seg_depths = {name: 0 for name in graph.segments}
    for path in graph.paths.values():
        for step in path.segments:
            seg_depths[step.name] += 1

    # Print the counts.
    print("seg\tdepth")
    for name, depth in seg_depths.items():
        print(f"{name}\t{depth}")


if __name__ == "__main__":
    print_depth(mygfa.Graph.parse(sys.stdin))
