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
        for step in PathParser::new(path) {
            dbg!(step);
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
        let mut seg_name: usize = 0;
        while self.index < self.str.len() {
            let byte = self.str[self.index];
            if byte.is_ascii_digit() {
                seg_name *= 10;
                seg_name += (byte - b'0') as usize;
                self.index += 1;
            } else {
                break;
            }
        }
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
