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

* [mygfa](./mygfa), a simple Python library for parsing, processing, and emitting [GFA][] files.
* [slow_odgi](./slow_odgi), a reference implementation of several GFA queries from the [odgi][] tool using `mygfa`.
* [FlatGFA](./polbin), an experimental fast binary format for representing and analyzing GFA files.
* A proof-of-concept Calyx-based [hardware accelerator generator](./pollen_py) for a single GFA query (`odgi depth`) and a data generator for this hardware.

[calyx]: https://calyxir.org
[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md


`mygfa` and `slow_odgi`
-----------------------

The `mygfa` library is an extremely simple Python library for representing (and parsing and emitting) GFA files. It emphasizes clarify over efficiency. Similarly, `slow_odgi` is a set of GFA analyses based on `mygfa`; it's meant to act as a *reference implementation* of the much faster functionality in [odgi][]. Check out [the slow_odgi README](slow_odgi/) for more details.

To use them, try using [uv][]:

    $ uv venv
    $ uv pip install -r requirements.txt
    $ source .venv/bin/activate

Now type `slow_odgi --help` to see if everything's working.

[uv]: https://github.com/astral-sh/uv


Credits
-------

This is a project of the [Capra][] lab at Cornell.
The license is [MIT][].

[capra]: https://capra.cs.cornell.edu
[mit]: https://choosealicense.com/licenses/mit/
