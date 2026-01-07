import pathlib
import sys
from itertools import islice

import flatgfa

FIRST_N = 100


def matrix_demo(gfa_path, gaf_dir):
    # Construct a pangenotype matrix for the `gfa_path` graph and all the GAF
    # files in `gaf_dir`.
    graph = flatgfa.parse(gfa_path)
    gaf = [str(p) for p in pathlib.Path(gaf_dir).glob("*.gaf")]
    pangenotype_matrix = graph.make_pangenotype_matrix(gaf)

    assert len(pangenotype_matrix) == len(gaf)

    # Just print out a few entries from the matrix.
    for gaf_path, row in zip(gaf, pangenotype_matrix):
        first_bits = islice(row, FIRST_N)
        print(pathlib.Path(gaf_path).name, *map(int, first_bits))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("usage: matrix.py <GFA file> <GAF directory>")
        sys.exit(1)
    matrix_demo(*sys.argv[1:])
