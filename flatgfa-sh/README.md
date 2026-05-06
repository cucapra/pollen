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

With optimizations enabled via `-O`, flash will detect when an `.og` file can be
bypassed in favor of an existing FlatGFA or text GFA equivalent:

```console
$ flash -p -c 'odgi depth -i ../tests/note5.og'
odgi-view("../tests/note5.og") -> pipe-0
parse-gfa(pipe-0) -> gfa-store-0
path-depth(gfa-store-0) -> stdout

$ flash -p -O -c 'odgi depth -i ../tests/note5.og'
map-file("../tests/note5.flatgfa") -> mmap-0
path-depth(mmap-0) -> stdout

```


Complicated Example
-------------------

Here's a shell script that uses a pipeline to combine `odgi depth` and
`bedtools makewindows`.

```console
$ flash windows.sh
5	0	4

```
