Some Examples
=============

This is just a test...

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

Actually running stuff...

```console
$ flash -c 'odgi depth -d -i ../tests/note5.gfa'
#node.id	depth	depth.uniq
1	2	2
2	0	0
3	2	2
4	2	2

```

You can even run script files...

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

And normal shell commands pass through...

```console
$ flash -c 'cat example.sh'
#!/usr/bin/env ../target/debug/flash
odgi depth -d -i ../tests/note5.gfa
odgi depth -d -i ../tests/overlap.gfa

$ flash -c 'wc < example.sh'
       3      12     111

```
