Some Examples
=============

This is just a test...

```console
$ flatgfa-sh -p -c 'odgi depth'
depth(stdin) -> stdout

$ flatgfa-sh -p -c 'odgi depth -i chr8.gfa'
depth("chr8.gfa") -> stdout

$ flatgfa-sh -p -c 'odgi depth -i chr8.gfa -r "chm13#chr8"'
depth("chr8.gfa", path="chm13#chr8") -> stdout

```
