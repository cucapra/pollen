use gfa::parser::GFAParserBuilder;
use std::io::{self, BufRead};

fn main() {
    let parser = GFAParserBuilder::none()
        .segments(true)
        .paths(true)
        .build_usize_id::<()>();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = parser.parse_gfa_line(line.unwrap().as_ref());
        dbg!(line.unwrap());
    }
}
