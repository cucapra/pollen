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
odgi depth -d -i ../tests/overlap.gfa

$ flash -c 'head -n1 < README.md'
The FlatGFA Fake Shell

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

You can also run shell script files:

```console
$ flash example.sh
#node.id	depth	depth.uniq
1	2	2
2	0	0
3	2	2
4	2	2
#node.id	depth	depth.uniq
1	1	1
2	2	2
3	1	1
4	1	1
5	2	2
6	4	3
7	1	1
8	2	2
9	2	2
10	1	1

```


Supported Syntax
----------------

Use the `-p` flag for "pretend mode" to see how `flash` parses your shell command.
Here are some things that we currently parse (not all of which are implemented in the evaluator just yet):

```console
$ flash -p -c 'odgi depth'
depth(stdin) -> stdout

$ flash -p -c 'odgi depth -i chr8.gfa'
depth("chr8.gfa") -> stdout

$ flash -p -c 'odgi depth -i chr8.gfa -r "chm13#chr8"'
depth("chr8.gfa", path="chm13#chr8") -> stdout

$ flash -p -c 'odgi depth < chr8.gfa > depth.tsv'
depth("chr8.gfa") -> "depth.tsv"

```
