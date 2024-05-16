# Python Bindings for FlatGFA

This is a Python wrapper for the FlatGFA library.
Read [the API documentation][flatgfa-py-docs] for details about what it can do so far.

To build it, first install [Maturin][]:

    pipx install maturin

Next, we'll build and install the Python library in our virtualenv.
Starting from the repository root:

    uv venv  # Unless you already created the virtualenv.
    uv pip install pip  # Maturin depends on pip.
    source .venv/bin/activate
    cd flatgfa-py
    maturin develop

Now the `flatgfa` module is available to Python programs.
Try our example:

    python example.py

Or run the tests:

    uv pip install pytest
    pytest

[maturin]: https://www.maturin.rs
[flatgfa-py-docs]: https://cucapra.github.io/pollen/flatgfa/
