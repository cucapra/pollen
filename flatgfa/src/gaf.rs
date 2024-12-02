use crate::flatgfa;
use crate::memfile::map_file;
use crate::namemap::NameMap;
use argh::FromArgs;
use bstr::BStr;

/// look up positions from a GAF file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gaf")]
pub struct GAFLookup {
    /// GAF file associated with the GFA
    #[argh(positional)]
    gaf: String,

    /// print the actual sequences
    #[argh(switch, short = 's')]
    seqs: bool,

    /// benchmark only: print nothing; limit reads if nonzero
    #[argh(option, short = 'b')]
    bench: Option<u32>,
}

pub fn gaf_lookup(gfa: &flatgfa::FlatGFA, args: GAFLookup) {
    // Build a map to efficiently look up segments by name.
    let name_map = NameMap::build(gfa);

    let gaf_buf = map_file(&args.gaf);
    let parser = GAFParser::new(&gaf_buf);

    if args.seqs {
        // Print the actual sequences for each chunk in the GAF.
        for read in parser {
            print!("{}\t", read.name);
            for event in PathChunker::new(gfa, &name_map, read) {
                print_seq(gfa, event);
            }
            println!();
        }
    } else if let Some(limit) = args.bench {
        // Benchmarking mode: just process all the chunks but print nothing.
        let mut count = 0;
        for (i, read) in parser.enumerate() {
            for _event in PathChunker::new(gfa, &name_map, read) {
                count += 1;
            }
            if limit > 0 && i >= (limit as usize) {
                break;
            }
        }
        println!("{}", count);
    } else {
        // Just print some info about the offsets in the segments.
        for read in parser {
            println!("{}", read.name);
            for event in PathChunker::new(gfa, &name_map, read) {
                print_event(gfa, event);
            }
        }
    }
}

fn print_event(gfa: &flatgfa::FlatGFA, event: ChunkEvent) {
    let seg = gfa.segs[event.handle.segment()];
    let seg_name = seg.name;
    match event.range {
        ChunkRange::Partial(start, end) => {
            println!(
                "{}: {}{}, {}-{}bp",
                event.index,
                seg_name,
                event.handle.orient(),
                start,
                end,
            );
        }
        ChunkRange::All => {
            println!(
                "{}: {}{}, {}bp",
                event.index,
                seg_name,
                event.handle.orient(),
                seg.len()
            );
        }
        ChunkRange::None => {
            println!("{}: (skipped)", event.index);
        }
    }
}

fn print_seq(gfa: &flatgfa::FlatGFA, event: ChunkEvent) {
    let seg = gfa.segs[event.handle.segment()];
    let seq = gfa.get_seq(&seg);
    // TODO Reverse-complement for backward orientation.
    match event.range {
        ChunkRange::Partial(start, end) => {
            print!("{}", &seq[start..end]);
        }
        ChunkRange::All => {
            print!("{}", seq);
        }
        ChunkRange::None => {}
    }
}

struct GAFParser<'a> {
    buf: &'a [u8],
    pos: usize,
}

#[derive(Debug)]
struct GAFLine<'a> {
    name: &'a BStr,
    start: usize,
    end: usize,
    path: &'a [u8],
}

impl<'a> GAFParser<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn next_field(&mut self) -> Option<&'a [u8]> {
        let start = self.pos;
        let end = memchr::memchr(b'\t', &self.buf[self.pos..])?;
        self.pos += end + 1;
        Some(&self.buf[start..(start + end)])
    }

    fn skip_fields(&mut self, n: usize) -> Option<()> {
        for _ in 0..n {
            let end = memchr::memchr(b'\t', &self.buf[self.pos..])?;
            self.pos += end + 1;
        }
        Some(())
    }

    fn int_field(&mut self) -> Option<usize> {
        let val = parse_int(&self.buf, &mut self.pos);
        assert!(matches!(self.buf[self.pos], b'\t' | b'\n'));
        self.pos += 1;
        val
    }

    fn advance_line(&mut self) -> bool {
        let newline_pos = memchr::memchr(b'\n', &self.buf[self.pos..]);
        match newline_pos {
            None => {
                self.pos = self.buf.len();
                false
            }
            Some(pos) => {
                self.pos += pos + 1;
                true
            }
        }
    }

    fn parse_line(&mut self) -> GAFLine<'a> {
        assert!(self.pos < self.buf.len());

        let name = BStr::new(self.next_field().unwrap());
        self.skip_fields(4);

        // The actual path string (which we don't parse yet).
        let path = self.next_field().unwrap();

        // Get the read's coordinates.
        self.skip_fields(1); // Skip path length.
        let start: usize = self.int_field().unwrap();
        let end: usize = self.int_field().unwrap();

        self.advance_line();

        GAFLine {
            name,
            start,
            end,
            path,
        }
    }
}

impl<'a> Iterator for GAFParser<'a> {
    type Item = GAFLine<'a>;

    fn next(&mut self) -> Option<GAFLine<'a>> {
        if self.pos >= self.buf.len() {
            return None;
        }
        Some(self.parse_line())
    }
}

struct PathChunker<'a, 'b> {
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
    fn new(gfa: &'a flatgfa::FlatGFA, name_map: &'a NameMap, read: GAFLine<'b>) -> Self {
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
struct ChunkEvent {
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
