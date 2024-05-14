import pytest
import flatgfa

TINY_GFA = b"""
H	VN:Z:1.0
S	1	CAAATAAG
S	2	AAATTTTCTGGAGTTCTAT
S	3	TTG
S	4	CCAACTCTCTG
P	one	1+,2+,4-	*
P	two	1+,2+,3+,4-	*
L	1	+	2	+	0M
L	2	+	4	-	0M
L	2	+	3	+	0M
L	3	+	4	-	0M
""".strip()


@pytest.fixture
def gfa():
    return flatgfa.parse_bytes(TINY_GFA)


def test_segs(gfa):
    assert len(gfa.segments) == 4
    seg = gfa.segments[0]
    assert seg.name == 1
    assert seg.sequence() == b"CAAATAAG"
    seg = list(gfa.segments)[2]
    assert seg.name == 3


def test_paths(gfa):
    assert len(gfa.paths) == 2
    assert len(list(gfa.paths)) == 2
    path = gfa.paths[0]
    assert path.name == b"one"


def test_path_steps(gfa):
    path = gfa.paths[1]
    assert len(path) == 4
    step = list(path)[0]
    assert step.segment.name == 1
    assert step.is_forward
