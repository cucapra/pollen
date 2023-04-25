import sys
from . import mygfa, preprocess


def getpaths(infile):
    return list(mygfa.nonblanks(infile))


def touches(path1, path2, graph):
    """Are these two paths different,
    and if so, do they have any segments in common?
    """
    if path1 == path2:
        return False
    segs1 = set(graph.paths[path1].segments)
    segs2 = set(graph.paths[path2].segments)
    return bool(segs1 & segs2)


def overlap(graph, inputpaths):
    """Which paths touch these input paths?"""
    print("\t".join(["#path", "start", "end", "path.touched"]))
    for ip in inputpaths:
        assert ip in graph.paths
        for path in graph.paths.keys():
            if touches(ip, path, graph):
                print(
                    "\t".join([ip, "0", str(len(preprocess.pathseq(graph)[ip])), path])
                )


# inputpaths = getpaths(open(sys.argv[1], "r"))
