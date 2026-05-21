The FlatGFA Fake Shell
======================

`flash` is a fake Unix shell that lets you run FlatGFA-oriented pipelines using shell-compatible syntax.
You write a script that *appears* to invoke odgi but actually triggers built-in FlatGFA functionality.

Demo
----

Just invoke `flash` by itself for an interactive shell, or use `-c` to run a single command.
You can run normal shell commands, which really do run subprocesses:

```console
$ flash -c 'cat example.sh'
#!/usr/bin/env ../target/debug/flash
odgi depth -d -i ../tests/note5.gfa
odgi depth -i ../tests/note5.gfa

$ flash -c 'head -n1 < README.md'
The FlatGFA Fake Shell

$ flash -c 'head -n1 < README.md | rev'
llehS ekaF AFGtalF ehT

```

But more importantly, you can also use odgi subcommands, which are implemented using in-process calls to the FlatGFA library, without forking anything:

```console
$ flash -c 'odgi depth -d -i ../tests/note5.gfa'
#node.id	depth	depth.uniq
1	2	2
2	0	0
3	2	2
4	2	2

```

Built-in commands even compose with external commands via pipes:

```console
$ flash -c 'odgi depth -d -i ../tests/note5.gfa | tail -n1'
4	2	2

```

You can also run shell script files:

```console
$ flash example.sh
#node.id	depth	depth.uniq
1	2	2
2	0	0
3	2	2
4	2	2
#path	start	end	mean.depth
5	0	13	2
5-	0	13	2

```


Supported Syntax
----------------

Use the `-p` flag for "pretend mode" to see how `flash` parses your shell command.
Here are some things that we currently parse:

```console
$ flash -p -c 'odgi depth'
parse-gfa(stdin) -> gfa-store-0
path-depth(gfa-store-0) -> stdout

$ flash -p -c 'odgi depth -d'
parse-gfa(stdin) -> gfa-store-0
node-depth(gfa-store-0) -> stdout

$ flash -p -c 'odgi depth -i chr8.gfa'
parse-gfa("chr8.gfa") -> gfa-store-0
path-depth(gfa-store-0) -> stdout

$ flash -p -c 'odgi depth -i chr8.gfa -r "chm13#chr8"'
parse-gfa("chr8.gfa") -> gfa-store-0
path-depth(gfa-store-0, path="chm13#chr8") -> stdout

$ flash -p -c 'odgi depth < chr8.gfa > depth.tsv'
parse-gfa("chr8.gfa") -> gfa-store-0
path-depth(gfa-store-0) -> "depth.tsv"

```

Here's how pipelines get parsed:

```console
$ flash -p -c 'foo | bar | baz'
shell("foo", [], input=stdin) -> pipe-0
shell("bar", [], input=pipe-0) -> pipe-1
shell("baz", [], input=pipe-1) -> stdout

$ flash -p -c 'foo | bar | baz > qux'
shell("foo", [], input=stdin) -> pipe-0
shell("bar", [], input=pipe-0) -> pipe-1
shell("baz", [], input=pipe-1) -> "qux"

```


Input GFA File Types
--------------------

Flash also detects FlatGFA files and odgi files (by filename extension)
everywhere that an input GFA is allowed:

```console
$ flash -p -c 'odgi depth -i chr8.flatgfa'
map-file("chr8.flatgfa") -> mmap-0
path-depth(mmap-0) -> stdout

$ flash -p -c 'odgi depth -i chr8.og'
odgi-view("chr8.og") -> pipe-0
parse-gfa(pipe-0) -> gfa-store-0
path-depth(gfa-store-0) -> stdout

```

With optimizations enabled via `-O`, flash will detect when you're reading a
plain-text GFA file but have a FlatGFA file you can use directly instead:

```console
$ flash -p -c 'odgi depth -i ../tests/note5.gfa'
parse-gfa("../tests/note5.gfa") -> gfa-store-0
path-depth(gfa-store-0) -> stdout

$ flash -p -O -c 'odgi depth -i ../tests/note5.gfa'
map-file("../tests/note5.flatgfa") -> mmap-0
path-depth(mmap-0) -> stdout

```

Similarly, when you use an odgi-native `.og` file, an optimization can rewrite
this to use a plain-text GFA file directly or a FlatGFA binary file:

```console
$ flash -p -c 'odgi depth -i ../tests/note5.og'
odgi-view("../tests/note5.og") -> pipe-0
parse-gfa(pipe-0) -> gfa-store-0
path-depth(gfa-store-0) -> stdout

$ flash -p -O -c 'odgi depth -i ../tests/note5.og'
map-file("../tests/note5.flatgfa") -> mmap-0
path-depth(mmap-0) -> stdout

```


Optimizations
-------------

The `-O` flag enables some optimizations in the intermediate representation.
We've seen one (avoiding parsing when a pre-parsed file exists) above.
Here are some other optimizations.

### Eliminate Intermediate BED Files

Some commands that produce BED files as text can also produce in-memory FlatBED resources, avoiding the need for a print/parse round trip.
One optimization detects these round trips and removes them:

```console
$ flash -p -c 'bedtools makewindows -b in.bed -w 16 > intermediate.bed ; odgi depth -i g.gfa -b intermediate.bed'
parse-bed("in.bed") -> bed-store-0
make-windows(bed-store-0, size=16) -> "intermediate.bed"
parse-gfa("g.gfa") -> gfa-store-0
parse-bed("intermediate.bed") -> bed-store-1
interval-depth(gfa-store-0, bed-store-1) -> stdout

$ flash -p -O -c 'bedtools makewindows -b in.bed -w 16 > intermediate.bed ; odgi depth -i g.gfa -b intermediate.bed'
parse-bed("in.bed") -> bed-store-0
make-windows(bed-store-0, size=16) -> bed-store-1
parse-gfa("g.gfa") -> gfa-store-0
interval-depth(gfa-store-0, bed-store-1) -> stdout

```

The `intermediate.bed` file is eliminated.


Complicated Example
-------------------

Here's a shell script that uses a pipeline to combine `odgi depth` and
`bedtools makewindows`.

```console
$ flash windows.sh
#path	start	end	mean.depth
5	0	4	2
5	4	8	2
5	8	12	2
5	12	13	2

```
