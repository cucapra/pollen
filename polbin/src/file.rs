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
    alignment_count: usize,
    name_data_len: usize,
    optional_data_len: usize,
    line_order_len: usize,
}

/// Get the first `len` bytes in a byte slice, and return the rest of the slice.
fn get_prefix(data: &[u8], len: usize) -> (&[u8], &[u8]) {
    assert!(data.len() >= len);
    (&data[0..len], &data[len..])
}

pub fn load(data: &[u8]) -> flatgfa::FlatGFA {
    // Table of contents.
    let toc = TOC::ref_from_prefix(data).unwrap();
    let rest = &data[std::mem::size_of::<TOC>()..];
    assert_eq!(toc.magic, MAGIC_NUMBER);

    // Get slices for each chunk.
    let (header, rest) = get_prefix(rest, toc.header_len);
    let (segs, rest) = flatgfa::Segment::slice_from_prefix(rest, toc.segs_count).unwrap();
    let (paths, rest) = flatgfa::Path::slice_from_prefix(rest, toc.paths_count).unwrap();
    let (links, rest) = flatgfa::Link::slice_from_prefix(rest, toc.links_count).unwrap();
    let (steps, rest) = flatgfa::Handle::slice_from_prefix(rest, toc.steps_count).unwrap();
    let (seq_data, rest) = get_prefix(rest, toc.seq_data_len);
    let (overlaps, rest) = flatgfa::Span::slice_from_prefix(rest, toc.overlaps_count).unwrap();
    let (alignment, rest) = flatgfa::AlignOp::slice_from_prefix(rest, toc.alignment_count).unwrap();
    let (name_data, rest) = get_prefix(rest, toc.name_data_len);
    let (optional_data, rest) = get_prefix(rest, toc.optional_data_len);
    let (line_order, _) = get_prefix(rest, toc.line_order_len);

    flatgfa::FlatGFA {
        header: header.into(),
        segs,
        paths,
        links,
        steps,
        seq_data,
        overlaps,
        alignment,
        name_data: name_data.into(),
        optional_data: optional_data.into(),
        line_order,
    }
}
