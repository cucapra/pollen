use crate::flatgfa;
use crate::memfile::{map_file, MemchrSplit};
use argh::FromArgs;
use bstr::BStr;

/// look up positions from a GAF file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gaf")]
pub struct GAFLookup {
    /// path_name,offset,orientation
    #[argh(positional)]
    gaf: String,
}

pub fn gaf_lookup(gfa: &flatgfa::FlatGFA, args: GAFLookup) {
    // Read the lines in the GAF.
    let gaf_buf = map_file(&args.gaf);
    for line in MemchrSplit::new(b'\n', &gaf_buf) {
        let read = GAFLine::parse(line);
        println!("{}", read.name);

        for event in PathChunker::new(gfa, read) {
            let seg = gfa.segs[event.handle.segment()];
            let seg_name = seg.name;
            match event.range {
                ChunkRange::Partial(len) => {
                    println!(
                        "{}: {}{}, {}bp",
                        event.index,
                        seg_name,
                        event.handle.orient(),
                        len
                    );
                }
                ChunkRange::All => {
                    println!("{}: {}{}", event.index, seg_name, event.handle.orient());
                }
                ChunkRange::None => {
                    println!("{}: (skipped)", event.index);
                }
            }
        }
    }
}

struct GAFLine<'a> {
    name: &'a BStr,
    start: usize,
    end: usize,
    path: &'a [u8],
}

impl<'a> GAFLine<'a> {
    fn parse(line: &'a [u8]) -> Self {
        // Lines in a GAF are tab-separated.
        let mut field_iter = MemchrSplit::new(b'\t', line);
        let name = BStr::new(field_iter.next().unwrap());

        // Skip the other fields up to the actual path. Would be nice if
        // `Iterator::advance_by` was stable.
        field_iter.next().unwrap();
        field_iter.next().unwrap();
        field_iter.next().unwrap();
        field_iter.next().unwrap();

        // The actual path string (which we don't parse yet).
        let path = field_iter.next().unwrap();

        // Get the read's coordinates.
        field_iter.next().unwrap(); // Skip path length.
        let start: usize = parse_int_all(field_iter.next().unwrap()).unwrap();
        let end: usize = parse_int_all(field_iter.next().unwrap()).unwrap();

        Self {
            name,
            start,
            end,
            path,
        }
    }
}

struct PathChunker<'a, 'b> {
    gfa: &'a flatgfa::FlatGFA<'a>,
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
    fn new(gfa: &'a flatgfa::FlatGFA, read: GAFLine<'b>) -> Self {
        let steps = PathParser::new(read.path);
        Self {
            gfa,
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
struct ChunkEvent {
    index: usize,
    handle: flatgfa::Handle,
    range: ChunkRange,
}

#[derive(Debug)]
enum ChunkRange {
    None,
    All,
    Partial(usize),
}

impl<'a, 'b> Iterator for PathChunker<'a, 'b> {
    type Item = ChunkEvent;

    fn next(&mut self) -> Option<ChunkEvent> {
        let (seg_name, forward) = self.steps.next()?;

        // Get the corresponding handle from the GFA.
        let seg_id = self
            .gfa
            .find_seg(seg_name)
            .expect("GAF references unknown segment");
        let dir = match forward {
            true => flatgfa::Orientation::Forward,
            false => flatgfa::Orientation::Backward,
        };
        let handle = flatgfa::Handle::new(seg_id, dir);

        // Accumulate the length to track our position in the path.
        let next_pos = self.pos + self.gfa.segs[seg_id].len();
        let range = if !self.started && self.pos <= self.start && self.start < next_pos {
            self.started = true;
            ChunkRange::Partial(self.start - self.pos)
        } else if self.started && !self.ended && self.pos <= self.end && self.end < next_pos {
            self.ended = true;
            ChunkRange::Partial(self.end - self.pos)
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

/// Parse an integer from a byte string, which should contain only the integer.
fn parse_int_all(bytes: &[u8]) -> Option<usize> {
    let mut index = 0;
    let num = parse_int(bytes, &mut index)?;
    if index == bytes.len() {
        return Some(num);
    } else {
        return None;
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
