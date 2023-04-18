import sys
from dataclasses import dataclass
from typing import List, Tuple, Optional, Dict, TextIO, Iterator
from enum import Enum
import re


def parse_orient(o) -> bool:
    """Parse an orientation string as a bool.
    In our convention, "True" is forward and "False" is reverse.
    """
    assert o in ("+", "-")
    return o == "+"


@dataclass
class Segment:
    """A GFA segment is nucleotide sequence."""

    name: str
    seq: str

    @classmethod
    def parse(cls, fields: List[str]) -> "Segment":
        _, name, seq = fields[:3]
        return Segment(name, seq)

    def __str__(self):
        return "\t".join(
            [
                "S",
                self.name,
                self.seq,
            ]
        )


class AlignOp(Enum):
    """An operator in an Alignment."""

    MATCH = "M"
    GAP = "N"
    DELETION = "D"
    INSERTION = "I"


@dataclass
class Alignment:
    """CIGAR representation of a sequence alignment."""

    ops: List[Tuple[int, AlignOp]]

    @classmethod
    def parse(cls, s: str) -> "Alignment":
        """Parse a CIGAR string, which looks like 3M7N4M."""
        ops = [
            (int(amount_str), AlignOp(op_str))
            for amount_str, op_str in re.findall(r"(\d+)([^\d])", s)
        ]
        return Alignment(ops)

    def __str__(self):
        return "".join(f"{amount}{op.value}" for (amount, op) in self.ops)


@dataclass(eq=True, frozen=True, order=True)
class Handle:
    """A specific orientation for a segment, referenced by name."""

    name: str
    orientation: bool

    @classmethod
    def parse(cls, s, o) -> "Handle":
        return Handle(s, parse_orient(o))

    def rev(self) -> "Handle":
        return Handle(self.name, not self.orientation)

    """We need two str methods because Links and Paths
    have different preferences when converting Handles to string
    """

    def __str__(self):  # This is what a path wants.
        return "".join([self.name, ("+" if self.orientation else "-")])

    def linkstr(self):  # While this is what a link wants.
        return "\t".join([self.name, ("+" if self.orientation else "-")])


@dataclass(eq=True, order=True)
class Link:
    """A GFA link is an edge connecting two sequences."""

    from_: Handle
    to: Handle
    overlap: Alignment

    @classmethod
    def parse(cls, fields: List[str]) -> "Link":
        _, from_, from_orient, to, to_orient, overlap = fields[:6]
        return Link(
            Handle.parse(from_, from_orient),
            Handle.parse(to, to_orient),
            Alignment.parse(overlap),
        )

    def rev(self) -> "Link":
        return Link(self.to.rev(), self.from_.rev(), self.overlap)

    def __str__(self):
        return "\t".join(
            [
                "L",
                self.from_.linkstr(),
                self.to.linkstr(),
                str(self.overlap),
            ]
        )


@dataclass
class Path:
    """A GFA path is an ordered series of links."""

    name: str
    segments: List[Handle]  # Segment names and orientations.
    overlaps: Optional[List[Alignment]]

    @classmethod
    def parse(cls, fields: List[str]) -> "Path":
        _, name, seq, overlaps = fields[:4]
        seq_lst = [Handle.parse(s[:-1], s[-1]) for s in seq.split(",")]
        overlaps_lst = (
            None
            if overlaps == "*"
            else [Alignment.parse(s) for s in overlaps.split(",")]
        )
        if overlaps_lst:
            # I'm not sure yet why there can sometimes be one fewer
            # overlaps than sequences.
            assert len(overlaps_lst) in (len(seq_lst), len(seq_lst) - 1)

        return Path(
            name,
            seq_lst,
            overlaps_lst,
        )

    def __str__(self):
        return "\t".join(
            ["P", self.name, ",".join(str(seg) for seg in self.segments), "*"]
        )


def nonblanks(f: TextIO) -> Iterator[str]:
    """Generate trimmed, nonempty lines from a text file."""
    for line in f:
        line = line.strip()
        if line:
            yield line


@dataclass
class Graph:
    """An entire GFA file."""

    headers: List[str]
    segments: Dict[str, Segment]
    links: List[Link]
    paths: Dict[str, Path]

    @classmethod
    def parse(cls, infile: TextIO) -> "Graph":
        graph = Graph([], {}, [], {})

        for line in nonblanks(infile):
            fields = line.split()
            if fields[0] == "H":
                graph.headers.append(line)  # Parse headers verbatim.
            elif fields[0] == "S":
                segment = Segment.parse(fields)
                graph.segments[segment.name] = segment
            elif fields[0] == "L":
                graph.links.append(Link.parse(fields))
            elif fields[0] == "P":
                path = Path.parse(fields)
                graph.paths[path.name] = path
            else:
                assert False, f"unknown line marker {fields[0]}"

        return graph

    def emit(self, outfile: TextIO, showlinks=True):
        for header in self.headers:
            print(header, file=outfile)
        for segment in self.segments.values():
            print(str(segment), file=outfile)
        for path in self.paths.values():
            print(str(path), file=outfile)
        if showlinks:
            for link in sorted(self.links):
                print(str(link), file=outfile)


if __name__ == "__main__":
    graph = Graph.parse(sys.stdin)
    graph.emit(sys.stdout, "--nl" not in sys.argv[1:])
