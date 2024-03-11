use crate::flatgfa;
use bstr::BStr;
use zerocopy::{FromBytes, FromZeroes};

const MAGIC_NUMBER: usize = 0x1337_4915;

#[derive(FromBytes, FromZeroes)]
struct TOC {
    magic: usize,
    header_len: usize,
    segs_count: usize,
    paths_count: usize,
    links_count: usize,
    steps_count: usize,
    seq_data_len: usize,
    overlaps_count: usize,
    alignment_len: usize,
    name_data_len: usize,
    optional_data_len: usize,
    line_order_len: usize,
}

pub fn load(data: &[u8]) -> flatgfa::FlatGFA {
    // Table of contents.
    let toc = TOC::ref_from_prefix(data).unwrap();
    let rest = &data[std::mem::size_of::<TOC>()..];
    assert_eq!(toc.magic, MAGIC_NUMBER);

    // Header (version).
    let header = BStr::new(&rest[0..toc.header_len]);
    let rest = &rest[toc.header_len..];

    // Segments.
    let (segs, rest) = flatgfa::Segment::slice_from_prefix(rest, toc.segs_count).unwrap();

    flatgfa::FlatGFA {
        header,
        segs,
        paths: todo!(),
        links: todo!(),
        steps: todo!(),
        seq_data: todo!(),
        overlaps: todo!(),
        alignment: todo!(),
        name_data: todo!(),
        optional_data: todo!(),
        line_order: todo!(),
    }
}
