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

.PHONY: test-flatgfa
test-flatgfa: fetch
	cd flatgfa ; cargo build

	turnt -e flatgfa_mem -e flatgfa_file -e flatgfa_file_inplace tests/*.gfa

	-turnt --save -v -e chop_oracle_fgfa tests/*.gfa
	turnt -v -e flatgfa_chop tests/*.gfa

	-turnt --save -v -e odgi_depth tests/*.gfa
	turnt -v -e flatgfa_depth tests/*.gfa

clean:
	rm tests/*.flatgfa tests/*.inplace.flatgfa tests/*.chop tests/*.depth