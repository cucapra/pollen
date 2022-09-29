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
Follow these [instructions](https://docs.calyxir.org/) to install calyx. You will need to configure the calyx driver, `fud`. We recommend using the native calyx interpreter, so once `fud` is set up, run
```
fud config stages.interpreter $ABSOLUTE_PATH_TO_CALYX_REPO/target/debug/interp
```
where `$ABSOLUTE_PATH_TO_CALYX_REPO` is the absolute path to the root directory. For example, if you downloaded calyx in `/Users/username/project`, you would run `fud config stages.interpreter /Users/username/project/target/debug/interp`.

#### Odgi

You will need to install the python bindings for [odgi]. Instructions for installing odgi can be found [here](https://odgi.readthedocs.io/en/latest/rst/installation.html). If you compile odgi from its source, you will need to [edit your python path](https://odgi.readthedocs.io/en/latest/rst/binding/usage.html) to use the python bindings.

To verify that you have the python bindings, open up a python shell and try `import odgi`. If this doesn't work, you can try a different method of installation, or you can download the `.so` files from [bioconda](https://anaconda.org/bioconda/odgi/files) for the version of python you are running (`python3 --version`) and add them to your `PYTHONPATH` instead.

### Generating an Accelerator

To generator and run an accelerator to compute the node depth for `k.og`, first navigate to the root directory of this repository. Then run
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

```
python3 calyx_depth.py -o depth.futil
python3 calyx_depth.py -a <filename> -o depth.futil
python3 calxy_depth.py -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.futil
```

The first command uses default hardware parameters; the second automatically infers them from a `.og` file; the third takes the parameters as input. Automatically inferred parameters take precedence over manually specified ones, and just a subset of parameters may be specified.

To run the hardware accelerator, we need to generate some input using one of the following commands:

```
python3 parse_data.py <filename> -o depth.data
python3 parse_data.py <filename> -a -o depth.data
python3 parse_data.py <filename> -a <filename2> -o depth.data
python3 calxy_depth.py <filename> -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.data
```
    
This is similar to the previous command except that if no argument is passed to the `-a` flag, the dimensions are inferred from the input file. The dimensions of the input must be the same as that of the hardware accelerator.

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
