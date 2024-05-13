import pytest
import flatgfa

TEST_GFA = "../tests/k.gfa"


@pytest.fixture
def gfa():
    return flatgfa.parse(TEST_GFA)


def test_segs(gfa):
    assert len(gfa.segments) == 15
    seg = gfa.segments[0]
    assert seg.name == 1
    assert seg.sequence() == b"CAAATAAG"
