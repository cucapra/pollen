from collections.abc import Iterator
from typing import Optional


class Segment:
    id: int
    name: int

    def sequence(self) -> bytes: ...


class Handle:
    seg_id: int
    segment: Segment
    is_forward: bool


class Path:
    id: int
    name: bytes

    def __iter__(self) -> Iterator[Handle]: ...


class Link:
    id: int
    from_: Handle
    to: Handle


class SegmentList:
    def __getitem__(self, idx: int) -> Segment: ...
    def __iter__(self) -> Iterator[Segment]: ...
    def find(self, name: int) -> Optional[Segment]: ...


class PathList:
    def __getitem__(self, idx: int) -> Path: ...
    def __iter__(self) -> Iterator[Path]: ...
    def find(self, name: bytes) -> Optional[Path]: ...


class LinkList:
    def __getitem__(self, idx: int) -> Link: ...
    def __iter__(self) -> Iterator[Link]: ...


class FlatGFA:
    segments: SegmentList
    paths: PathList
    links: LinkList

    def write_flatgfa(self, filename: str) -> None: ...
    def write_gfa(self, filename: str) -> None: ...


def parse(filename: str) -> FlatGFA: ...
def load(filename: str) -> FlatGFA: ...
