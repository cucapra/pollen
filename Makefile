TEST_FILES := t k note5 overlap q.chop LPA DRB1-3123 chr6.C4
GFA_URL := https://raw.githubusercontent.com/pangenome/odgi/ebc493f2622f49f1e67c63c1935d68967cd16d85/test

# A smaller set of test inputs for faster runs.
ifdef SMALL
TEST_FILES := t k note5 overlap q.chop DRB1-3123
endif

tests/%.gfa:
	curl -Lo ./$@ $(GFA_URL)/$*.gfa

.PHONY: fetch
fetch: $(TEST_FILES:%=tests/%.gfa)

.PHONY: test-slow-odgi
test-slow-odgi: fetch
	make -C slow_odgi test
