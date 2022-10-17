quick: gfa.tex
	xelatex gfa.tex

full: gfa.tex
	xelatex gfa.tex; bibtex gfa; xelatex gfa.tex; xelatex gfa.tex; open gfa.pdf

.PHONY: clean
clean:
	rm -f *.aux *.xcp *.blg *.log *.pdf *.bbl *.dvi *.out
