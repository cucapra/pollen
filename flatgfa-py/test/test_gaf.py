import pytest
import flatgfa


def test_gaf():
    gfa = flatgfa.parse("test/tiny.gfa")
    gaf = gfa.test_gaf("test/tiny.gaf")
    seqs = ["".join(e.get_seq() for e in line) for line in gaf]
    assert seqs == [
        "AAGAAATTTTCT",
        "GAAATTTTCTGGAGTTCTAT",
    ]
