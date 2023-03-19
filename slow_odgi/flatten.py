import sys
import mygfa

def fasta(graph):
    fasta = ""
    for segment in graph.segments.values():
        fasta += segment.seq
    return(fasta)

def bed(graph):
    print("\t".join(["#name", "start", "end", "path.name", "strand", "step.rank"]))
    for path in graph.paths.values():
        i = 0
        f = fasta(graph)
        ptr = 0
        for (segname, orientation) in path.segments:
            seq = graph.segments[segname].seq
            start = f[ptr:].find(seq)
            end = start + len(seq)
            print ("\t".join([odginame, str(start+ptr), str(end+ptr), path.name, "+", str(i)]))
            i += 1
            ptr += end

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1][-4:] == ".gfa":
        infile = open(sys.argv[1], 'r')
        graph = mygfa.Graph.parse(infile)
        odginame = sys.argv[1][:-4] + ".og"
        print(f">{odginame}")
        # TODO: this is a bit hardocded for files living in test/file.gfa
        # Would be nice to neaten this up and make it less brittle.
        print(fasta(graph))
        bed(graph)