import sys
import json
import dataclasses
from json import JSONEncoder
from . import mygfa, preprocess

MAX_STEPS = 15
MAX_NODES = 16
MAX_PATHS = 15

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


def paths_to_consider(o, n, p):
    """Currently just a stub; later we will populate this with a
    bitvector of length MAX_PATHS, where the i'th index will be 1 if
    the i'th path is to be considered during depth calculation.

    Somewhat annoyingly, we need as many copies of this bitvector as there
    are nodes in the graph.
    """
    output = {}
    for i in range(1, n + 1):
        # Would rather do the above for size(g). See issue 24
        data = [0] + [1] * (p)
        output[f"paths_to_consider{i}"] = {"data": data, "format": format_width1}
    return output


class NodeDepthEncoder(JSONEncoder):
    """Encodes the entire graph as a JSON object, for the purpose of node depth.

    The exine command `depth` is the oracle for this encoding.
    """

    def __init__(self, n, e, p, **kwargs):
        super(NodeDepthEncoder, self).__init__(**kwargs)
        self.n = n if n else MAX_NODES
        self.e = e if e else MAX_STEPS
        self.p = p if p else MAX_PATHS

    def default(self, o):
        # This prints the word "null" after everything else is done,
        # which I think is because the graph has some field that
        # we do not yet encode nicely.
        answer_field = {
            "depth_output": {"data": list([0] * self.n), "format": format_width4}
        }
        answer_field_uniq = {
            "uniq_output": {"data": list([0] * self.n), "format": format_width4}
        }
        paths = paths_viewed_from_nodes(o, self.n, self.e) | paths_to_consider(
            o, self.n, self.p
        )
        print(
            json.dumps(
                answer_field | paths | answer_field_uniq, indent=2, sort_keys=True
            )
        )


class GenericSimpleEncoder(JSONEncoder):
    def default(self, o):
        if isinstance(o, mygfa.Path):
            items = str(o).split("\t")
            return {"segments": items[2], "overlaps": items[3]}
        elif isinstance(o, mygfa.Link):
            return {
                "from": o.from_.name,
                "from_orient": o.from_.orientation,
                "to": o.to.name,
                "to_orient": o.to.orientation,
                "overlap": str(o.overlap),
            }
        elif isinstance(o, mygfa.Segment) or isinstance(o, mygfa.Alignment):
            return dataclasses.asdict(o)


def simple_json(graph):
    """A wholesale dump of the graph, for completeness."""
    print(json.dumps(graph.headers, indent=4))
    print(json.dumps(graph.segments, indent=4, cls=GenericSimpleEncoder))
    print(json.dumps(graph.links, indent=4, cls=GenericSimpleEncoder))
    print(json.dumps(graph.paths, indent=4, cls=GenericSimpleEncoder))


def depth_json(graph, n, e, p):
    """Specific to the exine command `depth`."""
    print(NodeDepthEncoder(n=int(n), e=int(e), p=int(p)).encode(graph))
