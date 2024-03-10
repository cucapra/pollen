use crate::flatgfa::{FlatGFA, Handle, Orientation};
use gfa::{self, gfa::Line, parser::GFAParserBuilder};
use std::collections::HashMap;

#[derive(Default)]
pub struct Parser {
    // Track the segment IDs by their name, which we need to refer to segments in paths.
    segs_by_name: HashMap<usize, usize>,

    // The flat representation we're building.
    flat: FlatGFA,
}

impl Parser {
    /// Parse a GFA text file.
    pub fn parse<R: std::io::BufRead>(stream: R) -> FlatGFA {
        let gfa_parser = GFAParserBuilder::none()
            .segments(true)
            .paths(true)
            .build_usize_id::<()>();
        let mut parser = Self::default();
        for line in stream.lines() {
            let gfa_line = gfa_parser.parse_gfa_line(line.unwrap().as_ref()).unwrap();
            parser.parse_line(gfa_line);
        }
        parser.flat
    }

    /// Parse a single GFA line and add it to the flat representation.
    fn parse_line(&mut self, line: Line<usize, ()>) {
        match line {
            Line::Header(h) => {
                self.flat.add_header(h.version.unwrap());
            }
            Line::Segment(s) => {
                let seg_id = self.flat.add_seg(s.name, s.sequence);
                self.segs_by_name.insert(s.name, seg_id);
            }
            Line::Link(l) => {
                let from = Handle {
                    segment: self.segs_by_name[&l.from_segment],
                    orient: convert_orient(l.from_orient),
                };
                let to = Handle {
                    segment: self.segs_by_name[&l.to_segment],
                    orient: convert_orient(l.to_orient),
                };
                self.flat.add_link(from, to);
            }
            Line::Path(p) => {
                let steps = parse_path_steps(p.segment_names);
                self.flat.add_path(
                    p.path_name,
                    steps
                        .into_iter()
                        .map(|(name, dir)| Handle {
                            segment: self.segs_by_name[&name],
                            orient: if dir {
                                Orientation::Forward
                            } else {
                                Orientation::Backward
                            },
                        })
                        .collect(),
                );
            }
            Line::Containment(_) => {}
        }
    }
}

fn convert_orient(o: gfa::gfa::Orientation) -> Orientation {
    match o {
        gfa::gfa::Orientation::Forward => Orientation::Forward,
        gfa::gfa::Orientation::Backward => Orientation::Backward,
    }
}

/// Parse GFA paths' segment lists. These look like `1+,2-,3+`.
///
/// The underlying gfa-rs library does not yet parse the actual segments
/// involved in the path. So we do it ourselves: splitting on commas and
/// matching the direction.
fn parse_path_steps(data: Vec<u8>) -> Vec<(usize, bool)> {
    // The parser state: we're either looking for a segment name (or a +/- terminator),
    // or we're expecting a comma (or end of string).
    enum PathParseState {
        Seg,
        Comma,
    }

    let mut state = PathParseState::Seg;
    let mut seg: usize = 0;
    let mut steps = vec![];
    for byte in data {
        match state {
            PathParseState::Seg => {
                if byte == b'+' || byte == b'-' {
                    steps.push((seg, byte == b'+'));
                    state = PathParseState::Comma;
                } else if byte.is_ascii_digit() {
                    seg *= 10;
                    seg += (byte - b'0') as usize;
                } else {
                    panic!("unexpected character in path: {}", byte as char);
                }
            }
            PathParseState::Comma => {
                if byte == b',' {
                    state = PathParseState::Seg;
                    seg = 0;
                } else {
                    panic!("unexpected character in path: {}", byte as char);
                }
            }
        }
    }
    steps
}

#[test]
fn test_parse_path() {
    let path = parse_path_steps(b"1+,23-,4+".to_vec());
    assert_eq!(path, vec![(1, true), (23, false), (4, true)]);
}
