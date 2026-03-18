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

## Development

Build the lib by running:

```
$ cargo build
```

This will also update the header file `include/flatgfa.h`. The lib will be built in `target` of the parent directory.
