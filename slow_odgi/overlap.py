import sys
import mygfa

def getpaths(infile):
    return list(mygfa.nonblanks(infile))

def touches(path1, path2, graph):
    if path1 == path2:
        return False
    segs1 = set(graph.paths[path1].segments)
    segs2 = set(graph.paths[path2].segments)
    return bool(segs1 & segs2)

def pathseqlen(path, graph):
    return sum(len(graph.segments[seg.name].seq) for seg in \
        graph.paths[path].segments)

def print_overlaps(graph, inputpaths):
    print("\t".join(["#path", "start", "end", "path.touched"]))
    for ip in inputpaths:
        assert (ip in graph.paths)
        for pathname in graph.paths.keys():
            if touches(ip, pathname, graph):
                print("\t".join([ip, "0", str(pathseqlen(ip, graph)), \
                    pathname]))

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1][-6:] == ".paths":
        inputpaths = getpaths(open(sys.argv[1], 'r'))
        graph = mygfa.Graph.parse(sys.stdin)
        print_overlaps(graph, inputpaths)
