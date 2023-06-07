import sys
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


class GenericSimpleEncoder(JSONEncoder):
    """A generic JSON encoder for mygfa graphs."""

    def default(self, o: Any) -> SimpleType:
        if isinstance(o, mygfa.Path):
            items = str(o).split("\t")
            # We can drop the 0th cell, which will just be 'P',
            # and the 1st cell, which will just be the path's name.
            return {"segments": items[2], "overlaps": items[3]}
        if isinstance(o, mygfa.Link):
            # We perform a little flattening.
            return {
                "from": o.from_.name,
                "from_orient": "+" if o.from_.ori else "-",
                "to": o.to_.name,
                "to_orient": "+" if o.to_.ori else "-",
                "overlap": str(o.overlap),
            }
        if isinstance(o, mygfa.Header):
            # We can flatten the header objects into a simple list of strings.
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
            | {"paths": graph.paths}
            | {"links": graph.links},
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
            (k.split("_")[3]): mygfa.Segment.parse_inner(
                k.split("_")[3], number_list_to_strand(v)
            )
            for k, v in graph.items()
            if k.startswith("seg_to_seq_")
        },
        [
            mygfa.Link.parse_inner(
                link["from"],
                link["from_orient"],
                link["to"],
                link["to_orient"],
                link["overlap"],
            )
            for link in graph["links"]
        ],
        {
            k: mygfa.Path.parse_inner(k, v["segments"], v["overlaps"])
            for k, v in graph["paths"].items()
        },
    )
    # graph_gfa.emit(sys.stdout)   # Good for debugging.
    return graph_gfa


def roundtrip_test(graph: mygfa.Graph) -> None:
    """Tests that the graph can be serialized and deserialized."""
    dump(graph, "roundtrip_test.json")
    assert parse("roundtrip_test.json") == graph
