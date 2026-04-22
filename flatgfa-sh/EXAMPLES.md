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

You can even run script files...

```console
$ flash -p example.sh
depth("chr8.pan.og", path="chm13#chr8") -> stdout

```

And normal shell commands pass through...

```console
$ flash -c 'cat example.sh'
#!/usr/bin/env ../target/debug/flash
odgi depth -i chr8.pan.og -r 'chm13#chr8'

```
