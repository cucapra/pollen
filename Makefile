TEST_FILES := t k note5 overlap q.chop LPA DRB1-3123 chr6.C4
BASIC_TESTS := ex1 ex2
GFA_URL := https://raw.githubusercontent.com/pangenome/odgi/ebc493f2622f49f1e67c63c1935d68967cd16d85/test

# A smaller set of test inputs for faster runs.
ifdef SMALL
TEST_FILES := t k note5 overlap q.chop DRB1-3123
endif

OG_FILES := $(BASIC_TESTS:%=tests/basic/%.og) $(TEST_FILES:%=tests/%.og)
DEPTH_OG_FILES := $(OG_FILES:tests/%.og=tests/depth/%.og)

.PHONY: fetch og test clean test-all
fetch: $(TEST_FILES:%=tests/%.gfa)

tests/%.gfa:
	curl -Lo ./$@ $(GFA_URL)/$*.gfa

og: $(OG_FILES)

test: fetch test-depth

test-depth: fetch og
	-turnt --save --env baseline tests/depth/subset-paths/*.txt
	turnt --env calyx-depth tests/depth/subset-paths/*.txt

	-turnt --save --env baseline $(DEPTH_OG_FILES)
	turnt --env calyx $(DEPTH_OG_FILES)


test-data-gen: og
	-turnt --save --env pollen_data_gen_depth_oracle tests/*.og
	turnt --env pollen_data_gen_depth_test tests/*.gfa


#################
#   slow-odgi   #
#################

test-slow-odgi: fetch
	make -C slow_odgi test
