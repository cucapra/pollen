import pathlib
import flatgfa

TEST_DIR = pathlib.Path(__file__).parent
TEST_GFA = str(TEST_DIR / "tiny.gfa")
TEST_GAF = str(TEST_DIR / "tiny.gaf")


def test_gaf():
    gfa = flatgfa.parse(TEST_GFA)
    gaf = gfa.all_reads(TEST_GAF)
    seqs = ["".join(e.sequence() for e in line) for line in gaf]
    assert seqs == [
        "AAGAAATTTTCT",
        "GAAATTTTCTGGAGTTCTAT",
    ]
