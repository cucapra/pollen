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


def simple(graph: mygfa.Graph):
    """Prints a "wholesale dump" JSON representation of `graph`"""
    print(
        json.dumps(
            {"headers": graph.headers}
            | {"segments": graph.segments}
            | {"links": graph.links}
            | {"paths": graph.paths},
            indent=4,
            cls=GenericSimpleEncoder,
        )
    )
