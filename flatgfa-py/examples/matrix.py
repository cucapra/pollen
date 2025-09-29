import pathlib
import flatgfa
TEST_DIR = pathlib.Path(__file__).parent
# CHR6_GFA = TEST_DIR /"chr6.gfa"
# CHR6_GAF = TEST_DIR /"chr6gaf.gaf"
TEST_GFA = TEST_DIR / "../test/tiny.gfa"
GAF_DIR = (TEST_DIR / "./matrix_gaf_folder").resolve()
graph = flatgfa.parse(str(TEST_GFA))
gaf = [str(p) for p in GAF_DIR.glob("*.gaf")]
pangenotype_matrix = graph.make_pangenotype_matrix(gaf)
assert len(pangenotype_matrix) == len(gaf)
for gaf_path, row in zip(gaf, pangenotype_matrix):
    row01 = [int(b) for b in row]   
    print(pathlib.Path(gaf_path).name, *row01)