GFA_FILES := k.gfa DRB1-3123.gfa
GFA_URL := https://raw.githubusercontent.com/pangenome/odgi/ebc493f2622f49f1e67c63c1935d68967cd16d85/test

.PHONY: fetch
fetch: $(GFA_FILES)

$(GFA_FILES): %.gfa:
	curl -Lo $@ $(GFA_URL)/$@
