import sys
import mygfa



def getpaths(infile):
    paths = []
    for line in mygfa.nonblanks(infile):
        paths.append(line)
    return paths

def touches(path1, path2, graph):
    if path1 == path2:
        return False
    for seg_o1 in graph.paths[path1].segments: # (segment, orientation) tuple
        for seg_o2 in graph.paths[path2].segments:
            if seg_o1 == seg_o2:
                return True
    return False

def pathseqlen(path, graph):
    length = 0
    for seg, _ in graph.paths[path].segments:
        length += len (graph.segments[seg].seq)
    return length

def print_overlaps(graph, inputpaths):
    print("\t".join(["#path","path_touched"]))
    for ip in inputpaths:
        assert (ip in graph.paths)
        for pathname in graph.paths.keys():
            if touches(ip, pathname, graph):
                print("\t".join([ip, "0", str(pathseqlen(ip, graph)), pathname]))

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1][-6:] == ".paths":
        inputpaths = getpaths(open(sys.argv[1], 'r'))
        graph = mygfa.Graph.parse(sys.stdin)
        print_overlaps(graph, inputpaths)
