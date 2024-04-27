%.svg: %.csv bar.vl.json
	jq '.data.url |= "$<"' bar.vl.json | npx -p vega -p vega-lite vl2svg > $@
