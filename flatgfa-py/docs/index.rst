FlatGFA: An Efficient Pangenome Representation
==============================================

.. py:module:: flatgfa

`FlatGFA`_ is an efficient on-disk and in-memory way to represent
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
:func:`load` and :meth:`FlatGFA.write_flatgfa`.

The library is on `PyPI`_, so you can get started by typing
``pip install flatgfa``.

.. _GFA: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md
.. _node depth: https://odgi.readthedocs.io/en/latest/rst/commands/odgi_depth.html
.. _FlatGFA: https://github.com/cucapra/pollen/tree/main/flatgfa
.. _PyPI: https://pypi.org/project/flatgfa/

API Reference
-------------

Loading Data
''''''''''''

The FlatGFA library can both read and write files in two formats: the standard
`GFA`_ text format, and its own efficient binary representation (called
"FlatGFA" files). Each of these functions below return a :class:`FlatGFA`
object. Parsing GFA text can take some time, but loading a binary FlatGFA file
should be very fast.

.. autofunction:: parse

.. autofunction:: parse_bytes

.. autofunction:: load

GFA Graphs
''''''''''

The :class:`FlatGFA` class provides the entry point to access the data either
loaded from a FlatGFA binary file or parsed from a GFA text file. Most
importantly, you can iterate over the :class:`Segment`, :class:`Path`, and
:class:`Link` objects that it contains. The :class:`FlatGFA` class exposes
:class:`list`-like containers for each of these types::

    for seg in graph.segments:
        print(seg.name)
    print(graph.segments[0].sequence())

These containers support both iteration (like the ``for`` above) and random
access (like ``graph.segments[0]`` above).

You can also write graphs out to disk using :meth:`FlatGFA.write_gfa`
(producing a standard GFA text file) and :meth:`FlatGFA.write_flatgfa` (our
binary format). If you just want a GFA string, use `str(graph)`.

.. autoclass:: FlatGFA
   :members:

The GFA Data Model
''''''''''''''''''

These classes represent the core data model for GFA graphs:
:class:`Segment` for vertices in the graph,
:class:`Path` for walks through the graph,
and :class:`Link` for edges in the graph.
Internally, all of these objects only contain references to the underlying
data stored in a :class:`FlatGFA`, so they are very small, but accessing any
of the associated data (such as the nucleotide sequence for a segment) require
further lookups.

The :class:`Handle` class is a segment--orientation pair: both paths and links
traverse these handles.

To get a GFA text representation of any of these objects, use ``str(obj)``.
All these objects are equatable (so you can compare them with ``==``) and
hashable (so you can store them in dicts and sets). This reflects equality on
the underlying references to the data store, so two objects are equal if they
refer to the same index in the same :class:`FlatGFA`.

.. autoclass:: Segment
   :members:

.. autoclass:: Path
   :members:

.. autoclass:: Link
   :members:

.. autoclass:: Handle
   :members:

.. toctree::
   :maxdepth: 2
   :caption: Contents:

Iteration
'''''''''

The FlatGFA library exposes special container classes to access the
:class:`Segment`, :class:`Path`, and :class:`Link` objects that make up a GFA
graph. These classes are meant to behave sort of like Python :class:`list`
objects while supporting efficient iteration over FlatGFA's internal
representation.

All of these container objects support subscripting (like
``graph.segments[i]`` where ``i`` is an integer index) and iteration.

.. autoclass:: SegmentList
   :members:

.. autoclass:: PathList
   :members:

.. autoclass:: LinkList
   :members:

.. autoclass:: StepList
   :members:
