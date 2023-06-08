import sys
from dataclasses import dataclass
from typing import List, Tuple, Optional, Dict, TextIO, Iterator
from enum import Enum
import re


def parse_orientation(ori: str) -> bool:
    """Parse an orientation string as a bool.
    In our convention, "True" is forward and "False" is reverse.
    """
    assert ori in ("+", "-")
    return ori == "+"


LegendType = Dict[str, Tuple[int, int]]
# A legend is a mapping from segment names to pairs of integers.


@dataclass
class Bed:
    """A BED (Browser Extensible Data) file describes regions of a genome.
    lo and hi tell us when to start and stop reading, and the name is
    the name that region should get.

    Used only by `inject` for now, which adds a fourth column for the
    new name of the injected path.
    """

    name: str
    low: int
    high: int
    new: str  # Used by `inject` to give the new path a name.
    # In the future, make `new` Optional.

    @classmethod
    def parse(cls, line: str) -> "Bed":
        """Parse a BED line."""
        name, low, high, new = line.split("\t")
        return Bed(name, int(low), int(high), new)

    def __str__(self) -> str:
        return "\t".join([self.name, str(self.low), str(self.high), self.new])


@dataclass
class Segment:
    """A GFA segment is nucleotide sequence."""

    name: str
    seq: str

    @classmethod
    def parse_inner(cls, name, seq) -> "Segment":
        """Parse a GFA segment, assuming that the name and sequence
        have already been extracted."""
        return Segment(name, seq)

    @classmethod
    def parse(cls, fields: List[str]) -> "Segment":
        """Parse a GFA segment."""
        _, name, seq = fields[:3]
        return cls.parse_inner(name, seq)

    def revcomp(self) -> "Segment":
        """Returns the reverse complement of this segment."""
        comp = {"A": "T", "C": "G", "G": "C", "T": "A"}
        seq = "".join(reversed([comp[c] for c in self.seq]))
        return Segment(self.name, seq)

    def __str__(self) -> str:
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

    ops: List[Tuple[int, AlignOp]]  # noqa

    @classmethod
    def parse(cls, cigar: str) -> "Alignment":
        """Parse a CIGAR string, which looks like 3M7N4M."""
        ops = [
            (int(amount_str), AlignOp(op_str))
            for amount_str, op_str in re.findall(r"(\d+)([^\d])", cigar)
        ]
        return Alignment(ops)

    def __str__(self) -> str:
        return "".join(f"{amount}{op.value}" for (amount, op) in self.ops)


@dataclass(eq=True, frozen=True, order=True)
class Handle:
    """A specific orientation for a segment, referenced by name."""

    name: str
    ori: bool

    @classmethod
    def parse(cls, seg: str, ori: str) -> "Handle":
        """Parse a Handle."""
        return Handle(seg, parse_orientation(ori))

    def rev(self) -> "Handle":
        """Return the handle representing the complement of this handle."""
        return Handle(self.name, not self.ori)

    def __str__(self) -> str:
        """This is how a path wants handles to be string-ified."""
        return "".join([self.name, ("+" if self.ori else "-")])

    def linkstr(self) -> str:
        """This is how a link wants handles to be string-ified."""
        return "\t".join([self.name, ("+" if self.ori else "-")])


@dataclass(eq=True, order=True)
class Link:
    """A GFA link is an edge connecting two sequences."""

    from_: Handle
    to_: Handle
    overlap: Alignment

    @classmethod
    def parse_inner(cls, from_, from_ori, to_, to_ori, overlap) -> "Link":
        """Parse a GFA link, assuming that the key elements have
        already been extracted.
        """
        return Link(
            Handle.parse(from_, from_ori),
            Handle.parse(to_, to_ori),
            Alignment.parse(overlap),
        )

    @classmethod
    def parse(cls, fields: List[str]) -> "Link":
        """Parse a GFA link."""
        _, from_, from_ori, to_, to_ori, overlap = fields[:6]
        return cls.parse_inner(from_, from_ori, to_, to_ori, overlap)

    def rev(self) -> "Link":
        """Return the link representing the reverse of this link.
        i.e, `AAAA --> GGGG` becomes `TTTT <-- CCCC`
        """
        return Link(self.to_.rev(), self.from_.rev(), self.overlap)

    def __str__(self) -> str:
        return "\t".join(
            [
                "L",
                self.from_.linkstr(),
                self.to_.linkstr(),
                str(self.overlap),
            ]
        )


@dataclass
class Path:
    """A GFA path is an ordered series of links."""

    name: str
    segments: List[Handle]  # Segment names and orientations.
    olaps: Optional[List[Alignment]]

    @classmethod
    def parse_inner(cls, name, seq: str, overlaps: str) -> "Path":
        """Parse a GFA path, assuming that
        the name, sequence and overlaps have already been extracted."""

        seq_lst = [Handle.parse(s[:-1], s[-1]) for s in seq.split(",")]
        olaps_lst = (
            None
            if overlaps == "*"
            else [Alignment.parse(s) for s in overlaps.split(",")]
        )
        if olaps_lst:
            # I'm not sure yet why there can sometimes be one fewer
            # overlaps than sequences.
            assert len(olaps_lst) in (len(seq_lst), len(seq_lst) - 1)

        return Path(
            name,
            seq_lst,
            olaps_lst,
        )

    @classmethod
    def parse(cls, fields: List[str]) -> "Path":
        """Parse a GFA path.
        Extract the name, seq, and overlaps, and dispatch to the helper above."""
        _, name, seq, overlaps = fields[:4]
        return cls.parse_inner(name, seq, overlaps)

    def drop_overlaps(self) -> "Path":
        """Return a copy of this path without overlaps."""
        return Path(self.name, self.segments, None)

    def __str__(self) -> str:
        return "\t".join(
            [
                "P",
                self.name,
                ",".join(str(seg) for seg in self.segments),
                ",".join(str(a) for a in self.olaps) if self.olaps else "*",
            ]
        )


def nonblanks(file: TextIO) -> Iterator[str]:
    """Generate trimmed, nonempty lines from a text file."""
    for line in file:
        line = line.strip()
        if line:
            yield line


@dataclass
class Header:
    """A GFA header."""

    header: str

    @classmethod
    def parse(cls, line: str) -> "Header":
        """Parse a GFA header."""
        return Header(line)

    def __str__(self) -> str:
        return self.header


@dataclass
class Graph:
    """An entire GFA file."""

    headers: List[Header]
    segments: Dict[str, Segment]
    links: List[Link]
    paths: Dict[str, Path]

    @classmethod
    def parse(cls, infile: TextIO) -> "Graph":
        """Parse a GFA file."""
        graph = Graph([], {}, [], {})

        for line in nonblanks(infile):
            fields = line.split()
            if fields[0] == "H":
                graph.headers.append(Header.parse(line))
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

    def emit(self, outfile: TextIO, showlinks: bool = True) -> None:
        """Emit a GFA file."""
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
    mygraph = Graph.parse(sys.stdin)
    if len(sys.argv) > 1 and sys.argv[1] == "--nl":
        mygraph.emit(sys.stdout, False)
    else:
        mygraph.emit(sys.stdout)
