from typing import List
import mygfa
import mygfa.preprocess


def touches(path1: str, path2: str, graph: mygfa.Graph) -> bool:
    """Are these two paths different,
    and if so, do they have any segments in common?
    """
    if path1 == path2:
        return False
    segs1 = set(graph.paths[path1].segments)
    segs2 = set(graph.paths[path2].segments)
    return bool(segs1 & segs2)


def overlap(graph: mygfa.Graph, inputpaths: List[str]) -> mygfa.Graph:
    """Which paths touch these input paths?"""
    header_printed = False
    for ip in inputpaths:
        assert ip in graph.paths
        for path in graph.paths.keys():
            if touches(ip, path, graph):
                if not header_printed:
                    print("\t".join(["#path", "start", "end", "path.touched"]))
                    header_printed = True
                print(
                    "\t".join(
                        [ip, "0", str(len(mygfa.preprocess.pathseq(graph)[ip])), path]
                    )
                )
    return graph
