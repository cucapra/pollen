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

mygfa is `on PyPI`_, so you can install it with ``pip install mygfa``.

.. _GFA: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md
.. _on PyPI: https://pypi.org/project/mygfa/

API Reference
-------------

.. automodule:: mygfa

    .. autoclass:: Graph
       :members:

    .. autoclass:: Segment
       :members:

    .. autoclass:: Link
       :members:

    .. autoclass:: Path
       :members:

    .. autoclass:: Handle
       :members:

    .. autoclass:: Strand
       :members:

    .. autoclass:: Alignment
       :members:

    .. autoclass:: AlignOp
       :members:

.. toctree::
   :maxdepth: 2
   :caption: Contents:
