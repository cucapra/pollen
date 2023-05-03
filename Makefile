TEST_FILES := t k note5 overlap q.chop LPA DRB1-3123 chr6.C4
BASIC_TESTS := ex1 ex2
OG_FILES := $(BASIC_TESTS:%=test/basic/%.og) $(TEST_FILES:%=test/%.og)
DEPTH_OG_FILES := $(OG_FILES:test/%.og=test/depth/%.og)
GFA_URL := https://raw.githubusercontent.com/pangenome/odgi/ebc493f2622f49f1e67c63c1935d68967cd16d85/test
GFA_ZIP_URL := https://s3-us-west-2.amazonaws.com/human-pangenomics/pangenomes/scratch/2021_05_06_pggb/gfas/chr8.pan.gfa.gz

.PHONY: fetch og test clean test-all
fetch: $(TEST_FILES:%=test/%.gfa)

og: $(OG_FILES)

test: og test-depth

test-depth: og
	-turnt --save --env baseline test/depth/subset-paths/*.txt
	turnt test/depth/subset-paths/*.txt

	-turnt --save --env baseline $(DEPTH_OG_FILES)
	turnt $(DEPTH_OG_FILES)


test-data-gen: og
	-turnt --save --env data_gen_depth_oracle test/*.og
	turnt --env data_gen_depth_test test/*.gfa


#################
#   slow-odgi   #
#################

# Sets up all the odgi-oracles and then tests slow-odgi against them.
test-slow-odgi: slow-odgi-setup slow-odgi-oracles slow-odgi-tests

# Collects all the setup/oracle stages of slow-odgi into once place.
slow-odgi-setup: og
	-turnt --save --env depth_setup test/*.gfa
	-turnt --save --env inject_setup test/*.gfa
	-turnt --save --env overlap_setup test/*.gfa

# Collects all the oracle stages of slow-odgi into once place.
# This can be run once, noisily, and then slow-odgi-all-tests can be run
# quietly against the expect-files created here.
# Note that, in reality, this depends on the setup stage above.
# Run this by itself ONLY if you know that the setup stages don't need to be
# run afresh.
slow-odgi-oracles: og
	-turnt --save --env chop_oracle test/*.og
	-turnt --save --env crush_oracle test/*.og
	-turnt --save --env degree_oracle test/*.og
	-turnt --save --env depth_oracle test/*.og
	-turnt --save --env flip_oracle test/*.og
	-turnt --save --env flatten_oracle test/*.og
	-turnt --save --env inject_oracle test/*.og
	-turnt --save --env matrix_oracle test/*.og
	-turnt --save --env overlap_oracle test/*.og
	-turnt --save --env paths_oracle test/*.og
	-turnt --save --env validate_oracle test/*.og

# In reality slow-odgi-tests depends on slow-odgi-oracles above.
# Running the below by itself is faster and less noisy,
# but do so ONLY if you know that the graphs and the odgi commands have not
# changed, in which case slow-odgi-oracles would have had no effect anyway.
slow-odgi-tests:
	turnt --env chop_test test/*.gfa
	-turnt --env crush_test test/*.gfa
	-turnt --env degree_test test/*.gfa
	-turnt --env depth_test test/*.gfa
	-turnt --env flip_test test/*.gfa
	-turnt --env flatten_test test/*.gfa
	-turnt --env inject_test test/*.gfa
	-turnt --env matrix_test test/*.gfa
	-turnt --env overlap_test test/*.gfa
	-turnt --env paths_test test/*.gfa
	-turnt --env validate_test test/*.gfa


# The basic test suite above, plus a few handmade tests for good measure.
# Those are described below.
test-slow-odgi-careful: slow-odgi-validate-careful slow-odgi-crush-careful slow-odgi-flip-careful

# The fetch-ed graphs are valid, so `validate` succeeds quietly against them.
# To actually see `validate` in action, we need to walk over the graphs and
# drop some of their links.
# This is what `validate_setup` does, creating graphname.temp files.
# It is too annoying to run this every time (it pollutes the .gfa namespace
# of the test directory), so we provide this as a separate "careful" test.
slow-odgi-validate-careful: fetch
	-turnt --save --env validate_setup test/*.gfa
	for fn in `ls test/*.temp`; do `mv $$fn $${fn%.*}_temp.gfa`; done
	-turnt --save --env validate_oracle_careful test/*_temp.gfa
	turnt --env validate_test test/*_temp.gfa
	rm test/*_temp.gfa

# Handmade files to test crush and flip more comprehensively.
slow-odgi-crush-careful: fetch
	-turnt --save --env crush_oracle test/handmade/crush*.gfa
	-turnt --env crush_test test/handmade/crush*.gfa

slow-odgi-flip-careful: fetch
	-turnt --save --env flip_oracle test/handmade/flip*.gfa
	-turnt --env flip_test test/handmade/flip*.gfa


# All the slow-odgi tests, basic + careful.
test-slow-odgi-all: test-slow-odgi test-slow-odgi-careful

clean:
	rm -rf $(TEST_FILES:%=%.*)
	rm -rf $(TEST_FILES:%=test/%.*)

	rm -rf test/basic/*.og
	rm -rf test/*temp.*
	rm -rf test/depth/*.out
	rm -rf test/depth/basic/*.out
	rm -rf test/depth/subset-paths/*.out
	rm -rf test/handmade/*.crush
	rm -rf test/handmade/*.flip

test/chr8.pan.gfa:
	curl -Lo ./test/chr8.pan.gfa.gz $(GFA_ZIP_URL)
	gunzip ./test/chr8.pan.gfa.gz

test/%.gfa:
	curl -Lo ./$@ $(GFA_URL)/$*.gfa

%.og: %.gfa
	odgi build -g $^ -o $@
