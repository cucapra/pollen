TEST_FILES := t k note5 overlap q.chop DRB1-3123 chr6.C4 LPA
BASIC_TESTS := ex1 ex2
GFA_FILES := $(TEST_FILES:%=test/%.gfa) $(BASIC_TESTS:%=test/basic/%.gfa)
OG_FILES := $(GFA_FILES:%.gfa=%.og)
GFA_URL := https://raw.githubusercontent.com/pangenome/odgi/ebc493f2622f49f1e67c63c1935d68967cd16d85/test
GFA_ZIP_URL := https://s3-us-west-2.amazonaws.com/human-pangenomics/pangenomes/scratch/2021_05_06_pggb/gfas/chr8.pan.gfa.gz

.PHONY: fetch og test clean test-all
fetch: $(TEST_FILES)

og: fetch $(OG_FILES)

test: og
	-turnt --save --env baseline $(GFA_FILES)
	turnt $(GFA_FILES)

	-turnt --save --env baseline test/subset-paths/*.txt
	-turnt test/subset-paths/*.txt

test-slow: GFA_ZIP_URL test/chr8.pan.og
	-turnt --save --env baseline test/chr8.pan.gfa
	turnt test/chr8.pan.gfa

test-all: test test-slow

clean:
	rm -rf $(TEST_FILES:%=test/%.*)

	rm -rf test/basic/*.og
	rm -rf test/basic/*.out

	rm -rf test/subset-paths/*.out

test/chr8.pan.gfa:
	curl -Lo ./test/chr8.pan.gfa.gz $(GFA_ZIP_URL)
	gunzip ./test/chr8.pan.gfa.gz

$(TEST_FILES):
	curl -Lo ./test/$@.gfa $(GFA_URL)/$@.gfa

%.og: %.gfa
	odgi build -g $^ -o $@
