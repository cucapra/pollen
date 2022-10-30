main: full

quick: proposal.tex
	xelatex proposal.tex

full: proposal.tex
	latexmk -pdf proposal.tex

.PHONY: clean quick main full
clean:
	rm -f *.aux *.xcp *.blg *.log *.pdf *.bbl *.dvi *.out *.fls *.fdb_latexmk