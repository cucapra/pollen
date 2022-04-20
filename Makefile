GFA_FILES := k.gfa DRB1-3123.gfa
TEST_FILE := DRB1-3123.gfa
GFA_URL := https://raw.githubusercontent.com/pangenome/odgi/ebc493f2622f49f1e67c63c1935d68967cd16d85/test

.PHONY: fetch test
fetch: $(GFA_FILES)

test: $(TEST_FILE)
	./test.sh $(TEST_FILE)

$(GFA_FILES): %.gfa:
	curl -Lo $@ $(GFA_URL)/$@

%.og: %.gfa
	odgi build -g $^ -o $@
