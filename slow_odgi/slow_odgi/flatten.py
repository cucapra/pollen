def get_fasta_legend(graph):
    """The main deliverable is the FASTA:
    Simply traverse the segments in order and glue their seqs together.
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


def print_bed(graph, legend, name):
    """With the legend computed during FASTA-building, this is easy."""

    print("\t".join(["#name", "start", "end", "path.name", "strand", "step.rank"]))
    for path in graph.paths.values():
        for i, handle in enumerate(path.segments):
            start, end = legend[handle.name]
            print(
                "\t".join(
                    [
                        name,
                        str(start),
                        str(end),
                        path.name,
                        "+" if handle.ori else "-",
                        str(i),
                    ]
                )
            )


def insert_newlines(string, every=80):
    """odgi's output does this for this algorithm, so we follow them."""
    return "\n".join(string[i : i + every] for i in range(0, len(string), every))


def flatten(graph, name):
    """Print out the FASTA and BED."""
    print(f">{name}")
    # This is a bit harcoded for files living in test/file.gfa
    # Would be nice to neaten this up and make it less brittle.
    fasta, legend = get_fasta_legend(graph)
    print(insert_newlines(fasta))
    print_bed(graph, legend, name)
