import sys
import mygfa

def fasta(graph):
    fasta = ""
    for segment in graph.segments.values():
        fasta += segment.seq
    print(fasta)

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1][-4:] == ".gfa":
        infile = open(sys.argv[1], 'r')
        graph = mygfa.Graph.parse(infile)
        print(f">{sys.argv[1][5:-4]}.og")
        # TODO: this is a bit hardocded for files living in test/file.gfa
        # Would be nice to neaten this up and make it less brittle.
        fasta(graph)