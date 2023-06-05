import sys
import json
import dataclasses
from typing import Dict, Union, Optional, Any
from json import JSONEncoder
from mygfa import mygfa


SimpleType = Optional[Dict[str, Union[bool, str, int]]]


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
                "from_orient": o.from_.ori,
                "to": o.to_.name,
                "to_orient": o.to_.ori,
                "overlap": str(o.overlap),
            }
        if isinstance(o, mygfa.Header):
            # We can flatten the header objects into a simple list of strings.
            return str(o)
        if isinstance(o, (mygfa.Segment, mygfa.Alignment, mygfa.Link)):
            return dataclasses.asdict(o)
        return None


def dump(graph: mygfa.Graph, json_file: str) -> None:
    """Outputs the graph as a JSON, with some redundant information removed."""
    with open(json_file, "w", encoding="utf-8") as file:
        json.dump(
            {"headers": graph.headers}
            | {"segments": graph.segments}
            | {"links": graph.links}
            | {"paths": graph.paths},
            file,
            indent=2,
            cls=GenericSimpleEncoder,
        )


def parse(json_file: str) -> mygfa.Graph:
    """Reads a JSON file and returns a mygfa.Graph object."""
    with open(json_file, "r", encoding="utf-8") as file:
        graph = json.load(file)
    return mygfa.Graph(
        [mygfa.Header.parse(h) for h in graph["headers"]],
        {k: mygfa.Segment(v["name"], v["seq"]) for k, v in graph["segments"].items()},
        [
            mygfa.Link(
                mygfa.Handle.parse(l["from"], "+" if l["from_orient"] else "-"),
                mygfa.Handle.parse(l["to"], "+" if l["to_orient"] else "-"),
                mygfa.Alignment.parse(l["overlap"]),
            )
            for l in graph["links"]
        ],
        {
            k: mygfa.Path.parse_inner(k, v["segments"], v["overlaps"])
            for k, v in graph["paths"].items()
        },
    )


def roundtrip_test(graph: mygfa.Graph) -> None:
    """Tests that the graph can be serialized and deserialized."""
    dump(graph, "roundtrip_test.json")
    assert parse("roundtrip_test.json") == graph
