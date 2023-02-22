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

    """Compact any "runs" of N down to a single N."""

    def crush_n(self) -> "Segment":
        seq = ""  # the crushed sequence will be built up here
        in_n = False
        for char in self.seq:
            if char == 'N':
                if in_n:
                    continue
                else:
                    in_n = True
            else:
                in_n = False
            seq += char
        return Segment(self.name, seq)

    def __str__(self):
        return '\t'.join([
            "S",
            self.name,
            self.seq,
        ])


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
    overlap: Alignment

    @classmethod
    def parse(cls, fields: List[str]) -> "Link":
        _, from_, from_orient, to, to_orient, overlap = fields[:6]
        return Link(
            from_,
            parse_orient(from_orient),
            to,
            parse_orient(to_orient),
            Alignment.parse(overlap),
        )

    def __str__(self):
        return '\t'.join([
            "L",
            self.from_,
            "+" if self.from_orient else "-",
            self.to,
            "+" if self.to_orient else "-",
            str(self.overlap),
        ])


@dataclass
class Path:
    """A GFA path is an ordered series of links."""
    name: str
    segments: List[Tuple[str, bool]]  # Segment names and orientations.
    overlaps: Optional[List[Alignment]]

    @classmethod
    def parse(cls, fields: List[str]) -> "Path":
        _, name, seq, overlaps = fields[:4]

        seq_lst = [(s[:-1], parse_orient(s[-1])) for s in seq.split(',')]
        overlaps_lst = None if overlaps == '*' else \
            [Alignment.parse(s) for s in overlaps.split(',')]
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
        return '\t'.join([
            "P",
            self.name,
            ",".join(f"{n}{'+' if o else '-'}" for (n, o) in self.segments),
            ",".join(str(a) for a in self.overlaps) if self.overlaps else "*",
        ])


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

    def crush_n(self) -> "Graph":
        crushed_segments = \
            {name: seg.crush_n() for name, seg in self.segments.items()}
        return Graph(crushed_segments, self.links, self.paths)

    def emit(self, outfile: TextIO):
        for segment in self.segments.values():
            print(str(segment), file=outfile)
        for link in self.links:
            print(str(link), file=outfile)
        for path in self.paths.values():
            print(str(path), file=outfile)


def node_steps(graph):
    """For each segment in the graph,
       list the times the segment was crossed by a path"""

    # segment name, (path name, index on path, direction) list
    crossings: Dict[str, List[Tuple[str, int, bool]]] = {}
    for segment in graph.segments.values():
        crossings[segment.name] = []

    for path in graph.paths.values():
        for id, (seg_name, seg_orient) in enumerate(path.segments):
            crossings[seg_name].append((path.name, id, seg_orient))

    return crossings


def node_depth(graph):
    # Here I show that node_depth is just the cardinality of the above,
    # again as observed in the note.
    for (segment, crossings) in node_steps(graph).items():
        print('\t'.join([segment, str(len(crossings))]))


if __name__ == "__main__":
    graph = Graph.parse(sys.stdin)
    graph.emit(sys.stdout)
