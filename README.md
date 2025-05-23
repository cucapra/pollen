<h1>
<p align="center">
<img src="https://github.com/cucapra/pollen/blob/main/pollen_icon_transparent.png">
</h1>

Accelerated Pangenome Graph Queries
===================================

Pollen is a nascent project to accelerate queries on pangenomic graphs.
We are designing a graph-manipulating DSL that exposes functionality that pangenomicists care about.
Our DSL will support graph queries in the vein of the [odgi][] project.
We will compile programs written in this DSL into fast query code.
Eventually, we aim to generate custom hardware accelerators for these queries via the [Calyx][] compiler.

There are several things in this repository:

* [mygfa](./mygfa), a simple Python library for parsing, processing, and emitting [GFA][] files. See [its documentation][mygfa-docs].
* [slow_odgi](./slow_odgi), a reference implementation of several GFA queries from the [odgi][] tool using `mygfa`.
* [FlatGFA](./flatgfa), an experimental fast binary format for representing and analyzing GFA files. There are also [Python bindings](./flatgfa-py) for this library; check out [their documentation][flatgfa-py-docs].
* A proof-of-concept Calyx-based [hardware accelerator generator](./pollen_py) for a single GFA query (`odgi depth`) and a data generator for this hardware.

[calyx]: https://calyxir.org
[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md
[flatgfa-py-docs]: https://cucapra.github.io/pollen/flatgfa/


`mygfa` and `slow_odgi`
-----------------------

The `mygfa` library is an extremely simple Python library for representing (and parsing and emitting) GFA files. It emphasizes clarify over efficiency. Use `pip install mygfa` to get started, and read the [API documentation][mygfa-docs] for details.

Similarly, `slow_odgi` is a set of GFA analyses based on `mygfa`; it's meant to act as a *reference implementation* of the much faster functionality in [odgi][]. Check out [the slow_odgi README](slow_odgi/) for more details.

To set up both of them from this repository, try using [uv][]:

    $ uv run slow_odgi --help

Or, alternatively, you can set up and activate the environment manually:

    $ uv sync
    $ source .venv/bin/activate
    $ slow_odgi --help

[uv]: https://github.com/astral-sh/uv
[mygfa-docs]: http://cucapra.github.io/pollen/mygfa/


FlatGFA
-------

[FlatGFA](./flatgfa) is an efficient representation for GFA files. It is implemented in Rust and available with [Python bindings](./flatgfa-py). The latter is [on PyPI][flatgfa-pypi], so you can get started with:

    $ pip install flatgfa

Then read the [API documentation][flatgfa-py-docs] to see what's available. Or see [the included example](./flatgfa-py/example.py) for a synopsis.

[flatgfa-pypi]: https://pypi.org/project/flatgfa/


Credits
-------

This is a project of the [Capra][] lab at Cornell.
The license is [MIT][].

[capra]: https://capra.cs.cornell.edu
[mit]: https://choosealicense.com/licenses/mit/
