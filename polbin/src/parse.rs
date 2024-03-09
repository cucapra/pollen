use crate::flatgfa::{FlatGFA, Handle, PathInfo, SegInfo};
use bstr::BString;
use gfa::gfa::Line;
use gfa::parser::{GFAParser, GFAParserBuilder};

pub fn parse_line(parser: &mut GFAParser<usize, ()>, flat: &mut FlatGFA, gfa_line: &[u8]) {
    let line = parser.parse_gfa_line(gfa_line.as_ref()).unwrap();
    match line {
        Line::Header(_) => {}
        Line::Segment(mut s) => {
            flat.segs.push(SegInfo {
                name: s.name,
                seq_offset: flat.seqdata.len(),
                seq_len: s.sequence.len(),
            });
            flat.seqdata.append(&mut s.sequence);
        }
        Line::Link(_) => {}
        Line::Path(p) => {
            // The underlying gfa-rs library does not yet parse the actual segments
            // involved in the path. So we do it ourselves: splitting on commas and
            // matching the direction.
            let mut segs = parse_path_segs(p.segment_names);

            flat.paths.push(PathInfo {
                name: BString::new(p.path_name),
                step_offset: flat.steps.len(),
                step_len: segs.len(),
            });
            flat.steps.append(&mut segs);

            // TODO Handle the overlaps.
        }
        Line::Containment(_) => {}
    }
}

pub fn parse<R: std::io::BufRead>(stream: R) -> FlatGFA {
    let mut parser = GFAParserBuilder::none()
        .segments(true)
        .paths(true)
        .build_usize_id::<()>();

    let mut flat = FlatGFA::default();
    for line in stream.lines() {
        parse_line(&mut parser, &mut flat, line.unwrap().as_ref());
    }
    flat
}

enum PathParseState {
    Seg,
    Comma,
}

/// Parse GFA paths' segment lists. These look like `1+,2-,3+`.
fn parse_path_segs(data: Vec<u8>) -> Vec<Handle> {
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
                } else if byte >= b'0' && byte <= b'9' {
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
    let path = parse_path_segs(b"1+,23-,4+".to_vec());
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
