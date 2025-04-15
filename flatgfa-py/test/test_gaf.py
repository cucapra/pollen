import pytest
import flatgfa


def test_gaf():
    gfa = flatgfa.parse("something.gfa")
    gfa.test_gaf("something.gaf")
