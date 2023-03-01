import sys
import json
from json import JSONEncoder
import mygfa
import preprocess


class SegmentEncoder(JSONEncoder):
    def default(self, o):
        return o.__dict__


MAX_STEPS = 15

format = {"is_signed": False,
          "numeric_type": "bitnum",
          "width": 4}


def paths_viewed_from_nodes(graph):
    path2id = {path: id for id, path in enumerate(graph.paths, start=1)}
    output = {}
    for (seg, crossings) in preprocess.node_steps(graph).items():
        data = list(path2id[c[0]] for c in crossings)
        data = data + [0] * (MAX_STEPS - len(data))
        output[f'path_ids{seg}'] = {"data": data, "format": format}
    return output


class NodeDepthEncoder(JSONEncoder):

    def default(self, o):
        answer_field = {"depth_output": {"data": list([0]*MAX_STEPS),
                                         "format": format}}
        paths = paths_viewed_from_nodes(o)
        print(json.dumps(answer_field | paths, indent=2))


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


def json_for_node_depth(graph):
    json.dumps(graph, indent=4, cls=NodeDepthEncoder)


if __name__ == "__main__":
    graph = mygfa.Graph.parse(sys.stdin)
    json_for_node_depth(graph)
