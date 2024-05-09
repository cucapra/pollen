# Python Bindings for FlatGFA

This is a Python wrapper for the FlatGFA library.
It is currently in a "proof of concept" state.

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

[maturin]: https://www.maturin.rs
