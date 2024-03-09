use gfa::gfa::Line;
use gfa::parser::GFAParserBuilder;
use std::io::{self, BufRead};

#[derive(Debug)]
struct SegInfo {
    name: usize,
    seq_offset: usize,
    seq_len: usize,
}

fn main() {
    let parser = GFAParserBuilder::none()
        .segments(true)
        .paths(true)
        .build_usize_id::<()>();

    let mut seqdata: Vec<u8> = vec![];
    let mut segs: Vec<SegInfo> = vec![];

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
            Line::Path(_) => {}
            Line::Containment(_) => {}
        }
    }

    dbg!(seqdata);
    dbg!(segs);
}
