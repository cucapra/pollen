# import sys
import json
from typing import Dict, Union, Optional, Any, List
from json import JSONEncoder
from mygfa import mygfa


SimpleType = Optional[Dict[str, Union[bool, str, int]]]

char_to_number = {"A": 1, "T": 2, "G": 3, "C": 4, "N": 5}
number_to_char = {v: k for k, v in char_to_number.items()}


def strand_to_number_list(strand: str):
    """Converts a strand to a list of numbers following the mapping above.
    For instance, "AGGA" is converted to [1,3,3,1].
    """
    return [char_to_number[c] for c in strand]


def number_list_to_strand(numbers: List[str]):
    """Converts a list of numbers to a strand following the mapping above.
    For instance, [1,3,3,1] is converted to "AGGA"."""
    return "".join([number_to_char[number] for number in numbers])


def path_seq_to_number_list(path: str):
    """Converts a path's segment sequence into a list of numbers.
    Every + becomes 0 and - becomes 1.
    For instance, "1+,2-,14+" is converted to [1,0,2,1,14,0].
    The 1 at the 4th cell will not be confused for a node called "1" because
    it is at an even index.
    It's ugly but it works for now...
    """
    ans = []
    for chunk in path.split(","):
        num, orient = chunk[:-1], chunk[-1]
        ans.append(int(num))
        if orient == "+":
            ans.append(0)
        else:
            ans.append(1)

    return ans


def number_list_to_path_seq(numbers):
    """The inverse of the above function."""
    ans = []
    for i, number in enumerate(numbers):
        if i % 2:
            if number == 0:
                ans.append("+,")
            elif number == 1:
                ans.append("-,")
        else:
            ans.append(str(number))

    # Need to drop the last comma.
    return "".join(ans)[:-1]


def align_to_str(align: mygfa.Alignment):
    """Placeholder until we have reason to do anything cleverer."""
    return str(align)


def str_to_align(align_str: str):
    """Placeholder until we have reason to do anything cleverer."""
    return mygfa.Alignment.parse(align_str)


def link_to_number_list(link: mygfa.Link):
    """Converts a Link object to a list of four numbers and a string.
    As before, every + becomes 0 and - becomes 1."""
    return [
        int(link.from_.name),
        0 if link.from_.ori else 1,
        int(link.to_.name),
        0 if link.to_.ori else 1,
        align_to_str(link.overlap),
    ]


def number_list_to_link(link_json) -> mygfa.Link:
    """The inverse of the above function."""
    return mygfa.Link(
        mygfa.Handle(str(link_json[0]), link_json[1] == 0),
        mygfa.Handle(str(link_json[2]), link_json[3] == 0),
        str_to_align(link_json[4]),
    )


class GenericSimpleEncoder(JSONEncoder):
    """A generic JSON encoder for mygfa graphs."""

    def default(self, o: Any) -> SimpleType:
        if isinstance(o, mygfa.Path):
            items = str(o).split("\t")
            # We can drop the 0th cell, which will just be 'P',
            # and the 1st cell, which will just be the path's name.
            # Not doing anything clever with the overlaps yet.
            return {"segments": path_seq_to_number_list(items[2]), "overlaps": items[3]}
        if isinstance(o, mygfa.Link):
            return link_to_number_list(o)
        if isinstance(o, mygfa.Header):
            return str(o)
        if isinstance(o, mygfa.Segment):
            return strand_to_number_list(o.seq)
        return None


def dump(graph: mygfa.Graph, json_file: str) -> None:
    """Outputs the graph as a JSON, with some redundant information removed."""
    with open(json_file, "w", encoding="utf-8") as file:
        json.dump(
            {"headers": graph.headers}
            | {f"seg_to_seq_{k}": v for k, v in graph.segments.items()}
            | {"links": graph.links}
            | {f"path_details_{k}": v for k, v in graph.paths.items()},
            file,
            indent=2,
            cls=GenericSimpleEncoder,
        )


def parse(json_file: str) -> mygfa.Graph:
    """Reads a JSON file and returns a mygfa.Graph object."""
    with open(json_file, "r", encoding="utf-8") as file:
        graph = json.load(file)
    graph_gfa = mygfa.Graph(
        [mygfa.Header.parse(h) for h in graph["headers"]],
        {
            k.split("_")[3]: mygfa.Segment.parse_inner(
                k.split("_")[3], number_list_to_strand(v)
            )
            for k, v in graph.items()
            if k.startswith("seg_to_seq_")
        },
        [number_list_to_link(link) for link in graph["links"]],
        {
            k.split("_")[2]: mygfa.Path.parse_inner(
                k.split("_")[2], number_list_to_path_seq(v["segments"]), v["overlaps"]
            )
            for k, v in graph.items()
            if k.startswith("path_details_")
        },
    )
    # graph_gfa.emit(sys.stdout)  # Good for debugging.
    return graph_gfa


def roundtrip_test(graph: mygfa.Graph) -> None:
    """Tests that the graph can be serialized and deserialized."""
    dump(graph, "roundtrip_test.json")
    assert parse("roundtrip_test.json") == graph
