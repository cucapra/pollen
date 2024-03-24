mygfa
=====

This is a simple Python library for parsing, manipulating, and emitting pangenomic graphs in the [GFA][] format.
It prioritizes simplicity and clarity over performance and functionality.

As demonstrated in [`example.py`](./example.py), this is what it looks like to compute the node depth for a GFA file:

    import mygfa
    import sys
    graph = mygfa.Graph.parse(sys.stdin)
    seg_depths = {name: 0 for name in graph.segments}
    for path in graph.paths.values():
        for step in path.segments:
            seg_depths[step.name] += 1

Type `pip install mygfa` to get started.

[gfa]: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md
