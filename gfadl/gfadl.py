from mygfa import mygfa
import sys
import os
import csv


def gfadl(outdir):
    graph = mygfa.Graph.parse(sys.stdin)

    os.makedirs(outdir, exist_ok=True)

    # Output segment names.
    with open(os.path.join(outdir, 'segments.csv'), 'w') as f:
        for seg_name in graph.segments:
            print(seg_name, file=f)

    # Output path names.
    with open(os.path.join(outdir, 'paths.csv'), 'w') as f:
        for path_name in graph.paths:
            print(path_name, file=f)

    # Output the main relation.
    with open(os.path.join(outdir, 'gfa.csv'), 'w') as f:
        writer = csv.writer(f)
        for path in graph.paths.values():
            for i, step in enumerate(path.segments):
                writer.writerow([
                    path.name,
                    i,
                    "forward" if step.ori else backward,
                    step.name,
                ])


if __name__ == "__main__":
    gfadl(sys.argv[1])
