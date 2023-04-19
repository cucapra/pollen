# Pollen Python Library

## Installation
To use the Pollen odgi accelerators:
1. Install [flit](https://flit.readthedocs.io/en/latest/#install).
2. Install the `pollen` package:
```
  $ cd pollen-py/
  $ flit install --symlink
```

## Example
```
exine depth
```
generates a calyx hardware accelerator which can compute `node depth` for very small `odgi` graphs.