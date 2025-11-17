import pathlib
import flatgfa
import numpy as np

TEST_DIR = pathlib.Path(__file__).parent
TEST_GFA = TEST_DIR / "tiny.gfa"
TEST_GAF = TEST_DIR / "tiny.gaf"


def test_gaf_seqs():
    gfa = flatgfa.parse_bytes(TEST_GFA.read_bytes())
    gaf = gfa.all_reads(str(TEST_GAF))
    seqs = ["".join(e.sequence() for e in line) for line in gaf]
    assert seqs == [
        "AAGAAATTTTCT",
        "GAAATTTTCTGGAGTTCTAT",
    ]


def test_gaf_ranges():
    gfa = flatgfa.parse_bytes(TEST_GFA.read_bytes())
    gaf = gfa.all_reads(str(TEST_GAF))
    ranges = [[e.range for e in line] for line in gaf]
    assert ranges == [
        [(5, 8), (0, 9), (1, 0)],
        [(7, 8), (0, 18), (0, 0)],
    ]

def test_construct_pangenotype_matrix():
    gfa = flatgfa.parse_bytes(TEST_GFA.read_bytes())
    pangenotype_matrix = gfa.make_pangenotype_matrix([(str(TEST_GAF))])
    assert pangenotype_matrix == [[True, True,True,True]]

def test_pangenotype_regression():
    gfa = flatgfa.parse_bytes(TEST_GFA.read_bytes())
    pangenotype_matrix = gfa.make_pangenotype_matrix([str(TEST_GAF)])

    # Optional: inspect structure
    print("Matrix:", pangenotype_matrix)

    # Example nested loop: iterate over rows and values
    for row in pangenotype_matrix:
        for val in row:
            print("Segment present:", val)

    # Convert nested list of bools -> NumPy array of floats
    a = np.array(pangenotype_matrix, dtype=float)
    b = np.random.rand(a.shape[0])
    result = np.linalg.lstsq(a, b, rcond=None)
    x, residuals, rank, s = result

    # Print or assert just to verify
    print("Coefficients:", x)
    print("Residuals:", residuals)
    print("Rank:", rank)
    print("Singular values:", s)

    # Simple test condition: check the shapes
    assert x.shape[0] == a.shape[1]
