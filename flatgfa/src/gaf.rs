use crate::flatgfa;
use crate::memfile::{map_file, MemchrSplit};
use argh::FromArgs;
use bstr::BStr;

/// look up positions from a GAF file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gaf")]
pub struct GafLookup {
    /// path_name,offset,orientation
    #[argh(positional)]
    gaf: String,
}

pub fn gaf_lookup(gfa: &flatgfa::FlatGFA, args: GafLookup) {
    // Read the GAF file, I suppose.
    let gaf_buf = map_file(&args.gaf);
    for line in MemchrSplit::new(b'\n', &gaf_buf) {
        let mut field_iter = MemchrSplit::new(b'\t', line);
        let read_name = BStr::new(field_iter.next().unwrap());
        dbg!(read_name);

        // Skip the other fields up to the actual path. Would be nice if
        // `Iterator::advance_by` was stable.
        field_iter.next().unwrap();
        field_iter.next().unwrap();
        field_iter.next().unwrap();
        field_iter.next().unwrap();

        // Using MemchrSplit to get the GAF fields is pretty lazy; it would be
        // better to manage the indices directly.
        let path = field_iter.next().unwrap();

        // Get the read's coordinates.
        let path_len: usize = parse_int_all(field_iter.next().unwrap()).unwrap();
        let read_start: usize = parse_int_all(field_iter.next().unwrap()).unwrap();
        let read_end: usize = parse_int_all(field_iter.next().unwrap()).unwrap();

        // Walk down the path to find the start and end coordinates in the segments.
        let mut pos = 0;
        let mut started = false;
        for (seg_name, forward) in PathParser::new(path) {
            let seg_id = gfa.find_seg(seg_name).expect("GAF has unknown segment");

            // Accumulate the length to track our position in the path.
            let next_pos = pos + gfa.segs[seg_id].len();
            if !started {
                if pos <= read_start && read_start < next_pos {
                    let seg_start_pos = read_start - pos;
                    println!("started at {}.{}", seg_name, seg_start_pos);
                    started = true;
                }
            } else {
                if pos <= read_end && read_end < next_pos {
                    let seg_end_pos = read_end - pos;
                    println!("ended at {}.{}", seg_name, seg_end_pos);
                    break;
                }
            }
            pos = next_pos;
        }
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
