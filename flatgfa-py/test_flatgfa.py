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
    seg = list(gfa.segments)[2]
    assert seg.name == 3


def test_paths(gfa):
    assert len(gfa.paths) == 2
    path = gfa.paths[0]
    assert path.name == b"x"
    assert len(path) == 10
    step = list(path)[0]
    assert step.segment.name == 1
    assert step.is_forward
