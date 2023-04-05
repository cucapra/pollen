import sys
import mygfa

def fasta(graph):
    """ The main deliverable is the FASTA:
    Simply, for all segments, the seqs glued together. Traverse segments in order.
    However, it pays to do some bookkeeping now.
    legend[segname] stores the [start, end) of the spot in the FASTA that
    segname's seq is featured.
    """
    ans = ""
    legend = {}
    ptr = 0
    for segment in graph.segments.values():
        ans += segment.seq
        length = len(segment.seq)
        legend[segment.name] = (ptr, ptr + length)
        ptr += length
    return ans, legend

def print_bed(graph, legend):
    print("\t".join(["#name", "start", "end", "path.name", "strand", "step.rank"]))
    for path in graph.paths.values():
        for i, (segname, o) in enumerate(path.segments):
            start, end = legend[segname]
            print ("\t".join([odginame, str(start), str(end), path.name, "+" if o else "-", str(i)]))

def insert_newlines(string, every=80):
    return '\n'.join(string[i:i+every] for i in range(0, len(string), every))

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1][-4:] == ".gfa":
        infile = open(sys.argv[1], 'r')
        graph = mygfa.Graph.parse(infile)
        odginame = sys.argv[1][:-4] + ".og"
        print(f">{odginame}")
        # TODO: this is a bit hardocded for files living in test/file.gfa
        # Would be nice to neaten this up and make it less brittle.

        fasta, legend = fasta(graph)
        print(insert_newlines(fasta))
        print_bed(graph, legend)