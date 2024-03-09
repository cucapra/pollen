use crate::flatgfa::{FlatGFA, Handle};
use gfa::gfa::Line;
use gfa::parser::GFAParserBuilder;

/// Parse a GFA text file.
pub fn parse<R: std::io::BufRead>(stream: R) -> FlatGFA {
    let parser = GFAParserBuilder::none()
        .segments(true)
        .paths(true)
        .build_usize_id::<()>();
    let mut flat = FlatGFA::default();
    for line in stream.lines() {
        let gfa_line = parser.parse_gfa_line(line.unwrap().as_ref()).unwrap();
        parse_line(&mut flat, gfa_line);
    }
    flat
}

/// Parse a single GFA line and add it to the flat representation.
fn parse_line(flat: &mut FlatGFA, line: Line<usize, ()>) {
    match line {
        Line::Header(_) => {}
        Line::Segment(s) => {
            flat.add_seg(s.name, s.sequence);
        }
        Line::Link(_) => {}
        Line::Path(p) => {
            let steps = parse_path_steps(p.segment_names);
            flat.add_path(p.path_name, steps);
        }
        Line::Containment(_) => {}
    }
}

/// Parse GFA paths' segment lists. These look like `1+,2-,3+`.
///
/// The underlying gfa-rs library does not yet parse the actual segments
/// involved in the path. So we do it ourselves: splitting on commas and
/// matching the direction.
fn parse_path_steps(data: Vec<u8>) -> Vec<Handle> {
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
                    steps.push(Handle {
                        segment: seg,
                        forward: byte == b'+',
                    });
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
    assert_eq!(
        path,
        vec![
            Handle {
                segment: 1,
                forward: true
            },
            Handle {
                segment: 23,
                forward: false
            },
            Handle {
                segment: 4,
                forward: true
            }
        ]
    );
}
