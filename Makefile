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

# This is the version that does, for each algorithm, setup-then-test.
test-slow-odgi-interleave: og test-slow-chop test-slow-crush test-slow-degree test-slow-depth test-slow-flatten test-slow-flip test-slow-inject test-slow-matrix test-slow-normalize test-slow-overlap test-slow-paths test-slow-validate
# Known points of divergence:
# test-slow-flip: we disagree with odgi over note5
# test-slow-inject: we disagree with odgi over DRB1 and chr6
# These are documented as issues in our repo.

test-slow-chop: og
	-turnt --save --env chop_oracle test/*.og
	turnt --env chop_test test/*.gfa

test-slow-crush: og
	-turnt --save --env crush_oracle test/*.og
	-turnt --env crush_test test/*.gfa
	# -turnt --save --env crush_oracle test/handmade/crush*.gfa
	# turnt --env crush_test test/handmade/crush*.gfa

test-slow-degree: og
	-turnt --save --env degree_oracle test/*.og
	turnt --env degree_test test/*.gfa

test-slow-depth: og
	-turnt --save --env depth_oracle test/*.og
	turnt --env depth_test test/*.gfa

test-slow-flip: fetch
	-turnt --save --env flip_oracle test/*.gfa
	# -turnt --save --env flip_oracle test/handmade/flip*.gfa
	-turnt --env flip_test test/*.gfa
	# turnt --env flip_test test/handmade/flip*.gfa

test-slow-flatten: og
	-turnt --save --env flatten_oracle test/*.og
	turnt --env flatten_test test/*.gfa

test-slow-inject: og
	-turnt --save --env inject_setup test/*.gfa
	-turnt --save --env inject_oracle test/*.og
	turnt --env inject_test test/*.gfa

test-slow-matrix: og
	-turnt --save --env matrix_oracle test/*.og
	turnt --env matrix_test test/*.gfa

test-slow-normalize: og
	-turnt --save --env normalize_oracle test/*.og
	turnt --env normalize_test test/*.gfa

test-slow-overlap: og
	-turnt --save --env overlap_setup test/*.gfa
	-turnt --save --env overlap_oracle test/*.og
	turnt --env overlap_test test/*.gfa

test-slow-paths: og
	-turnt --save --env paths_oracle test/*.og
	turnt --env paths_test test/*.gfa

test-slow-validate: fetch
	-turnt --save --env validate_setup test/*.gfa
	for fn in `ls test/*.temp`; do `mv $$fn $${fn%.*}_temp.gfa`; done
	-turnt --save --env validate_oracle test/*_temp.gfa
	turnt --env validate_test test/*_temp.gfa
	rm test/*_temp.gfa

# This is the version that first sets up all the oracles and then tests them.
test-slow-odgi: test-slow-validate slow-odgi-all-oracles slow-odgi-all-tests
# It is too annoying to "unzip" `validate` in this way,
# so I just do it the old way, first.

# Collecting all the setup/oracle stages of slow-odgi into once place.
# This can be run once, noisily, and then slow-odgi-all-tests can be run
# quietly against the expect files created here.
slow-odgi-all-oracles: og
	-turnt --save --env chop_oracle test/*.og
	-turnt --save --env crush_oracle test/*.og
	# -turnt --save --env crush_oracle test/handmade/crush*.gfa
	-turnt --save --env degree_oracle test/*.og
	-turnt --save --env depth_oracle test/*.og
	-turnt --save --env flip_oracle test/*.gfa
	# -turnt --save --env flip_oracle test/handmade/flip*.gfa
	-turnt --save --env flatten_oracle test/*.og
	-turnt --save --env inject_setup test/*.gfa
	-turnt --save --env inject_oracle test/*.og
	-turnt --save --env matrix_oracle test/*.og
	-turnt --save --env normalize_oracle test/*.og
	-turnt --save --env overlap_setup test/*.gfa
	-turnt --save --env overlap_oracle test/*.og
	-turnt --save --env paths_oracle test/*.og

# In reality slow-odgi-all-tests needs slow-odgi-all-oracles as a dependency.
# Running the below by itself is faster and less noisy,
# but do so ONLY if you know that the GFAs have not changed,
# in which case slow-odgi-all-oracles would have had no effect anyway.
slow-odgi-all-tests:
	turnt --env chop_test test/*.gfa
	-turnt --env crush_test test/*.gfa
	# -turnt --env crush_test test/handmade/crush*.gfa
	-turnt --env degree_test test/*.gfa
	-turnt --env depth_test test/*.gfa
	-turnt --env flip_test test/*.gfa
	# -turnt --env flip_test test/handmade/flip*.gfa
	-turnt --env flatten_test test/*.gfa
	-turnt --env inject_test test/*.gfa
	-turnt --env matrix_test test/*.gfa
	-turnt --env normalize_test test/*.gfa
	-turnt --env overlap_test test/*.gfa
	-turnt --env paths_test test/*.gfa

clean:
	rm -rf $(TEST_FILES:%=%.*)
	rm -rf $(TEST_FILES:%=test/%.*)

	rm -rf test/basic/*.og

	rm -rf test/*temp.*
	rm -rf test/depth/*.out
	rm -rf test/depth/basic/*.out
	rm -rf test/depth/subset-paths/*.out

	# rm -rf test/handmade/*.crush
	# rm -rf test/handmade/*.flip

test/chr8.pan.gfa:
	curl -Lo ./test/chr8.pan.gfa.gz $(GFA_ZIP_URL)
	gunzip ./test/chr8.pan.gfa.gz

test/%.gfa:
	curl -Lo ./$@ $(GFA_URL)/$*.gfa

%.og: %.gfa
	odgi build -g $^ -o $@
