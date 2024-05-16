FlatGFA: An Efficient Pangenome Representation
==============================================

`FlatGFA`_ is an efficient on-disk and in-memory way to represent emits
pangenomic variation graphs. It can losslessly represent `GFA`_ files.
Here's a quick example::

    import flatgfa
    from collections import Counter

    graph = flatgfa.parse("something.gfa")
    depths = Counter()
    for path in graph.paths:
        for step in path:
            depths[step.segment.id] += 1

    print('#node.id\tdepth')
    for seg in graph.segments:
        print('{}\t{}'.format(seg.name, depths[seg.id]))

This example computes the `node depth`_ for every segment in a graph.
It starts by parsing a GFA text file, but FlatGFA also has its own efficient
binary representation---you can read and write this format with
:func:`flatgfa.load` and :meth:`flatgfa.FlatGFA.write_flatgfa`.

.. _GFA: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md
.. _node depth: https://odgi.readthedocs.io/en/latest/rst/commands/odgi_depth.html
.. _FlatGFA: https://github.com/cucapra/pollen/tree/main/flatgfa

API Reference
-------------

.. automodule:: flatgfa

    .. autofunction:: parse

    .. autofunction:: load

    .. autoclass:: FlatGFA
       :members:

    .. autoclass:: Segment
       :members:

    .. autoclass:: Link
       :members:

    .. autoclass:: Path
       :members:

    .. autoclass:: Handle
       :members:

.. toctree::
   :maxdepth: 2
   :caption: Contents:
