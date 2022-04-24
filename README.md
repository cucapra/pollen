Pangenome Graph Queries in Calyx
================================

This is a nascent project to build a DSL-to-hardware compiler using [Calyx][] to implement pangenomic graph queries in the vein of [odgi][].
It is very much a work in progress.

Getting Started
---------------

You will need to fetch some useful inputs by typing `make fetch`.
This downloads some simple [GFA][] data files.

Then, to compute the node depth for these graphs, type one of these commands:

    $ python3 depth.py k.gfa
    $ python3 depth.py DRB1-3123.gfa

The first works on an extremely small example, and the second is a tiny bit bigger.

If you want to build odgi graph files from the GFA files, try something like `make k.og`.

[calyx]: https://calyxir.org
[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8006571/#FN8
