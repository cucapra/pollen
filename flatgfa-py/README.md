Python Bindings for FlatGFA
===========================

This is a Python interface for the [FlatGFA][] library, which provides an efficient representation for pangenomic variation graphs in the [Graphical Fragment Assembly (GFA)][gfa] format.

You can install it [from PyPI][flatgfa-pypi]:

    $ pip install flatgfa

Then, read [the API documentation][flatgfa-py-docs] for details about what it can do so far.

Development
-----------

To build this library, first install [Maturin][]:

    $ pipx install maturin

Next, we'll build and install the Python library in our virtualenv.
Starting from the repository root:

    $ uv venv  # Unless you already created the virtualenv.
    $ uv pip install pip  # Maturin depends on pip.
    $ source .venv/bin/activate
    $ cd flatgfa-py
    $ maturin develop

Now the `flatgfa` module is available to Python programs.
Try our [example][]:

    $ python example.py

Or run the tests:

    $ uv pip install pytest
    $ pytest

[maturin]: https://www.maturin.rs
[flatgfa-py-docs]: https://cucapra.github.io/pollen/flatgfa/
[flatgfa]: https://github.com/cucapra/pollen/tree/main/flatgfa
[gfa]: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md
[flatgfa-pypi]: https://pypi.org/project/flatgfa/
[example]: https://github.com/cucapra/pollen/blob/main/flatgfa-py/example.py
