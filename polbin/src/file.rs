use crate::flatgfa;
use std::mem::{size_of, size_of_val};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

const MAGIC_NUMBER: u64 = 0xB101_1054;

#[derive(FromBytes, FromZeroes, AsBytes)]
#[repr(packed)]
struct Toc {
    magic: u64,
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

/// Get a FlatGFA backed by the data in a byte buffer.
pub fn view(data: &[u8]) -> flatgfa::FlatGFA {
    // Table of contents.
    let toc = Toc::ref_from_prefix(data).unwrap();
    let rest = &data[size_of::<Toc>()..];
    let magic = toc.magic;
    assert_eq!(magic, MAGIC_NUMBER);

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

fn write_bump<'a, T: AsBytes + ?Sized>(buf: &'a mut [u8], data: &T) -> Option<&'a mut [u8]> {
    let len = size_of_val(data);
    data.write_to_prefix(buf)?;
    Some(&mut buf[len..])
}

fn write_bytes<'a>(buf: &'a mut [u8], data: &[u8]) -> Option<&'a mut [u8]> {
    let len = data.len();
    buf[0..len].copy_from_slice(data);
    Some(&mut buf[len..])
}

/// Copy a FlatGFA into a byte buffer.
pub fn dump(gfa: &flatgfa::FlatGFA, buf: &mut [u8]) {
    // Table of contents.
    let toc = Toc {
        magic: MAGIC_NUMBER,
        header_len: gfa.header.len(),
        segs_count: gfa.segs.len(),
        paths_count: gfa.paths.len(),
        links_count: gfa.links.len(),
        steps_count: gfa.steps.len(),
        seq_data_len: gfa.seq_data.len(),
        overlaps_count: gfa.overlaps.len(),
        alignment_count: gfa.alignment.len(),
        name_data_len: gfa.name_data.len(),
        optional_data_len: gfa.optional_data.len(),
        line_order_len: gfa.line_order.len(),
    };
    let rest = write_bump(buf, &toc).unwrap();

    // All the slices.
    let rest = write_bytes(rest, gfa.header).unwrap();
    let rest = write_bump(rest, gfa.segs).unwrap();
    let rest = write_bump(rest, gfa.paths).unwrap();
    let rest = write_bump(rest, gfa.links).unwrap();
    let rest = write_bump(rest, gfa.steps).unwrap();
    let rest = write_bytes(rest, gfa.seq_data).unwrap();
    let rest = write_bump(rest, gfa.overlaps).unwrap();
    let rest = write_bump(rest, gfa.alignment).unwrap();
    let rest = write_bytes(rest, gfa.name_data).unwrap();
    let rest = write_bytes(rest, gfa.optional_data).unwrap();
    write_bytes(rest, gfa.line_order).unwrap();
}

/// Get the total size in bytes of a FlatGFA structure. This should result in a big
/// enough buffer to write the entire FlatGFA into with `dump`.
pub fn size(gfa: &flatgfa::FlatGFA) -> usize {
    size_of::<Toc>()
        + gfa.header.len()
        + size_of_val(gfa.segs)
        + size_of_val(gfa.paths)
        + size_of_val(gfa.links)
        + size_of_val(gfa.steps)
        + size_of_val(gfa.seq_data)
        + size_of_val(gfa.overlaps)
        + size_of_val(gfa.alignment)
        + gfa.name_data.len()
        + gfa.optional_data.len()
        + gfa.line_order.len()
}
