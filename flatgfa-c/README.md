# C Bindings for FlatGFA

This project builds C bindings of a subset of operations from the [FlatGFA](https://github.com/cucapra/pollen/tree/main/flatgfa) library.

The bindings allow you to:

* Parse a GFA file to a FlatGFA representation.
* Get the number of segments within the FlatGFA.
* Get the sequence and length of segments.
* Get the number of paths.
* Get the name of a path.
* Get the number of steps within a specific path.
* Get the corresponding segment and orientation of a specific step within a path.

## Build the Library

Build the library by running:

    cargo build

This will produce a header file in `include/flatgfa.h` (using [cbindgen][]) as well as both static and dynamic libraries in the workspace `target` directory.

[cbindgen]: https://github.com/mozilla/cbindgen
