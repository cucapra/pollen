import sys
import json
from json import JSONEncoder
from . import mygfa, preprocess


class SegmentEncoder(JSONEncoder):
    def default(self, o):
        return o.__dict__


MAX_STEPS = 15
MAX_NODES = 16

format_width4 = {"is_signed": False, "numeric_type": "bitnum", "width": 4}

format_width1 = {"is_signed": False, "numeric_type": "bitnum", "width": 1}


def paths_viewed_from_nodes(graph, n, e):
    path2id = {path: id for id, path in enumerate(graph.paths, start=1)}
    output = {}
    for seg, crossings in preprocess.node_steps(graph).items():
        data = list(path2id[c[0]] for c in crossings)
        data = data + [0] * (e - len(data))
        output[f"path_ids{seg}"] = {"data": data, "format": format_width4}
    # Would rather not have the four lines below. See issue 24
    data = [0] * e
    for i in range(len(graph.segments) + 1, n + 1):
        output[f"path_ids{i}"] = {"data": data, "format": format_width4}
    return output


def paths_to_consider(o, n):
    """Currently just a stub; later we will populate this with a
    bitvector of length MAX_PATHS, where the i'th index will be 1 if
    the i'th path is to be considered during depth calculation.

    Somewhat annoyingly, we need as many copies of this bitvector as there
    are nodes in the graph.
    """
    output = {}
    for i in range(1, n + 1):
        # Would rather do the above for size(g). See issue 24
        data = [0] + [1] * (
            MAX_NODES - 1
        )  # The fact that MAX_NODES, not n, is used here, is weird behavior from exine. matching for now.
        output[f"paths_to_consider{i}"] = {"data": data, "format": format_width1}
    return output


class NodeDepthEncoder(JSONEncoder):
    def __init__(self, n, e, **kwargs):
        super(NodeDepthEncoder, self).__init__(**kwargs)
        if n:
            self.n = n
        else:
            self.n = MAX_NODES
        if e:
            self.e = e
        else:
            self.e = MAX_STEPS

    def default(self, o):
        answer_field = {
            "depth_output": {"data": list([0] * self.n), "format": format_width4}
        }
        answer_field_uniq = {
            "uniq_output": {"data": list([0] * self.n), "format": format_width4}
        }
        paths = paths_viewed_from_nodes(o, self.n, self.e) | paths_to_consider(
            o, self.n
        )
        print(
            json.dumps(
                answer_field | paths | answer_field_uniq, indent=2, sort_keys=True
            )
        )


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
            "overlap": str(o.overlap),
        }


class PathEncoder(JSONEncoder):
    def default(self, o):
        items = str(o).split("\t")
        return {"segments": items[2], "overlaps": items[3]}


def simple_dump(graph):
    print(json.dumps(graph.headers, indent=4))
    print(json.dumps(graph.segments, indent=4, cls=SegmentEncoder))
    print(json.dumps(graph.links, indent=4, cls=LinkEncoder))
    print(json.dumps(graph.paths, indent=4, cls=PathEncoder))


def mkjson(graph, n, e):
    print(NodeDepthEncoder(n=int(n), e=int(e)).encode(graph))
