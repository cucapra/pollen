use bstr::BString;
use gfa::gfa::Line;
use gfa::parser::GFAParserBuilder;
use std::io::{self, BufRead};

#[derive(Debug)]
struct SegInfo {
    name: usize,
    seq_offset: usize,
    seq_len: usize,
}

#[derive(Debug)]
struct PathInfo {
    name: BString,
    step_offset: usize,
    step_len: usize,
}

#[derive(Debug)]
struct Handle {
    segment: usize,
    forward: bool,
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

fn main() {
    let parser = GFAParserBuilder::none()
        .segments(true)
        .paths(true)
        .build_usize_id::<()>();

    let mut seqdata: Vec<u8> = vec![];
    let mut segs: Vec<SegInfo> = vec![];
    let mut paths: Vec<PathInfo> = vec![];
    let mut steps: Vec<Handle> = vec![];

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = parser.parse_gfa_line(line.unwrap().as_ref()).unwrap();
        match line {
            Line::Header(_) => {}
            Line::Segment(mut s) => {
                segs.push(SegInfo {
                    name: s.name,
                    seq_offset: seqdata.len(),
                    seq_len: s.sequence.len(),
                });
                seqdata.append(&mut s.sequence);
            }
            Line::Link(_) => {}
            Line::Path(p) => {
                // The underlying gfa-rs library does not yet parse the actual segments
                // involved in the path. So we do it ourselves: splitting on commas and
                // matching the direction.
                let mut segs = parse_path_segs(p.segment_names);

                paths.push(PathInfo {
                    name: BString::new(p.path_name),
                    step_offset: steps.len(),
                    step_len: segs.len(),
                });
                steps.append(&mut segs);

                // TODO Handle the overlaps.
            }
            Line::Containment(_) => {}
        }
    }

    dbg!(seqdata);
    dbg!(segs);
    dbg!(paths);
    dbg!(steps);
}
