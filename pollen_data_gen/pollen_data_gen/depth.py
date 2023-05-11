import json
from typing import Any, Collection, Dict, Union
from json import JSONEncoder
from mygfa import mygfa, preprocess


FormatType = Dict[str, Union[bool, str, int]]
OutputType = Dict[str, Dict[str, Collection[object]]]


def format_gen(width: int) -> FormatType:
    """Generates a format object for a bitvector of length `width`."""
    return {"is_signed": False, "numeric_type": "bitnum", "width": width}


def paths_viewed_from_nodes(
    graph: mygfa.Graph, max_n: int, max_e: int, max_p: int
) -> OutputType:
    """Given a graph, return a dict representing the paths
    viewed from the PoV of each node.
    """
    path2id = {path: id for id, path in enumerate(graph.paths, start=1)}
    output = {}
    json_format = format_gen(max_p.bit_length())
    for seg, crossings in preprocess.node_steps(graph).items():
        data = list(path2id[c[0]] for c in crossings)
        data = data + [0] * (max_e - len(data))
        output[f"path_ids{seg}"] = {"data": data, "format": json_format}
    # I would rather not have the for-loop below. See issue 24
    data = [0] * max_e
    for i in range(len(graph.segments) + 1, max_n + 1):
        output[f"path_ids{i}"] = {"data": data, "format": json_format}
    return output


def paths_to_consider(max_n: int, max_p: int) -> OutputType:
    """Currently just a stub; later we will populate this with a
    bitvector of length MAX_PATHS, where the i'th index will be 1 if
    the i'th path is to be considered during depth calculation.

    Somewhat annoyingly, we need as many copies of this bitvector as there
    are nodes in the graph.
    """
    output = {}
    for i in range(1, max_n + 1):
        # Would rather do the above for size(g). See issue 24
        data = [0] + [1] * (max_p)
        output[f"paths_to_consider{i}"] = {"data": data, "format": format_gen(1)}
    return output


class NodeDepthEncoder(JSONEncoder):
    """Encodes the entire graph as a JSON object, for the purpose of node depth.

    The exine command `depth` is the oracle for this encoding.
    """

    def __init__(self, max_n: int, max_e: int, max_p: int, **kwargs: Any) -> None:
        super(NodeDepthEncoder, self).__init__(**kwargs)
        self.max_n = max_n
        self.max_e = max_e
        self.max_p = max_p

    def default(self, o: Any) -> None:
        # This prints the word "null" after everything else is done,
        # which I think is because the graph has some field that
        # we do not yet encode nicely.
        answer_field = {
            "depth_output": {
                "data": list([0] * self.max_n),
                "format": format_gen(self.max_e.bit_length()),
            }
        }
        answer_field_uniq = {
            "uniq_output": {
                "data": list([0] * self.max_n),
                "format": format_gen(self.max_p.bit_length()),
            }
        }
        paths = paths_viewed_from_nodes(
            o, self.max_n, self.max_e, self.max_p
        ) | paths_to_consider(self.max_n, self.max_p)
        print(
            json.dumps(
                answer_field | paths | answer_field_uniq, indent=2, sort_keys=True
            )
        )


def depth(graph: mygfa.Graph, max_n: int, max_e: int, max_p: int) -> None:
    """Prints a JSON representation of `graph`
    that is specific to the exine command `depth`.
    """
    n_tight, e_tight, p_tight = preprocess.get_maxes(graph)
    # These values have been calculated automatically, and are likely optimal.
    # However, they are only to be used when the user-does not supply them via CLI.
    if not max_n:
        max_n = n_tight
    if not max_e:
        max_e = e_tight
    if not max_p:
        max_p = p_tight

    NodeDepthEncoder(max_n=int(max_n), max_e=int(max_e), max_p=int(max_p)).encode(graph)
