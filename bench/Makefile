%.svg: %.csv bar.vl.json
	sed 's/FILE/$*/' < bar.vl.json | npx -p vega -p vega-lite vl2svg > $@
