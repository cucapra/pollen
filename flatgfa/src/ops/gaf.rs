use crate::flatgfa;
use crate::memfile::MemchrSplit;
use crate::namemap::NameMap;
use bstr::BStr;
use rayon::iter::{plumbing::UnindexedConsumer, ParallelIterator};

pub struct GAFLineParser<'a> {
    buf: &'a [u8],
}

#[derive(Debug)]
pub struct GAFLine<'a> {
    pub name: &'a BStr,
    pub start: usize,
    pub end: usize,
    pub path: &'a [u8],
}

impl<'a> GAFLineParser<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    fn advance(&mut self, offset: usize) {
        self.buf = &self.buf[offset..];
    }

    fn next_field(&mut self) -> Option<&'a [u8]> {
        let end = memchr::memchr(b'\t', self.buf)?;
        let res = &self.buf[..end];
        self.advance(end + 1);
        Some(res)
    }

    fn skip_fields(&mut self, n: usize) -> Option<()> {
        for _ in 0..n {
            let end = memchr::memchr(b'\t', self.buf)?;
            self.advance(end + 1);
        }
        Some(())
    }

    fn int_field(&mut self) -> Option<usize> {
        let mut pos = 0;
        let val = parse_int(&self.buf, &mut pos);
        assert!(matches!(self.buf[pos], b'\t' | b'\n'));
        self.advance(pos + 1);
        val
    }

    fn parse(&mut self) -> GAFLine<'a> {
        assert!(!self.buf.is_empty());

        let name = BStr::new(self.next_field().unwrap());
        self.skip_fields(4);

        // The actual path string (which we don't parse yet).
        let path = self.next_field().unwrap();

        // Get the read's coordinates.
        self.skip_fields(1); // Skip path length.
        let start: usize = self.int_field().unwrap();
        let end: usize = self.int_field().unwrap();

        GAFLine {
            name,
            start,
            end,
            path,
        }
    }
}

pub struct GAFParser<'a> {
    split: MemchrSplit<'a>,
}

impl<'a> GAFParser<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        let split = MemchrSplit::new(b'\n', buf);
        Self { split }
    }
}

impl<'a> Iterator for GAFParser<'a> {
    type Item = GAFLine<'a>;

    fn next(&mut self) -> Option<GAFLine<'a>> {
        let line = self.split.next()?;
        Some(GAFLineParser::new(line).parse())
    }
}

impl<'a> ParallelIterator for GAFParser<'a> {
    type Item = GAFLine<'a>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        ParallelIterator::map(self.split, |line| GAFLineParser::new(line).parse())
            .drive_unindexed(consumer)
    }
}

pub struct PathChunker<'a, 'b> {
    gfa: &'a flatgfa::FlatGFA<'a>,
    name_map: &'a NameMap,
    steps: PathParser<'b>,
    start: usize,
    end: usize,

    // State for the walk.
    index: usize,
    pos: usize,
    started: bool,
    ended: bool,
}

impl<'a, 'b> PathChunker<'a, 'b> {
    pub fn new(gfa: &'a flatgfa::FlatGFA, name_map: &'a NameMap, read: GAFLine<'b>) -> Self {
        let steps = PathParser::new(read.path);
        Self {
            gfa,
            name_map,
            steps,
            start: read.start,
            end: read.end,
            index: 0,
            pos: 0,
            started: false,
            ended: false,
        }
    }
}

#[derive(Debug)]
pub struct ChunkEvent {
    index: usize,
    handle: flatgfa::Handle,
    range: ChunkRange,
}

#[derive(Debug)]
enum ChunkRange {
    None,
    All,
    Partial(usize, usize),
}

impl ChunkEvent {
    pub fn print(&self, gfa: &flatgfa::FlatGFA) {
        let seg = gfa.segs[self.handle.segment()];
        let seg_name = seg.name;
        match self.range {
            ChunkRange::Partial(start, end) => {
                println!(
                    "{}: {}{}, {}-{}bp",
                    self.index,
                    seg_name,
                    self.handle.orient(),
                    start,
                    end,
                );
            }
            ChunkRange::All => {
                println!(
                    "{}: {}{}, {}bp",
                    self.index,
                    seg_name,
                    self.handle.orient(),
                    seg.len()
                );
            }
            ChunkRange::None => {
                println!("{}: (skipped)", self.index);
            }
        }
    }

    pub fn print_seq(&self, gfa: &flatgfa::FlatGFA) {
        let seq = gfa.get_seq_oriented(self.handle);

        match self.range {
            ChunkRange::Partial(start, end) => {
                print!("{}", &seq.slice(start..end));
            }
            ChunkRange::All => {
                print!("{}", seq);
            }
            ChunkRange::None => {}
        }
    }
}

impl<'a, 'b> Iterator for PathChunker<'a, 'b> {
    type Item = ChunkEvent;

    fn next(&mut self) -> Option<ChunkEvent> {
        let (seg_name, forward) = self.steps.next()?;

        // Get the corresponding handle from the GFA.
        let seg_id = self.name_map.get(seg_name);
        let dir = match forward {
            true => flatgfa::Orientation::Forward,
            false => flatgfa::Orientation::Backward,
        };
        let handle = flatgfa::Handle::new(seg_id, dir);

        // Accumulate the length to track our position in the path.
        let seg_len = self.gfa.segs[seg_id].len();
        let next_pos = self.pos + seg_len;
        let range = if !self.started && self.start < next_pos {
            self.started = true;
            if self.end < next_pos {
                // Also ending in the same segment.
                self.ended = true;
                ChunkRange::Partial(self.start - self.pos, self.end - self.pos)
            } else {
                // Just starting in this segment.
                ChunkRange::Partial(self.start - self.pos, seg_len)
            }
        } else if self.started && !self.ended && self.end < next_pos {
            // Just ending in this segment.
            self.ended = true;
            ChunkRange::Partial(0, self.end - self.pos)
        } else if self.started && !self.ended {
            ChunkRange::All
        } else {
            ChunkRange::None
        };
        self.pos = next_pos;

        // Produce the event.
        let out = ChunkEvent {
            handle,
            index: self.index,
            range,
        };
        self.index += 1;
        Some(out)
    }
}

/// Parse a GAF path string, which looks like >12<34>56.
struct PathParser<'a> {
    str: &'a [u8],
    index: usize,
}

impl<'a> PathParser<'a> {
    pub fn new(str: &'a [u8]) -> Self {
        Self { str, index: 0 }
    }

    #[allow(dead_code)]
    pub fn rest(&self) -> &[u8] {
        &self.str[self.index..]
    }
}

/// Parse an integer from a byte string starting at `index`. Update `index` to
/// point just past the parsed integer.
fn parse_int(bytes: &[u8], index: &mut usize) -> Option<usize> {
    let mut num = 0;
    let mut first_digit = true;

    while *index < bytes.len() {
        let byte = bytes[*index];
        if byte.is_ascii_digit() {
            num *= 10;
            num += (byte - b'0') as usize;
            *index += 1;
            first_digit = false;
        } else {
            break;
        }
    }

    if first_digit {
        return None;
    } else {
        return Some(num);
    }
}

impl<'a> Iterator for PathParser<'a> {
    type Item = (usize, bool);

    fn next(&mut self) -> Option<(usize, bool)> {
        if self.index >= self.str.len() {
            return None;
        }

        // The first character must be a direction.
        let byte = self.str[self.index];
        self.index += 1;
        let forward = match byte {
            b'>' => true,
            b'<' => false,
            _ => return None,
        };

        // Parse the integer segment name.
        let seg_name = parse_int(self.str, &mut self.index)?;
        return Some((seg_name, forward));
    }
}

#[test]
fn test_parse_gaf_path() {
    let s = b">12<34>5 suffix";
    let mut parser = PathParser::new(s);
    let path: Vec<_> = (&mut parser).collect();
    assert_eq!(path, vec![(12, true), (34, false), (5, true)]);
    assert_eq!(parser.rest(), b"suffix");
}
