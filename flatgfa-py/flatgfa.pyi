from collections.abc import Iterator
from typing import Optional, overload

class Segment:
    id: int
    name: int

    def sequence(self) -> bytes: ...
    def __len__(self) -> int: ...

class Handle:
    seg_id: int
    segment: Segment
    is_forward: bool

class StepList:
    def __iter__(self) -> Iterator[Handle]: ...
    def __len__(self) -> int: ...
    @overload
    def __getitem__(self, idx: int) -> Handle: ...
    @overload
    def __getitem__(self, slice: slice) -> StepList: ...

class Path:
    id: int
    name: bytes

    def __iter__(self) -> Iterator[Handle]: ...
    @overload
    def __getitem__(self, idx: int) -> Handle: ...
    @overload
    def __getitem__(self, slice: slice) -> StepList: ...

class Link:
    id: int
    from_: Handle
    to: Handle

class SegmentList:
    @overload
    def __getitem__(self, idx: int) -> Segment: ...
    @overload
    def __getitem__(self, slice: slice) -> SegmentList: ...
    def __iter__(self) -> Iterator[Segment]: ...
    def __len__(self) -> int: ...
    def find(self, name: int) -> Optional[Segment]: ...

class PathList:
    @overload
    def __getitem__(self, idx: int) -> Path: ...
    @overload
    def __getitem__(self, slice: slice) -> PathList: ...
    def __iter__(self) -> Iterator[Path]: ...
    def __len__(self) -> int: ...
    def find(self, name: bytes) -> Optional[Path]: ...

class LinkList:
    @overload
    def __getitem__(self, idx: int) -> Link: ...
    @overload
    def __getitem__(self, slice: slice) -> LinkList: ...
    def __iter__(self) -> Iterator[Link]: ...
    def __len__(self) -> int: ...

class ChunkEvent:
    handle: Handle
    range: tuple[int, int]
    def sequence(self) -> str: ...

class GAFLine:
    name: str
    chunks: list[ChunkEvent]
    def segment_ranges(self) -> str: ...
    def sequence(self) -> str: ...
    def __iter__(self) -> Iterator[ChunkEvent]: ...

class GAFParser:
    def __iter__(self) -> Iterator[GAFLine]: ...

class FlatGFA:
    segments: SegmentList
    paths: PathList
    links: LinkList

    def write_flatgfa(self, filename: str) -> None: ...
    def write_gfa(self, filename: str) -> None: ...
    def all_reads(self, gaf: str) -> GAFParser: ...
    def print_gaf_lookup(self, gaf: str) -> None: ...

def parse(filename: str) -> FlatGFA: ...
def load(filename: str) -> FlatGFA: ...
def parse_bytes(gfa: bytes) -> FlatGFA: ...
