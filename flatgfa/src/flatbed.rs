use zerocopy::{AsBytes, FromBytes, FromZeroes};
use crate::flatgfa::{self, Handle, LineKind, Orientation};
use crate::memfile::MemchrSplit;
use crate::namemap::NameMap;
use std::io::BufRead;
use crate::pool::{Pool, Span, Store, HeapStore, FixedStore, Id};

#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct BEDEntry {
    pub name: Span<u8>,
    pub start: u64,
    pub end: u64,
}

#[derive(FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct FlatBED<'a> {
    pub name_data: Pool<'a, u8>,
    pub entries: Pool<'a, BEDEntry>,
}











/// The data storage pools for a `FlatBED`.
#[derive(Default)]
pub struct BEDStore<'a, P: StoreFamily<'a>> {
    pub name_data: P::Store<u8>,
    pub entries: P::Store<BEDEntry>,
}

impl<'a, P: StoreFamily<'a>> BEDStore<'a, P> {
    pub fn add_entry(&mut self, name: &[u8], start: u64, end: u64) -> Id<BEDEntry> {
        let name = self.name_data.add_slice(name);
        self.entries.add(BEDEntry {
            name,
            start,
            end,
        })
    }
}

pub trait StoreFamily<'a> {
    type Store<T: Clone + 'a>: Store<T>;
}

#[derive(Default)]
pub struct HeapFamily;
impl<'a> StoreFamily<'a> for HeapFamily {
    type Store<T: Clone + 'a> = HeapStore<T>;
}

pub struct FixedFamily;
impl<'a> StoreFamily<'a> for FixedFamily {
    type Store<T: Clone + 'a> = FixedStore<'a, T>;
}

/// A store for `FlatBED` data backed by fixed-size slices.
///
/// This store contains `SliceVec`s, which act like `Vec`s but are allocated within
/// a fixed region. This means they have a maximum size, but they can directly map
/// onto the contents of a file.
pub type FixedBEDStore<'a> = BEDStore<'a, FixedFamily>;

/// A mutable, in-memory data store for `FlatBED`.
///
/// This store contains a bunch of `Vec`s: one per array required to implement a
/// `FlatBED`. It exposes an API for building up a BED data structure, so it is
/// useful for creating new ones from scratch.
pub type HeapGFAStore = BEDStore<'static, HeapFamily>;




















pub struct BEDParser<'a, P: StoreFamily<'a>> {
    /// The flat representation we're building.
    flat: BEDStore<'a, P>,

    /// All segment IDs, indexed by their names, which we need to refer to segments in paths.
    seg_ids: NameMap,
}

impl<'a, P: StoreFamily<'a>> BEDParser<'a, P> {
    pub fn new(builder: BEDStore<'a, P>) -> Self {
        Self {
            flat: builder,
            seg_ids: NameMap::default(),
        }
    }

    /// Parse a GFA text file from an I/O stream.
    pub fn parse_stream<R: BufRead>(mut self, stream: R) -> BEDStore<'a, P> {
        // We can parse segments immediately, but we need to defer links and paths until we have all
        // the segment names that they might refer to.
        let mut deferred_links = Vec::new();
        let mut deferred_paths = Vec::new();

        // Parse or defer each line.
        for line in stream.split(b'\n') {
            let line = line.unwrap();

            // Avoid parsing paths entirely for now; just preserve the entire line for later.
            if line[0] == b'P' {
                self.flat.record_line(LineKind::Path);
                deferred_paths.push(line);
                continue;
            }

            // Parse other kinds of lines.
            let gfa_line = gfaline::parse_line(line.as_ref()).unwrap();
            self.record_line(&gfa_line);

            match gfa_line {
                gfaline::Line::Header(data) => {
                    self.flat.add_header(data);
                }
                gfaline::Line::Segment(seg) => {
                    self.add_seg(seg);
                }
                gfaline::Line::Link(link) => {
                    deferred_links.push(link);
                }
                gfaline::Line::Path(_) => {
                    unreachable!("paths handled separately")
                }
            }
        }

        // "Unwind" the deferred links and paths.
        for link in deferred_links {
            self.add_link(link);
        }
        for line in deferred_paths {
            if let gfaline::Line::Path(path) = gfaline::parse_line(&line).unwrap() {
                self.add_path(path);
            } else {
                unreachable!("unexpected deferred line")
            }
        }

        self.flat
    }

    /// Parse a GFA text file from an in-memory buffer.
    pub fn parse_mem(mut self, buf: &[u8]) -> flatgfa::GFAStore<'a, P> {
        let mut deferred_lines = Vec::new();

        for line in MemchrSplit::new(b'\n', buf) {
            // When parsing from memory, it's easy to entirely defer parsing of any line: we just keep
            // pointers to them. So we defer both paths and links.
            if line[0] == b'P' || line[0] == b'L' {
                self.flat.record_line(if line[0] == b'P' {
                    LineKind::Path
                } else {
                    LineKind::Link
                });
                deferred_lines.push(line);
                continue;
            }

            // Actually parse other lines.
            let gfa_line = gfaline::parse_line(line).unwrap();
            self.record_line(&gfa_line);
            match gfa_line {
                gfaline::Line::Header(data) => {
                    self.flat.add_header(data);
                }
                gfaline::Line::Segment(seg) => {
                    self.add_seg(seg);
                }
                gfaline::Line::Link(_) | gfaline::Line::Path(_) => {
                    unreachable!("paths and links handled separately")
                }
            }
        }

        // "Unwind" the deferred lines.
        for line in deferred_lines {
            let gfa_line = gfaline::parse_line(line).unwrap();
            match gfa_line {
                gfaline::Line::Link(link) => {
                    self.add_link(link);
                }
                gfaline::Line::Path(path) => {
                    self.add_path(path);
                }
                gfaline::Line::Header(_) | gfaline::Line::Segment(_) => {
                    unreachable!("unexpected deferred line")
                }
            }
        }

        self.flat
    }

    /// Record a marker that captures the original GFA line ordering.
    fn record_line(&mut self, line: &gfaline::Line) {
        match line {
            gfaline::Line::Header(_) => self.flat.record_line(LineKind::Header),
            gfaline::Line::Segment(_) => self.flat.record_line(LineKind::Segment),
            gfaline::Line::Link(_) => self.flat.record_line(LineKind::Link),
            gfaline::Line::Path(_) => self.flat.record_line(LineKind::Path),
        }
    }

    fn add_seg(&mut self, seg: gfaline::Segment) {
        let seg_id = self.flat.add_seg(seg.name, seg.seq, seg.data);
        self.seg_ids.insert(seg.name, seg_id);
    }

    fn add_link(&mut self, link: gfaline::Link) {
        let from = Handle::new(self.seg_ids.get(link.from_seg), link.from_orient);
        let to = Handle::new(self.seg_ids.get(link.to_seg), link.to_orient);
        self.flat.add_link(from, to, link.overlap);
    }

    fn add_path(&mut self, path: gfaline::Path) {
        // Parse the steps.
        let mut step_parser = gfaline::StepsParser::new(&path.steps);
        let steps = self.flat.add_steps((&mut step_parser).map(|(name, dir)| {
            Handle::new(
                self.seg_ids.get(name),
                if dir {
                    Orientation::Forward
                } else {
                    Orientation::Backward
                },
            )
        }));
        assert!(step_parser.rest().is_empty());

        self.flat
            .add_path(path.name, steps, path.overlaps.into_iter());
    }
}