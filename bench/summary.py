import csv
import sys
from collections import defaultdict


def summary():
    reader = csv.DictReader(sys.stdin)
    by_graph = defaultdict(dict)
    for row in reader:
        by_graph[row['graph']][row['cmd']] = row

    for graph, cmds in by_graph.items():
        print(graph)
        for cmd, row in cmds.items():
            mean = float(row["mean"])
            stddev = float(row["stddev"])

            if mean < 0.2:
                mean *= 1000
                stddev *= 1000
                unit = 'ms'
            else:
                unit = 's'

            print(f'  {cmd}: {mean:.1f} ± {stddev:.1f} {unit}')


if __name__ == "__main__":
    summary()
