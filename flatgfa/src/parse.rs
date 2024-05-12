use crate::flatgfa::{self, Handle, LineKind, Orientation};
use crate::gfaline;
use std::collections::HashMap;
use std::io::BufRead;

pub struct Parser<'a, P: flatgfa::StoreFamily<'a>> {
    /// The flat representation we're building.
    flat: flatgfa::GFAStore<'a, P>,

    /// All segment IDs, indexed by their names, which we need to refer to segments in paths.
    seg_ids: NameMap,
}

impl<'a, P: flatgfa::StoreFamily<'a>> Parser<'a, P> {
    pub fn new(builder: flatgfa::GFAStore<'a, P>) -> Self {
        Self {
            flat: builder,
            seg_ids: NameMap::default(),
        }
    }

    /// Parse a GFA text file from an I/O stream.
    pub fn parse_stream<R: BufRead>(mut self, stream: R) -> flatgfa::GFAStore<'a, P> {
        // We can parse sements immediately, but we need to defer links and paths until we have all
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
            self.add_path(&line);
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
            if line[0] == b'P' {
                self.add_path(line);
            } else {
                let gfa_line = gfaline::parse_line(line).unwrap();
                if let gfaline::Line::Link(link) = gfa_line {
                    self.add_link(link);
                } else {
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
        self.seg_ids.insert(seg.name, seg_id.into());
    }

    fn add_link(&mut self, link: gfaline::Link) {
        let from = Handle::new(self.seg_ids.get(link.from_seg).into(), link.from_orient);
        let to = Handle::new(self.seg_ids.get(link.to_seg).into(), link.to_orient);
        self.flat.add_link(from, to, link.overlap);
    }

    fn add_path(&mut self, line: &[u8]) {
        // This must be a path line.
        assert_eq!(&line[..2], b"P\t");
        let line = &line[2..];

        // Parse the name.
        let (name, rest) = gfaline::parse_field(line).unwrap();

        // Parse the steps.
        let mut step_parser = gfaline::StepsParser::new(rest);
        let steps = self.flat.add_steps((&mut step_parser).map(|(name, dir)| {
            Handle::new(
                self.seg_ids.get(name).into(),
                if dir {
                    Orientation::Forward
                } else {
                    Orientation::Backward
                },
            )
        }));
        let rest = step_parser.rest();

        // Parse the overlaps.
        let (overlaps, rest) = gfaline::parse_maybe_overlap_list(rest).unwrap();

        assert!(rest.is_empty());
        self.flat.add_path(name, steps, overlaps.into_iter());
    }
}

impl Parser<'static, flatgfa::HeapFamily> {
    pub fn for_heap() -> Self {
        Self::new(flatgfa::HeapGFAStore::default())
    }
}

impl<'a> Parser<'a, flatgfa::FixedFamily> {
    pub fn for_slice(store: flatgfa::FixedGFAStore<'a>) -> Self {
        Self::new(store)
    }
}

#[derive(Default)]
struct NameMap {
    /// Names at most this are assigned *sequential* IDs, i.e., the ID is just the name
    /// minus one.
    sequential_max: usize,

    /// Non-sequential names go here.
    others: HashMap<usize, u32>,
}

impl NameMap {
    fn insert(&mut self, name: usize, id: u32) {
        // Is this the next sequential name? If so, no need to record it in our hash table;
        // just bump the number of sequential names we've seen.
        if (name - 1) == self.sequential_max && (name - 1) == (id as usize) {
            self.sequential_max += 1;
        } else {
            self.others.insert(name, id);
        }
    }

    fn get(&self, name: usize) -> u32 {
        if name <= self.sequential_max {
            (name - 1) as u32
        } else {
            self.others[&name]
        }
    }
}

/// Scan a GFA text file to count the number of each type of line and measure some sizes
/// that are useful in estimating the final size of the FlatGFA file.
pub fn estimate_toc(buf: &[u8]) -> crate::file::Toc {
    let mut segs = 0;
    let mut links = 0;
    let mut paths = 0;
    let mut header_bytes = 0;
    let mut seg_bytes = 0;
    let mut path_bytes = 0;

    let mut rest = buf;
    while !rest.is_empty() {
        let marker = rest[0];
        let next = memchr::memchr(b'\n', rest).unwrap_or(rest.len() + 1);

        match marker {
            b'H' => {
                header_bytes += next;
            }
            b'S' => {
                segs += 1;
                seg_bytes += next;
            }
            b'L' => {
                links += 1;
            }
            b'P' => {
                paths += 1;
                path_bytes += next;
            }
            _ => {
                panic!("unknown line type")
            }
        }

        if next >= rest.len() {
            break;
        }
        rest = &rest[next + 1..];
    }

    crate::file::Toc::estimate(segs, links, paths, header_bytes, seg_bytes, path_bytes)
}

struct MemchrSplit<'a> {
    haystack: &'a [u8],
    memchr: memchr::Memchr<'a>,
    pos: usize,
}

impl<'a> Iterator for MemchrSplit<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos;
        let end = self.memchr.next()?;
        self.pos = end + 1;
        Some(&self.haystack[start..end])
    }
}

impl MemchrSplit<'_> {
    fn new(needle: u8, haystack: &[u8]) -> MemchrSplit {
        MemchrSplit {
            haystack,
            memchr: memchr::memchr_iter(needle, haystack),
            pos: 0,
        }
    }
}
