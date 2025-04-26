import pathlib
import flatgfa

TEST_DIR = pathlib.Path(__file__).parent
TEST_GFA = str(TEST_DIR / "tiny.gfa")
TEST_GAF = str(TEST_DIR / "tiny.gaf")


def test_gaf_seqs():
    gfa = flatgfa.parse(TEST_GFA)
    gaf = gfa.all_reads(TEST_GAF)
    seqs = ["".join(e.sequence() for e in line) for line in gaf]
    assert seqs == [
        "AAGAAATTTTCT",
        "GAAATTTTCTGGAGTTCTAT",
    ]


def test_gaf_ranges():
    gfa = flatgfa.parse(TEST_GFA)
    gaf = gfa.all_reads(TEST_GAF)
    ranges = [[e.range for e in line] for line in gaf]
    assert ranges == [
        [(5, 8), (0, 9), (1, 0)],
        [(7, 8), (0, 18), (0, 0)],
    ]
