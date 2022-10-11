Pangenome Graph Queries in Calyx
================================

This is a nascent project to build a DSL-to-hardware compiler using [Calyx][] to implement pangenomic graph queries in the vein of [odgi][].
It is very much a work in progress.

Getting Started
---------------

First, clone the repo using 
```git clone https://github.com/cucapra/calyx-pangenome.git```
Then follow the instructions below to set up `calyx` and `odgi`.

### Installing Dependencies

#### Calyx
Follow these [instructions](https://docs.calyxir.org/) to install calyx. You must complete the [first](https://docs.calyxir.org/#compiler-installation) and [third] (https://docs.calyxir.org/#installing-the-command-line-driver) sections, but feel free to skip the second. We recommend using the native calyx interpreter, so once `fud` is set up, run
```
fud config stages.interpreter <full path to Calyx repository>/target/debug/interp
```
where `<full path to Calyx repository` is the absolute path to the root directory. For example, if you downloaded calyx in `/Users/username/project`, you would run `fud config stages.interpreter /Users/username/project/calyx/target/debug/interp`.

#### Odgi

You will need to install the python bindings for [odgi]. Instructions for installing odgi can be found [here](https://odgi.readthedocs.io/en/latest/rst/installation.html). You might also need to [preload `jemalloc`](https://odgi.readthedocs.io/en/latest/rst/binding/usage.html#optimise).

Installing odgi via `bioconda` seems to be the most straightforward option. If you instead compile odgi from its source, you will need to [edit your python path](https://odgi.readthedocs.io/en/latest/rst/binding/usage.html) to use the python bindings.

To verify that the python bindings are working, open up a python shell and try `import odgi`. If this doesn't work, you can also download the `.so` files from [bioconda](https://anaconda.org/bioconda/odgi/files) for the version of python you are running and add them to your `PYTHONPATH`. For example, if `python --version` is 3.7, fetch `odgi...py37....tar.bz2`. Then, after unzipping the `.tar.bz2` file, the `.so` files would be located in `lib/python3.7/site-packages`, so you could run `export PYTHONPATH=$PYTHONPATH:$(pwd)/lib/python3.7/site-packages` to add this to your python path.

### Generating an Accelerator

Take node depth as an example. To generate and run a node depth accelerator for `k.og`, first navigate to the root directory of this repository. Then run
```
make fetch
make test/k.og
python3 calyx_depth.py -o depth.futil
python3 parse_data.py test/k.og
fud exec depth.futil --to interpreter-out -s verilog.data depth.data > depth.txt
python3 parse_data.py -di temp.txt
```

First, `make fetch` downloads some [GFA][] data files into the `./test` directory.

To build the odgi graph files from the GFA files, run `make test/*.og`.

Then, `python3 calyx_depth.py -o depth.futil` generates the hardware accelerator and writes it to a file named `depth.futil`. The commands to generate a node depth hardware accelerator in calyx include:

1. `python3 calyx_depth.py -o depth.futil`
2. `python3 calyx_depth.py -a <filename> -o depth.futil`
3. `python3 calxy_depth.py -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.futil`

The commands compute the hardware parameters as follows:
1. Uses default hardware parameters
2. Automatically infers the hardware parameters from a `.og` file
3. Takes the hardware parameters as input.

Automatically inferred parameters take precedence over manually specified ones, and a subset of parameters may be specified. For example, `python3 calyx_depth.py -a test/k.og -n=1` will infer `MAX_STEPS` and `MAX_PATHS` from `test/k.og`, but the resulting accelerator can only handle one node.

To run the hardware accelerator, we need to generate some input using one of the following commands:

1. `python3 parse_data.py <filename> -o depth.data`
2. `python3 parse_data.py <filename> -a <filename2> -o depth.data`
3. `python3 parse_data.py <filename> -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.data`
4. `python3 parse_data.py <filename> -a -o depth.data`
    
This is similar to the previous command except that if no argument is passed to the `-a` flag, the dimensions are inferred from the input file. **The dimensions of the input must be the same as that of the hardware accelerator.**

Now you can run your hardware accelerator: 

``` 
fud exec depth.futil --to interpreter-out -s verilog.data depth.data > depth.txt
```
    
will simulate the calyx code for the hardware accelerator. To parse the output in a more readable format, run
    
```
python3 parse_data.py -di temp.txt
```

[calyx]: https://calyxir.org
[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8006571/#FN8
