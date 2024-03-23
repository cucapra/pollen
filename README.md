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

* `mygfa`, a simple Python library for parsing, processing, and emitting [GFA][] files.
* `slow_odgi`, a reference implementation of several GFA queries from the [odgi][] tool using `mygfa`.
* A proof-of-concept Calyx-based hardware accelerator generator for a single GFA query (`odgi depth`) and a data generator for this hardware.
* FlatGFA, an experimental fast binary format for representing and analyzing GFA files.


`mygfa` and `slow_odgi`
-----------------------

The `mygfa` library is an extremely simple Python library for representing (and parsing and emitting) GFA files. It emphasizes clarify over efficiency. Similarly, `slow_odgi` is a set of GFA analyses based on `mygfa`; it's meant to act as a *reference implementation* of the much faster functionality in [odgi][]. Check out [the slow_odgi README](slow_odgi/) for more details.

To use them, try using [uv][]:

    $ uv venv
    $ uv pip install -r requirements.txt
    $ source .venv/bin/activate

Now type `slow_odgi --help` to see if everything's working.

[uv]: https://github.com/astral-sh/uv



Proof-of-Concept Hardware Generator
-----------------------------------

This repository contains a proof-of-concept hardware accelerator generator for a simple GFA query. This section contains some guides for trying out this generator.

### The Docker Image

Running the hardware generator is easy if you use our [Docker image][package]:

    docker run -it --rm ghcr.io/cucapra/pollen:latest

If you prefer to install locally, we point you to the somewhat more involved instructions [below](#installing-locally).

### Generating an Accelerator: Quick

If you want to compute the [depth][] of all the nodes in the graph, the following command will generate and run a node depth accelerator:
```
exine depth -a -r <filename.og>
```

This will automatically generate a node depth accelerator whose dimensions match the input data, compute the node depth, and remove the accelerator once the computation is done.

To save the files generated from the previous command in `<path>`, use the `--tmp-dir` flag:
```
exine depth -a -r <filename.og> --tmpdir <path>
```
The node depth accelerator will be saved at `<path>/<filename.futil>` and the input data will be saved at `<path>/<filename.data>`.

### Generating an Accelerator: Full Walkthrough

Take [depth][] as an example. To generate and run a node depth accelerator for the graph `k.og`, first navigate to the root directory of this repository. Then run
```
make fetch
make test/k.og
exine depth -o depth.futil
exine depth -d test/k.og -o depth.data
exine depth -r depth.data --accelerator depth.futil
```

What just happened? Below, we walk through the five commands we issued above, pointing out the other options that we could have used.

First, `make fetch` downloads some [GFA][] data files into the `./test` directory.

Second, `make test/*.og` builds the odgi graph files from those GFA files.

Third, we generate the hardware accelerator and write it to a file named `depth.futil`. The commands to generate a node depth hardware accelerator in Calyx include:

1. `exine depth -o depth.futil`
2. `exine depth -a <filename.og> -o depth.futil`
3. `exine depth -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.futil`

The commands use the hardware parameters as follows:
1. Uses default hardware parameters.
2. Automatically infers the hardware parameters from a `.og` file.
3. Takes the hardware parameters as input.

Parameters that are specified manually take precedence over those that are inferred automatically, and it is legal to specify just a subset of parameters. For example, `exine depth -a test/k.og -n=1` will infer `MAX_STEPS` and `MAX_PATHS` from `test/k.og`, but the resulting accelerator will only handle one node.

Fourth, we need to generate some input from our odgi file. This is what we will feed to the hardware accelerator. The following variations all accomplish this:

1. `exine depth -d <filename.og> -o depth.data`
2. `exine depth -d <filename.og> -a <filename2.og> -o depth.data`
3. `exine depth -d <filename.og> -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.data`
4. `exine depth -d <filename.og> -a -o depth.data`

The flags work as before, except that if no argument is passed to the `-a` flag, the dimensions are inferred from the input file. **The dimensions of the input must be the same as that of the hardware accelerator.**

Fifth, we run our hardware accelerator. The following code simulates the Calyx code for the hardware accelerator and outputs the node depth table:

```
exine depth -r depth.data -x depth.futil
```

### Installing Locally

You will need  [Flit][] version 3.7.1 and [Turnt][] version 1.11.0.
We will guide you through the installation of our major dependencies, [Calyx][] and [odgi][], and then show you how to install Pollen itself.

#### Calyx

Below we show you how to build Calyx from source and set it up for our use.
If you are curious, this tracks the "[installing from source][calyx-install-src]" and "[installing the command-line driver][calyx-install-fud]" sections of the Calyx documentation.

1. `git clone https://github.com/cucapra/calyx.git`
2. `cd calyx`
3. `cargo build`
3. `flit -f fud/pyproject.toml install -s --deps production`
4. `fud config --create global.root $(pwd)`
5. `cargo build -p interp`
6. `fud config stages.calyx.exec $(pwd)/target/debug/calyx`
7. `fud config stages.interpreter.exec $(pwd)/target/debug/interp`
8. `flit -f calyx-py/pyproject.toml install -s`
9. `fud check`

You will be warned that `synth-verilog` and `vivado-hls` were not installed correctly; this is fine for our purposes.

#### Odgi

We recommend that you build odgi from source, as described [here][odgi-from-source].
To check that this worked, run `odgi` from the command line.

Some parts of Pollen presently use odgi's Python bindings.
You will need to edit your PYTHONPATH, as explained [here][odgi-pythonpath], to enable this.
To verify that this worked, open up a Python shell and try `import odgi`.
If it succeeds quietly, great!
If it segfaults, try the preload step explained [here][odgi-preload].

#### Pollen

Clone this repository:

    git clone https://github.com/cucapra/pollen.git

And then install the Python tools using [uv][]:

    $ uv venv
    $ uv pip install -r requirements.txt
    $ source .venv/bin/activate

[calyx]: https://calyxir.org
[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://github.com/lh3/gfatools/blob/master/doc/rGFA.md#the-reference-gfa-rgfa-format
[bioconda]: https://anaconda.org/bioconda/odgi/files
[flit]: https://flit.pypa.io/en/stable/
[turnt]: https://github.com/cucapra/turnt
[calyx-install-src]: https://docs.calyxir.org/#installing-from-source-to-use-and-extend-calyx
[calyx-install-fud]: https://docs.calyxir.org/#installing-the-command-line-driver
[package]: https://github.com/cucapra/pollen/pkgs/container/pollen
[odgi-from-source]: https://odgi.readthedocs.io/en/latest/rst/installation.html#building-from-source
[odgi-pythonpath]: https://odgi.readthedocs.io/en/latest/rst/binding/usage.html
[odgi-preload]: https://odgi.readthedocs.io/en/latest/rst/binding/usage.html#optimise
[depth]: https://pangenome.github.io/odgi.github.io/rst/commands/odgi_depth.html
