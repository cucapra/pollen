import json
import dataclasses
from json import JSONEncoder
from mygfa import mygfa


class GenericSimpleEncoder(JSONEncoder):
    """A generic JSON encoder for mygfa graphs."""

    def default(self, o):
        if isinstance(o, mygfa.Path):
            items = str(o).split("\t")
            return {"segments": items[2], "overlaps": items[3]}
        if isinstance(o, mygfa.Link):
            return {
                "from": o.from_.name,
                "from_orient": o.from_.ori,
                "to": o.to_.name,
                "to_orient": o.to_.ori,
                "overlap": str(o.overlap),
            }
        if isinstance(o, mygfa.Segment) or isinstance(o, mygfa.Alignment):
            return dataclasses.asdict(o)


def simple(graph: mygfa.Graph) -> None:
    """Prints a "wholesale dump" JSON representation of `graph`"""
    print(json.dumps(graph.headers, indent=4))
    print(json.dumps(graph.segments, indent=4, cls=GenericSimpleEncoder))
    print(json.dumps(graph.links, indent=4, cls=GenericSimpleEncoder))
    print(json.dumps(graph.paths, indent=4, cls=GenericSimpleEncoder))
