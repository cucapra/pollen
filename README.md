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
Follow these [instructions](https://docs.calyxir.org/) to install calyx. You will need to complete the Installing the Commandline Driver section, but can skip Running the Core Tests. We recommend using the native calyx interpreter, so once `fud` is set up, run
```
fud config stages.interpreter <full path to Calyx repository>/target/debug/interp
```
where `<full path to Calyx repo` is the absolute path to the root directory. For example, if you downloaded calyx in `/Users/username/project`, you would run `fud config stages.interpreter /Users/username/project/calyx/target/debug/interp`.

#### Odgi

You will need to install the python bindings for [odgi]. Instructions for installing odgi can be found [here](https://odgi.readthedocs.io/en/latest/rst/installation.html). Installing odgi via `bioconda` seems to be the most straightforward option. If you instead compile odgi from its source, you will need to [edit your python path](https://odgi.readthedocs.io/en/latest/rst/binding/usage.html) to use the python bindings. 

To verify that the python bindings are working, open up a python shell and try `import odgi`. If this doesn't work, you can also download the `.so` files from [bioconda](https://anaconda.org/bioconda/odgi/files) for the version of python you are running and add them to your `PYTHONPATH`. For example, if `python --version` is 3.7, fetch `odgi...py37....tar.bz2`. You will have to extract the `.so` files by unzipping the `.tar.bz2` file.

To finish the setup, run `flit install --user -s` from the root directory.

### Generating an Accelerator

Take node depth as an example. To generate and run a node depth accelerator for `k.og`, first navigate to the root directory of this repository. Then run

```
make fetch
make test/k.og
pollen depth -o depth.futil
pollen depth --action=parse --file test/k.og -o depth.data
pollen depth --action=run --file depth.data --accelerator depth.futil
```

First, `make fetch` downloads some [GFA][] data files into the `./test` directory. Then `make test/*.og` builds the odgi graph files from the GFA files.

Then, `pollen depth -o depth.futil` generates a hardware accelerator and writes it to a file named `depth.futil`. The commands to generate a node depth hardware accelerator in calyx include:

```
pollen depth -o depth.futil
pollen depth -a <filename> -o depth.futil
pollen depth -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.futil
```

The first command uses default hardware parameters; the second automatically infers them from a `.og` file; the third takes the parameters as input. Manually specified parameters take precedence over automatically inferred ones, and just a subset of parameters may be specified.

To run the hardware accelerator, we need to generate some input using one of the following commands:

```
pollen depth --action=parse -f <filename> -o depth.data
pollen depth --action=parse -f <filename> -a <filename2> -o depth.data
pollen depth --action=parse -f <filename> -n=MAX_NODES -e=MAX_STEPS -p=MAX_PATHS -o depth.data
```
    
The `--action=parse` option indicates that we are generating input from `<filename>`. The `-f` flag must be specified. The dimensions of the input must match the dimensions of the hardware accelerator.

Now you can run your hardware accelerator: 

``` 
pollen depth --action=run -f <data_file> --accelerator <futil_file>
```
    
will simulate the calyx code for the hardware accelerator. If you want to quickly compute node depth, try

```
pollen depth --action=run -f <filename> -a <filename>
```

This will automatically generate a `.futil` file whose dimensions match the input data, compute the node depth, and remove the accelerator once the computation is done.

[calyx]: https://calyxir.org
[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8006571/#FN8