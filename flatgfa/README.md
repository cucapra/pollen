FlatGFA
=======

This is an experimental [odgi][]-like tool for manipulating pangenome graphs in the popular [GFA][] format. It works by converting the GFA to a "flat," pointer-free representation that can be stored directly on disk for zero-copy reads and writes.

[odgi]: https://odgi.readthedocs.io/en/latest/
[gfa]: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md

Build
-----

It's a Rust project, so all you need to do is:

    $ cargo build --release

Then you might like to do something like this to put a symlink on your `$PATH`:

    $ ln -s `pwd`/target/release/fgfa ~/.local/bin

Now see what's available:

    $ fgfa --help

Convert GFA Files
-----------------

This tool can run queries directly on GFA text files, but you can amortize that cost by converting to the native FlatGFA format. Try this:

    $ fgfa -I chr22.hprc-v1.0-pggb.gfa -o chr22.flatgfa

In general, you will want to remember these flags for input and output:

* `-i` or `-o`: Read or write our native FlatGFA binary format.
* `-I` or `-O`: Read or write the standard GFAv1 text format. Or, just omit the relevant flag to use standard input or standard output.

So combining `-I` and `-o` as above does the conversion you want. FlatGFA files should be a little smaller than their text counterparts. Now that we have one, we can convert it back to a GFA text file like this:

    $ fgfa -i chr22.flatgfa | less

Simple Queries
--------------

Here are some things we can do with FlatGFA files. See some basic statistics about the graph:

    $ fgfa -i chr22.flatgfa stats -S

Or use `-L` instead to see information about self-loops. This output should match [`odgi stats`][odgi-stats].

Get a list of all the path names in the graph---or, in this case, just the first few:

    $ fgfa -i chr22.flatgfa paths | head

Find the graph position of a given base-pair offset within a certain path, just like [`odgi position -v`][odgi-position]:

    $ fgfa -i chr22.flatgfa position -p chm13#chr22,12345,+

Extract a subgraph from a larger graph around a specific segment:

    $ fgfa -i chr22.flatgfa -o chr22.sub.flatgfa extract -n 25 -c
    $ fgfa -i chr22.sub.flatgfa stats -S

Unfortunately, this extraction doesn't quite match [`odgi extract`][odgi-extract] yet (because I haven't quite been able to figure out how it's supposed to work).

[odgi-stats]: https://odgi.readthedocs.io/en/latest/rst/commands/odgi_stats.html
[odgi-position]: https://odgi.readthedocs.io/en/latest/rst/commands/odgi_position.html
[odgi-extract]: https://odgi.readthedocs.io/en/latest/rst/commands/odgi_extract.html
