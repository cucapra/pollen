import pathlib
import flatgfa

TEST_DIR = pathlib.Path(__file__).parent
TEST_GFA = TEST_DIR / "../test/tiny.gfa"
TEST_GAF = TEST_DIR / "../test/tiny.gaf"
graph = flatgfa.parse(str(TEST_GFA))
gaf = str(TEST_GAF)
gaf_parser = graph.all_reads(gaf)
for lines in gaf_parser:
    print(lines.name)
    print(lines.sequence())
    print(lines.segment_ranges())
    for element in lines:
        print(element.handle)
        print(element.range)
