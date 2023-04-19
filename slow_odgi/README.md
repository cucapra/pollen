### Overview

`slow-odgi` is a reference implementation of [`odgi`](https://github.com/pangenome/odgi). It is written purely in Python, with correctness and clarity as goals and speed as a non-goal. Think of it as a code-ey spec for `odgi` functions.

### Testing

To test `slow-odgi`, we treat `odgi` as an oracle and compare our outputs against theirs. We mostly test against a set of GFA graphs available in the `odgi` repository, and, in a few cases, supplement these with short hand-rolled GFA files of our own.

To run these tests, you will need 
1. `odgi`; see [here](https://github.com/pangenome/odgi). Our tests were run against a built-from-source copy of `odgi` (commit 34f006f).
2. `turnt`; see [here](https://github.com/cucapra/turnt).

With these in place, run `make test-slow-odgi`. The "oracle" files will be generated first, and this will toss up a large number of warnings which can all be ignored. Then the tests will begin to run, and the `ok`/`not-ok` signals there are actually of interest. 

There are a few know points of divergence versus `odgi`:
1. `flip` disgrees against graphs note5.gfa and flip4.gfa.
2. `inject` disagrees against graphs DRB1-3123.gfa and chr6.C4.gfa.

### Overview of Commands

The remainder of this document will explain the eleven commands that we have implemented in some detail.

#### `chop`

#### `crush`

#### `degree`

#### `depth`

#### `emit`

#### `flip`

#### `flatten`

#### `inject`

#### `matrix`

#### `overlap`

#### `paths`

#### `validate`