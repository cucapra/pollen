Some Examples
=============

This is just a test...

```console
$ flatgfa-sh -p -c 'odgi depth'
depth(Stdin, path=None) -> Stdout

$ flatgfa-sh -p -c 'odgi depth -i chr8.gfa'
depth(File("chr8.gfa"), path=None) -> Stdout

$ flatgfa-sh -p -c 'odgi depth -i chr8.gfa -r "chm13#chr8"'
depth(File("chr8.gfa"), path=Some("chm13#chr8")) -> Stdout

```
