Pangenome Graph Queries in Calyx
================================

This is a nascent project to build a DSL-to-hardware compiler using [Calyx][] to implement pangenomic graph queries in the vein of [odgi][].
It is very much a work in progress.

Getting Started
---------------

You will need to install and configure [calyx][] and the python bindings for [odgi]. Instructions for installing odgi can be found [here](https://odgi.readthedocs.io/en/latest/rst/installation.html). If you compile odgi from its source, you will need to [edit your python path](https://odgi.readthedocs.io/en/latest/rst/binding/usage.html) to use the python bindings. Since this will only give you the odgi bindings for the most recent version of python, you may need to download the `.so` files from [bioconda](https://odgi.readthedocs.io/en/latest/rst/binding/usage.html) and add them to your `PYTHONPATH` instead.

You will need to fetch some useful inputs by typing `make fetch`.
This downloads some simple [GFA][] data files into the `./test` directory.

If you want to build odgi graph files from the GFA files, try something like `make k.og`.

Then, to generate a node depth hardware accelerator in calyx, type one of the following commands:
```
python3 calyx_depth.py -o depth.futil
python3 calyx_depth.py -a test/<filename> > depth.futil
python3 calxy_depth.py -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.futil
```

The calyx will be written to `depth.futil`. The first command uses default hardware parameters; the second automatically infers them from a `.og` file; the third takes the parameters as input. Automatically inferred parameters take precedence over manually specified ones, but a subset of parameters may be specified.

To run the hardware accelerator, we need to generate some input:
```
python3 parse_data.py test/<filename> -o depth.data
python3 parse_data.py test/<filename> -a > depth.data
python3 calxy_depth.py test/<filename> -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.data
```
    
This is similar to the previous command, except that the `-a` flag merely infers the dimensions from the input file. The dimensions of the input must be no larger than those of the hardware accelerator.

Finally, 

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
