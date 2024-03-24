mygfa: A Basic GFA Data Model
=============================

This library parses, represents, and emits pangenomic variation graphs in the
`GFA`_ format. Basic use looks like this::

    import mygfa
    import sys
    graph = mygfa.Graph.parse(sys.stdin)
    seg_depths = {name: 0 for name in graph.segments}
    for path in graph.paths.values():
        for step in path.segments:
            seg_depths[step.name] += 1

The :class:`mygfa.Graph` class represents an entire GFA file.
You can work down the object hierarchy from there to see everything that the
file contains.

.. _GFA: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md

API Reference
-------------

.. autoclass:: mygfa.Graph
   :members:

.. autoclass:: mygfa.Segment
   :members:

.. autoclass:: mygfa.Link
   :members:

.. autoclass:: mygfa.Path
   :members:

.. autoclass:: mygfa.Handle
   :members:

.. autoclass:: mygfa.Strand
   :members:

.. autoclass:: mygfa.Alignment
   :members:

.. toctree::
   :maxdepth: 2
   :caption: Contents:
