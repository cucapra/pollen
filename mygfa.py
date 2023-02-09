import sys
from dataclasses import dataclass
from typing import List, Tuple, Optional, Dict, TextIO, Iterator
from enum import Enum
import re


def parse_orient(s) -> bool:
    """Parse an orientation string as a bool.

    In our convention, "True" is forward and "False" is reverse.
    """
    assert s in ('+', '-')
    return s == '+'


@dataclass
class Segment:
    """A GFA segment is nucleotide sequence."""
    name: str
    seq: str

    @classmethod
    def parse(cls, fields: List[str]) -> "Segment":
        _, name, seq = fields[:3]
        return Segment(name, seq)


class AlignOp(Enum):
    """An operator in an Alignment."""
    MATCH = 'M'
    GAP = 'N'
    DELETION = 'D'
    INSERTION = 'I'


@dataclass
class Alignment:
    """CIGAR representation of a sequence alignment."""
    ops: List[Tuple[int, AlignOp]]

    @classmethod
    def parse(cls, s: str) -> "Alignment":
        """Parse a CIGAR string, which looks like 3M7N4M."""
        ops = [
            (int(amount_str), AlignOp(op_str))
            for amount_str, op_str in re.findall(r'(\d+)([^\d])', s)
        ]
        return Alignment(ops)

    def __str__(self):
        return ''.join(
            f'{amount}{op.value}' for (amount, op) in self.ops
        )


@dataclass
class Link:
    """A GFA link is an edge connecting two sequences."""
    from_: str  # The name of a segment.
    from_orient: bool
    to: str  # Also a segment name.
    to_orient: bool
    overlap: str  # "CIGAR string."

    @classmethod
    def parse(cls, fields: List[str]) -> "Link":
        _, from_, from_orient, to, to_orient, overlap = fields[:6]
        return Link(
            from_,
            parse_orient(from_orient),
            to,
            parse_orient(to_orient),
            overlap,
        )


@dataclass
class Path:
    """A GFA path is an ordered series of links."""
    name: str
    segments: List[Tuple[str, bool]]  # Segment names and orientations.
    overlaps: Optional[List[str]]  # "CIGAR strings."

    @classmethod
    def parse(cls, fields: List[str]) -> "Path":
        _, name, seq, overlaps = fields[:4]

        seq_lst = [(s[:-1], parse_orient(s[-1])) for s in seq.split(',')]
        overlaps_lst = None if overlaps == '*' else overlaps.split(',')
        if overlaps_lst:
            # I'm not sure yet why there can sometimes be one fewer
            # overlaps than sequences.
            assert len(overlaps_lst) in (len(seq_lst), len(seq_lst) - 1)

        return Path(
            name,
            seq_lst,
            overlaps_lst,
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
    segments: Dict[str, Segment]
    links: List[Link]
    paths: Dict[str, Path]

    @classmethod
    def parse(cls, infile: TextIO) -> "Graph":
        graph = Graph({}, [], {})

        for line in nonblanks(infile):
            fields = line.split()
            if fields[0] == 'H':
                pass  # Ignore headers for now.
            elif fields[0] == 'S':
                segment = Segment.parse(fields)
                graph.segments[segment.name] = segment
            elif fields[0] == 'L':
                graph.links.append(Link.parse(fields))
            elif fields[0] == 'P':
                path = Path.parse(fields)
                graph.paths[path.name] = path
            else:
                assert False, f"unknown line marker {fields[0]}"

        return graph


if __name__ == "__main__":
    graph = Graph.parse(sys.stdin)
    print(graph)
