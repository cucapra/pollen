GFA_FILES := t.gfa k.gfa note5.gfa overlap.gfa q.chop.gfa DRB1-3123.gfa chr6.C4.gfa LPA.gfa
GFA_URL := https://raw.githubusercontent.com/pangenome/odgi/ebc493f2622f49f1e67c63c1935d68967cd16d85/test
GFA_ZIP_URL := https://s3-us-west-2.amazonaws.com/human-pangenomics/pangenomes/scratch/2021_05_06_pggb/gfas/chr8.pan.gfa.gz

.PHONY: fetch test clean test-all
fetch: $(GFA_FILES)

test: $(GFA_FILES)
	-turnt --save --env baseline $(foreach file,$(GFA_FILES),test/$(file))
	turnt $(foreach file,$(GFA_FILES),test/$(file))
	-turnt --save --env baseline test/basic/*.gfa
	turnt test/basic/*.gfa

test-all: $(GFA_FILES) GFA_ZIP_URL
	-turnt --save --env baseline test/*.gfa
	turnt test/*.gfa
	-turnt --save --env baseline test/basic/*.gfa
	turnt test/basic/*.gfa

clean:
	rm -rf $(foreach file,$(GFA_FILES),test/$(file))
	rm -rf test/*.og
	rm -rf test/*.out

$(GFA_FILES): %.gfa:
	curl -Lo ./test/$@ $(GFA_URL)/$@

GFA_ZIP:
	curl -Lo ./test/chr8.pan.gfa.gz $(GFA_ZIP_URL)
	gunzip ./test/chr8.pan.gfa.gz

%.og: %.gfa
	odgi build -g $^ -o $@
