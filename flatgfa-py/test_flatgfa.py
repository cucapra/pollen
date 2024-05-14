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
    # `gfa.segments` acts like a list.
    assert len(gfa.segments) == 4
    seg = gfa.segments[0]

    # An individual segment exposes its name and nucleotide sequence.
    assert seg.name == 1
    assert seg.sequence() == b"CAAATAAG"

    # You can also pull out the entire sequence of segments.
    seg = list(gfa.segments)[2]
    assert seg.name == 3


def test_paths(gfa):
    # `gfa.paths` similarly acts like a list.
    assert len(gfa.paths) == 2
    assert len(list(gfa.paths)) == 2

    # Individual paths expose their name (a bytestring).
    path = gfa.paths[0]
    assert path.name == b"one"


def test_path_steps(gfa):
    # When you get a path, the path itself acts as a list of steps (handles).
    path = gfa.paths[1]
    assert len(path) == 4
    step = list(path)[0]

    # A step (handle) is a reference to a segment and an orientation.
    assert step.segment.name == 1
    assert step.is_forward
