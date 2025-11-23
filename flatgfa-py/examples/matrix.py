import pathlib
from itertools import islice

import flatgfa


def iter_bits_ones_dash(path, chunk_bytes: int = 64 * 1024):
    ws = b" \t\r\n\v\f"
    with open(path, "rb") as f:
        while True:
            chunk = f.read(chunk_bytes)
            if not chunk:
                break
            for b in chunk:
                if b in ws:
                    continue
                if b == ord("1"):
                    yield True
                elif b == ord("0"):
                    yield False
                else:
                    raise ValueError(
                        f"Invalid character in file: {chr(b)!r} (expected '1' or '0')"
                    )


def compare_row_to_file(row_bits, path) -> tuple[bool, int | None]:
    n = len(row_bits)
    i = 0
    for fb in iter_bits_ones_dash(path):
        if i >= n:
            # file has extra bits
            raise ValueError("File contains more bits than the row length.")
        if fb != bool(row_bits[i]):
            return False, i
        i += 1

    if i != n:
        # file ended early
        raise ValueError(f"File ended early: got {i} bits, expected {n}.")
    return True, None


TEST_DIR = pathlib.Path(__file__).parent
TEST_GFA = TEST_DIR / "./matrix_gaf_folder/Chr3.gfa"
GAF_DIR = (TEST_DIR / "./matrix_gaf_folder").resolve()
# MASTER_TXT = TEST_DIR / "./matrix_gaf_folder/Chr3-pangenotype"

graph = flatgfa.parse(str(TEST_GFA))
gaf = [str(p) for p in GAF_DIR.glob("*.gaf")]
pangenotype_matrix = graph.make_pangenotype_matrix(gaf)
# assert len(pangenotype_matrix) == len(gaf)
# first_row = list(pangenotype_matrix[0])
# ok, where = compare_row_to_file(first_row, str(MASTER_TXT))
# print("OK (all columns match)" if ok else f"Mismatch at column {where}")
FIRST_N = 100
for gaf_path, row in zip(gaf, pangenotype_matrix):
    row01 = [int(b) for b in row]
    # print(pathlib.Path(gaf_path).name, *row01)
    first_bits = islice(row, FIRST_N)
    print(pathlib.Path(gaf_path).name, *map(int, first_bits))
