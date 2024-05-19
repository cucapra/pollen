import csv
import sys
from collections import defaultdict
from statistics import harmonic_mean


def summary():
    reader = csv.DictReader(sys.stdin)
    by_graph = defaultdict(dict)
    for row in reader:
        by_graph[row["graph"]][row["cmd"]] = row

    # Guess a suitable baseline by taking the fastest time on the first graph.
    first_res = next(iter(by_graph.values()))
    min_row = min(first_res.values(), key=lambda r: r["mean"])
    baseline = min_row["cmd"]

    # Show each graph's times.
    ratios = defaultdict(list)
    for graph, cmds in by_graph.items():
        baseline_time = float(cmds[baseline]["mean"])

        print(graph)
        for cmd, row in cmds.items():
            mean = float(row["mean"])
            stddev = float(row["stddev"])
            ratio = mean / baseline_time
            ratios[cmd].append(ratio)

            if mean > 80:
                mins = int(mean / 60)
                secs = int(mean % 60)
                print(f"  {cmd}: {mins}m{secs}s ± {stddev:.1f}", end='')
            else:
                if mean < 0.2:
                    mean *= 1000
                    stddev *= 1000
                    unit = "ms"
                else:
                    unit = "s"
                print(f"  {cmd}: {mean:.1f} ± {stddev:.1f} {unit}", end='')

            print(f" ({ratio:.1f}× {baseline})")

    # Show the average across graphs.
    print("harmonic mean")
    for cmd, cmd_ratios in ratios.items():
        hmean = harmonic_mean(cmd_ratios)
        print(f"  {cmd}: {hmean:.1f}× {baseline}")


if __name__ == "__main__":
    summary()
