import sys
import json
from json import JSONEncoder
import mygfa


class SegmentEncoder(JSONEncoder):
    def default(self, o):
        return o.__dict__


class AlignmentEncoder(JSONEncoder):
    def default(self, o):
        return o.__dict__


class LinkEncoder(JSONEncoder):
    def default(self, o):
        return {
            "from": o.from_,
            "from_orient": o.from_orient,
            "to": o.to,
            "to_orient": o.to_orient,
            "overlap": str(o.overlap)
        }


class PathEncoder(JSONEncoder):
    def default(self, o):
        items = str(o).split("\t")
        return {
            "segments": items[2],
            "overlaps": items[3]
        }


def simple_dump(graph):
    print(json.dumps(graph.headers, indent=4))
    print(json.dumps(graph.segments, indent=4, cls=SegmentEncoder))
    print(json.dumps(graph.links, indent=4, cls=LinkEncoder))
    print(json.dumps(graph.paths, indent=4, cls=PathEncoder))


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    simple_dump(graph)
