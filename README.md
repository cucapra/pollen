<h1>
<p align="center">
<img src="https://github.com/cucapra/pollen/blob/main/pollen_icon.png">
</h1>

Pangenome Graph Queries in Calyx
================================

This is a nascent project to build a DSL-to-hardware compiler using [Calyx][] to implement pangenomic graph queries in the vein of [odgi][].

Getting Started
---------------

### Installation


#### Pollen

Clone this repository using 
```
git clone https://github.com/cucapra/pollen.git
```
and then follow the instructions below to set up our dependencies, `calyx` and `odgi`.


#### Calyx

Follow these [instructions](https://docs.calyxir.org/) to install calyx. You must complete the [first](https://docs.calyxir.org/#compiler-installation) and [third](https://docs.calyxir.org/#installing-the-command-line-driver) sections, but feel free to skip the second. The last step should be running `fud check`, which will report that some tools are unavailable. This is okay for our purposes.

We recommend using the native calyx interpreter. After completing the above, run
```
fud config stages.interpreter <full path to calyx repository>/target/debug/interp
```

#### Odgi

Installing odgi [via bioconda](https://odgi.readthedocs.io/en/latest/rst/installation.html#bioconda) seems to be the most straightforward option. If you instead [compile odgi from source](https://odgi.readthedocs.io/en/latest/rst/installation.html#building-from-source), you will need to [edit your python path](https://odgi.readthedocs.io/en/latest/rst/binding/usage.html) in order to use the python bindings.

To verify that odgi is installed and the python bindings are working, open up a python shell and try `import odgi`. If this works, move on to the next section.

We have encountered two gotchas when installing odgi: a version clash with python, and an issue with odgi's memory manager. Below we describe what we think is a complete installation of odgi that addresses both of these issues.

1. Check your python version with `python --version`. We use python 3.9.12 for the rest of this example.
2. Run `mkdir odgi-py; cd odgi-py`.
3. Download the appropriate tarball (in this example, it will have `py39` in its name) from [bioconda][].
4. Untar it, and run `ls lib/python3.9/site-packages/` to ensure that `odgi.cpython*.so` is there. If it is elsewhere, make note of the location and substitute in the next step.
5. Add this to your `PYTHONPATH` with `export PYTHONPATH=<full path to odgi-py>/lib/python3.9/site-packages/`.
6. Preload `jemalloc`: explore under `/usr/lib/x86_64-linux-gnu/` to ensure that `libjemalloc.so.2` is there. If it is not, search under `/lib/x86_64-linux-gnu/` and substitute in the next step.
7. Run `export LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so.2`.
8. Open up a python shell and try `import odgi`.

### Generating an Accelerator

Take [node depth](https://pangenome.github.io/odgi.github.io/rst/commands/odgi_depth.html) as an example. To generate and run a node depth accelerator for the graph `k.og`, first navigate to the root directory of this repository. Then run
```
make fetch
make test/k.og
python3 calyx_depth.py -o depth.futil
python3 parse_data.py test/k.og
fud exec depth.futil --to interpreter-out -s verilog.data depth.data > depth.txt
python3 parse_data.py -di temp.txt
```

What just happened? Below, we walk through the six commands we issued above, pointing out the other options that we could have used.

First, `make fetch` downloads some [GFA][] data files into the `./test` directory.

Second, `make test/*.og` builds the odgi graph files from those GFA files.

Third, we generate the hardware accelerator and write it to a file named `depth.futil`. The commands to generate a node depth hardware accelerator in calyx include:

1. `python3 calyx_depth.py -o depth.futil`
2. `python3 calxy_depth.py -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.futil`
3. `python3 calyx_depth.py -a <filename.og> -o depth.futil`

The commands use the hardware parameters as follows:
1. Uses default hardware parameters.
2. Takes the hardware parameters as input.
3. Automatically infers the hardware parameters from a `.og` file.

Parameters that are specified manually take precedence over those that are inferred automatically, and it is legal to specify just a subset of parameters. For example, `python3 calyx_depth.py -a test/k.og -n=1` will infer `MAX_STEPS` and `MAX_PATHS` from `test/k.og`, but the resulting accelerator will only handle one node.

Fourth, we need to generate some input from our odgi file. This is what we will feed to the hardware accelerator. The following variations all accomplish this:

1. `python3 parse_data.py <filename.og> -o depth.data`
2. `python3 parse_data.py <filename.og> -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.data`
3. `python3 parse_data.py <filename.og> -a <filename2.og> -o depth.data`
4. `python3 parse_data.py <filename.og> -a -o depth.data`
    
The flags work as before, except that if no argument is passed to the `-a` flag, the dimensions are inferred from the input file. **The dimensions of the input must be the same as that of the hardware accelerator.**

Fifth, we run our hardware accelerator. The following code simulates the calyx code for the hardware accelerator:

`fud exec depth.futil --to interpreter-out -s verilog.data depth.data > depth.txt`
    
Sixth, we parse the output into a more readable format.

`python3 parse_data.py -di temp.txt`

[calyx]: https://calyxir.org
[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8006571/#FN8
[bioconda]: https://anaconda.org/bioconda/odgi/files
