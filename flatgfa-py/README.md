Python Bindings for FlatGFA
===========================

This is a Python interface for the [FlatGFA][] library, which provides an efficient representation for pangenomic variation graphs in the [Graphical Fragment Assembly (GFA)][gfa] format.

You can install it [from PyPI][flatgfa-pypi]:

    $ pip install flatgfa

Then, read [the API documentation][flatgfa-py-docs] for details about what it can do so far.

Development
-----------

The easiest way to get started is with [uv][]:

    $ uv run --package flatgfa python example.py

That should build and install the package and then run our `example.py` script.

Or run the tests:

    $ uv run --package flatgfa pytest

During development, you'll want to rebuild the module using [Maturin][].
One way to do it is to install the necessary command-line tools into the virtualenv, like this:

    $ . .venv/bin/activate
    $ cd flatgfa-py
    $ uv pip install maturin pip
    $ maturin develop

Then, just type `maturin develop` and `pytest` while you work.

[maturin]: https://www.maturin.rs
[flatgfa-py-docs]: https://cucapra.github.io/pollen/flatgfa/
[flatgfa]: https://github.com/cucapra/pollen/tree/main/flatgfa
[gfa]: https://github.com/GFA-spec/GFA-spec/blob/master/GFA1.md
[flatgfa-pypi]: https://pypi.org/project/flatgfa/
[example]: https://github.com/cucapra/pollen/blob/main/flatgfa-py/example.py
[uv]: https://docs.astral.sh/uv/
