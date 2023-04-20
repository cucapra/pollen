### Overview

`slow-odgi` is a reference implementation of [`odgi`](https://github.com/pangenome/odgi). It is written purely in Python, with correctness and clarity as goals and speed as a non-goal. Think of it as a code-ey spec for `odgi` functions.

### Testing

	To test `slow-odgi`, we treat `odgi` as an oracle and compare our outputs against theirs. We mostly test against a set of GFA graphs available in the `odgi` repository, and, in a few cases, supplement these with short hand-rolled GFA files of our own.

To run these tests, you will need 
1. `odgi`; see [here](https://github.com/pangenome/odgi). Our tests were run against a built-from-source copy of `odgi` (commit 34f006f).
2. `turnt`; see [here](https://github.com/cucapra/turnt).

With these in place, run `make test-slow-odgi`. The "oracle" files will be generated first, and this will toss up a large number of warnings which can all be ignored. Then the tests will begin to run, and the `ok`/`not-ok` signals there are actually of interest. 

There are a few known points of divergence versus `odgi`:
1. `flip` disgrees against graphs note5.gfa and flip4.gfa.
2. `inject` disagrees against graphs DRB1-3123.gfa and chr6.C4.gfa.


### Explanation of Commands

The remainder of this document will explain the eleven commands that we have implemented in some detail. 


#### `chop`
Given the graph
```
S 1 AAAA
S 2 TTT
S 3 GGGGCCCC
P x 1+,3+,2+
L 1 + 3 +
L 3 + 2 +
```
running `chop` with parameter `3` gives
```
S 1 AAA
S 2 A
S 3 TTT
S 4 GGG
S 5 GCC
S 6 CC
P x 1+,2+,4+,5+,6+,3+
L 1 + 2 +   // new, to bridge the chop of S1
L 2 + 4 +   // renumber old link 1+3+
L 4 + 5 +   // new, to bridge the chop of S3
L 5 + 6 +   // new, to bridge the chop of S3
L 6 + 3 +   // renumber old link 3+2+
```
That is,
1. Segments that had sequences longer than `3` characters have been chopped, repeatedly if needed.
2. All segments have been renumbered.
3. Paths have been adjusted.
4. Old links have been adjusted and new links added  (for reasons given in the comments).


#### `crush`

Given the graph
```
S 1 ANNN
S 2 NTTN
S 3 NGGG
P x 1+,2+,3+
L ...
```
running `crush` gives
```
S 1 AN
S 2 NTTN
S 3 NGGG
P x 1+,2+,3+
L ...
```
That is, "runs" of `N` have been swapped out for single instances of `N`. Observe that this is entirely intra-segment; we did not treat the end of S2 and the beginning of S3 as a contiguous "run".


#### `degree`

Given the graph
```
S	1	AAAA
S	2	TTTT
S	3	GGGG
S	4	CCCC
P	x	1+,3+,4+,3+	*
P	y	1+,3-		*
L	1	+	2	+	0M
L	1	+	3	+	0M
L	1	+	3	-	0M
L	3	-	4	+	0M
L	2	+	4	+	0M
L	3	+	4	+	0M
```
running `degree` gives
```
1	3
2	2
3	4
4	3
```
Where each row has the name of the segment and the segment's _degree_. This is a count of how many times a segment appears on the graph's links; the direction of traversal is immaterial. It does not matter what the paths say.


#### `depth`

Given the graph
```
S	1	AAAA
S	2	TTTT
S	3	GGGG
S	4	CCCC
P	x	1+,3+,4+,3+	*
P	y	1+,3-		*
L	1	+	2	+	0M
L	1	+	3	+	0M
L	1	+	3	-	0M
L	3	-	4	+	0M
L	2	+	4	+	0M
L	3	+	4	+	0M
```
running `depth` gives
```
1	2	2
2	0	0
3	3	2
4	1	1
```
Where each row has the name of the segment, the segment's _depth_, and the segment's _unique depth_. The depth is a count of how many times a segment appears on the graph's paths; the direction of traversal is immaterial. The unique depth is similar but only counts an appearance on one path once. It does not matter what the links say.


#### `emit`

GFAs have line-entries of four kinds: headers, segments, paths, and links. Their order does not matter, so the following is fine:
```
H ...
L ...
P ...
S ...
L ...
P ...
S ...
...
```
`emit` normalizes a GFA so that its entries appear in a stable order: headers, then segments, then paths, and then links. Order is also enforced between lines of the same kind. Doing this minimizes diffs when modifying files. 


#### `flip`

Given the graph
```
S 1 A
S 2 TTT
S 3 G
P x 1+,2-,3+ *
P y 1+,2+ *
L 1 + 2 + 0M
L 2 + 3 + 0M
```
running `flip` gives 
```
S 1 A
S 2 TTT
S 3 G
P x_inv 3-,2+,1- *
P y 1+,2+ *
L 1 + 2 + 0M
L 2 + 1 - 0M    // new
L 2 + 3 + 0M
L 3 - 2 + 0M    // new

```
That is, 
1. Any paths that were traversing their segments _more backwards than forwards_ have been flipped. Note that this is _weighted_, which is why the single `2-` in path `x` is enough to justify flipping path `x`. The flipping involves flipping the sign of each step the path traverses and also reversing the path's list of steps.
2. Those paths that have just been fllipped have had `_inv` added to their names.
3. Links have been added in support of the newly flipped paths.


#### `flatten`

Given the graph
```
S 1 AAAA
S 2 TT
S 3 GGGG
P x 1+,2+,3+
P y 3+,2-
L ...
```
running `flatten` produces two outputs:
```
AAAATTTTGGGG
```
and
```
0	4	x	+	0
4	6	x	+	1
6	10	x	+	2
6	10	y	+	0
4	6	y	-	1
```
That is, 
1. In the first file, just a concatenation of all the segments' sequences. This is called the FASTA file.
2. In the second file, a "key" by which to retrieve path information from the FASTA file. This file is called the BED file, and each of its rows is read as follows:
	- The name in the middle tells us which path we are describing.
	- The number on the far right says which segment of the path we are describing.
	- The two numbers of the left say where to start and stop reading off the FASTA file.
	- The fourth item, the sign, says whether the path crossed that sequence in the forwards or backwards direction.


#### `inject`

Given the graph
```
S 1 AAAA
S 2 TTTT
S 3 GGGG
P x 1+,2+,3+ *
```
and a BED file
```
x    0    8    y 
x    0    4    z
```
running `inject` gives
```
S 1 AAAA
S 2 TTTT
S 3 GGGG
P x 1+,2+,3+ *
P y 1+,2+ *
P z 1+ *
```
That is, the BED file has information about which paths to track and, for each path, over which of its run to track it and what name to give the resultant subpath. Running `inject` adds these paths.

Consider, though, a more subtle example. The following BED file describes a legal subpath, but one that does not happen to line up the current segment-boundaries.
```
x    1    6    y    
```
Working with the same graph as before, `inject` now needs to split segments 1  and 2 in order to add path `y`.
```
S 1 A
S 2 AAA
S 3 TT
S 4 TT
S 5 GGGG
P x 1+,2+,3+,4+,5+	*
P y 2+,3+	*
```
Observe that this required edits to the path `x` as well.   


#### `matrix`

Given the graph
```
S	1	AAAA
S	2	TTTT
S	3	GGGG
S	4	CCCC
L	1	+	2	+	0M
L	1	+	3	+	0M
L	1	+	3	-	0M
```
running `matrix` produces 
```
4 4 12    // header
1 2 1
2 1 1
1 3 1
3 1 1
1 3 1
3 1 1
1 2 1
2 1 1
2 4 1
4 2 1
3 4 1
4 3 1
```
That is, 
1. A _header_ that twice lists the highest-numbered segments, and then lists the total number of entries in the matrix below.
2. Rows that indicate adjacency. Each row ends with `1`. Observe, further:
	- Each link triggers two adjacencies; e.g., the presence of `L 1 + 2 +` was enough to add both `1 2` and `2 1` to the matrix.
	- The direction of link traversal did not matter; the fact that `L 1 + 3 -` pointed at the "reverse" handle of segment 3 did not have any effect in the matrix representation.
	- No deduplication was performed.


#### `overlap`

Given the graph
```
S 1 AAAA... // a sequence of length 90
S 2 TTTT... // a sequence of length 10
S 3 GGGG... // a sequence of length 40
P x 1+,2+
P y 2+,3+
P z 1+
L ... 
```
and the BED file
```
x    0    30
```
running `overlap` gives nothing: no paths in the graph overlapped with path `x` between its 0th and 30th characters.

As one would expect, running the same graph with the BED file
```
x    95   97       
```
gives
```
x    95   97   y
```

It is also possible to query `overlap` with a simpler file of the form
```
y
```
and, in this case, `overlap` assumes that the querier wants to investigate the path `y` along its entire length. The answer is
```
y    0   50   x
```


#### `paths`

Given the graph
```
S 1 AAAA
S 2 TTTT
S 3 GGGG
P x 1+,2+,3+
P y 2+,3+
L ...
```
running `paths` gives
```
x
y
```
That is, it simply produces a list of the graph's paths by name.


#### `validate`
Given the graph
```
S 1 AAAA
S 2 TTTT
S 3 GGGG
P x 1+,2+,3+
L 1 + 2 +
```
running `validate` complains that we are missing a link; namely the link `L 2 + 3 +`. Run against a graph where each path _is_ backed up by links, this command decrees the graph valid and succeeds quietly.