import json
from json import JSONEncoder
from mygfa import preprocess


def format_gen(width):
    return {"is_signed": False, "numeric_type": "bitnum", "width": width}


def paths_viewed_from_nodes(graph, n, e, p):
    path2id = {path: id for id, path in enumerate(graph.paths, start=1)}
    output = {}
    format = format_gen(p.bit_length())
    for seg, crossings in preprocess.node_steps(graph).items():
        data = list(path2id[c[0]] for c in crossings)
        data = data + [0] * (e - len(data))
        output[f"path_ids{seg}"] = {"data": data, "format": format}
    # I would rather not have the forloop below. See issue 24
    data = [0] * e
    for i in range(len(graph.segments) + 1, n + 1):
        output[f"path_ids{i}"] = {"data": data, "format": format}
    return output


def paths_to_consider(n, p):
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
        output[f"paths_to_consider{i}"] = {"data": data, "format": format_gen(1)}
    return output


class NodeDepthEncoder(JSONEncoder):
    """Encodes the entire graph as a JSON object, for the purpose of node depth.

    The exine command `depth` is the oracle for this encoding.
    """

    def __init__(self, n, e, p, **kwargs):
        super(NodeDepthEncoder, self).__init__(**kwargs)
        self.n = n
        self.e = e
        self.p = p

    def default(self, o):
        # This prints the word "null" after everything else is done,
        # which I think is because the graph has some field that
        # we do not yet encode nicely.
        answer_field = {
            "depth_output": {
                "data": list([0] * self.n),
                "format": format_gen(self.e.bit_length()),
            }
        }
        answer_field_uniq = {
            "uniq_output": {
                "data": list([0] * self.n),
                "format": format_gen(self.p.bit_length()),
            }
        }
        paths = paths_viewed_from_nodes(o, self.n, self.e, self.p) | paths_to_consider(
            self.n, self.p
        )
        print(
            json.dumps(
                answer_field | paths | answer_field_uniq, indent=2, sort_keys=True
            )
        )


def depth(graph, n, e, p):
    """Prints a JSON representation of `graph`
    that is specific to the exine command `depth`.
    """
    n_tight, e_tight, p_tight = preprocess.get_maxes(graph)
    # These values have been calculated automatically, and are likely optimal.
    # However, they are only to be used when the user-does not supply them via CLI.
    if not n:
        n = n_tight
    if not e:
        e = e_tight
    if not p:
        p = p_tight

    NodeDepthEncoder(n=int(n), e=int(e), p=int(p)).encode(graph)
