import pathlib

import flatgfa

TEST_DIR = pathlib.Path(__file__).parent
TEST_GFA = TEST_DIR / "tiny.gfa"
TEST_GAF = TEST_DIR / "tiny.gaf"
TEST_GAF_2 = TEST_DIR / "tiny2.gaf"


def test_pangenotype_matrix():
    gfa = flatgfa.parse_bytes(TEST_GFA.read_bytes())
    pangenotype_matrix = gfa.make_pangenotype_matrix([str(TEST_GAF), str(TEST_GAF_2)])
    assert pangenotype_matrix == [
        [True, True, True, True],
        [True, True, False, True],
    ]
