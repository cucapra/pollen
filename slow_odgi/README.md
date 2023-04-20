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

### Explanation of Commands

The remainder of this document will explain the eleven commands that we have implemented in some detail.

#### `chop`
Given graph.gfa
```
S 1 AAAA
S 2 TTTT
S 3 GGGG
P 1+,3+,2+
L 1 + 3 +
L 3 + 2 +
```
running `chop` with parameter 3 gives
```
S 1 AAA
S 2 A
S 3 TTT
S 4 T
S 5 GGG
S 6 G
P 1+,2+,5+,6+,2+,3+
L 1 + 2 +   // new, to bridge the chop
L 2 + 5 +   // renumber old link
L 3 + 4 +   // new
L 5 + 6 +   // new
L 6 + 3 +   // renumber
```
Observe that the segments have not only been chopped have also been renumbered continuously, that the paths have been adjusted, and that changes have been made to the links (as marked).

#### `crush`
Given `graph.gfa`
```
S 1 ANNN
S 2 NTTN
S 3 NGGG
P 1+,2+,3+
L ...
```
running `crush` gives
```
S 1 AN
S 2 NTTN
S 3 NGGG
P 1+,2+,3+
L ...
```
That is, "runs" of `N` are swapped out for single instances of `N`. Observe that this is entirely intra-segment; we did not treat the end of S2 and the beginning of S3 as a contiguous "run".

#### `degree`

#### `depth`

#### `emit`

GFAs have line-entries of four kinds: headers, segments, paths, and links. Their order does not matter. 
`emit` normalizes a GFA so that its enties appear in a stable order: headers, then segments, then paths, and then links. The links are sorted, with the same goal in mind.
Doing this minimizes diffs between files. 

#### `flip`

#### `flatten`

#### `inject`

Given `graph.gfa`
```
S 1 AAAA
S 2 TTTT
S 3 GGGG
P x 1+,2+,3+ *
```
and `new_paths.bed`
```
x    0    8    y    
```
running `inject` gives
```
S 1 AAAA
S 2 TTTT
S 3 GGGG
P x 1+,2+,3+ *
P y 1+,2+ *
```
That is, you provide in the .bed file information about which path to track and over which of its run to track it, along with a new path name. The result is that a new path is inserted; it is a subpath of the original.

Here's the rub: what if the .bed file describes a _legal_ subpath, but one that does not happen to line up the current segment-boundaries?
```
x    1    6    y    
```
We need to split segments 1  and 2 in order to make this work.
```
S 1 A
S 2 AAA
S 3 TT
S 4 TT
S 5 GGGG
P x 1+,2+,3+,4+,5+	*
P y 2+,3+	*
```
As you can see, this required edits to the path x as well.

#### `matrix`

#### `overlap`

#### `paths`

#### `validate`