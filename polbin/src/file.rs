use crate::flatgfa;
use std::mem::{size_of, size_of_val};
use tinyvec::SliceVec;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

const MAGIC_NUMBER: u64 = 0xB101_1054;

/// A table of contents for the FlatGFA file.
#[derive(FromBytes, FromZeroes, AsBytes)]
#[repr(packed)]
struct Toc {
    magic: u64,
    header: Size,
    segs: Size,
    paths: Size,
    links: Size,
    steps: Size,
    seq_data: Size,
    overlaps: Size,
    alignment: Size,
    name_data: Size,
    optional_data: Size,
    line_order: Size,
}

/// A table-of-contents entry for a pool in the FlatGFA file.
#[derive(FromBytes, FromZeroes, AsBytes, Clone, Copy)]
#[repr(packed)]
struct Size {
    /// The number of actual elements in the pool.
    len: usize,

    // The allocated space for the pool. `capacity - len` slots are "empty."
    capacity: usize,
}

impl Size {
    fn of_slice<T>(slice: &[T]) -> Self {
        Size {
            len: slice.len(),
            capacity: slice.len(),
        }
    }
}

/// Consume `size.len` items from a byte slice, skip the remainder of `size.capacity`
/// elements, and return the items and the rest of the slice.
fn slice_prefix<T: FromBytes>(data: &[u8], size: Size) -> (&[T], &[u8]) {
    let (prefix, rest) = T::slice_from_prefix(data, size.len).unwrap();
    let pad = size_of::<T>() * (size.capacity - size.len);
    (prefix, &rest[pad..])
}

/// Read the table of contents from a prefix of the byte buffer.
fn read_toc(data: &[u8]) -> (&Toc, &[u8]) {
    let toc = Toc::ref_from_prefix(data).unwrap();
    let rest = &data[size_of::<Toc>()..];
    let magic = toc.magic;
    assert_eq!(magic, MAGIC_NUMBER);
    (toc, rest)
}

fn read_toc_mut(data: &mut [u8]) -> (&mut Toc, &mut [u8]) {
    let (toc_slice, rest) = Toc::mut_slice_from_prefix(data, 1).unwrap();
    let toc = &mut toc_slice[0];
    let magic = toc.magic;
    assert_eq!(magic, MAGIC_NUMBER);
    (toc, rest)
}

/// Get a FlatGFA backed by the data in a byte buffer.
pub fn view(data: &[u8]) -> flatgfa::FlatGFA {
    let (toc, rest) = read_toc(data);

    let (header, rest) = slice_prefix(rest, toc.header);
    let (segs, rest) = slice_prefix(rest, toc.segs);
    let (paths, rest) = slice_prefix(rest, toc.paths);
    let (links, rest) = slice_prefix(rest, toc.links);
    let (steps, rest) = slice_prefix(rest, toc.steps);
    let (seq_data, rest) = slice_prefix(rest, toc.seq_data);
    let (overlaps, rest) = slice_prefix(rest, toc.overlaps);
    let (alignment, rest) = slice_prefix(rest, toc.alignment);
    let (name_data, rest) = slice_prefix(rest, toc.name_data);
    let (optional_data, rest) = slice_prefix(rest, toc.optional_data);
    let (line_order, _) = slice_prefix(rest, toc.line_order);

    flatgfa::FlatGFA {
        header,
        segs,
        paths,
        links,
        steps,
        seq_data,
        overlaps,
        alignment,
        name_data,
        optional_data,
        line_order,
    }
}

/// Like `slice_prefix`, but produce a `SliceVec`.
fn slice_vec_prefix<T: FromBytes + AsBytes>(
    data: &mut [u8],
    size: Size,
) -> (SliceVec<T>, &mut [u8]) {
    let (prefix, rest) = T::mut_slice_from_prefix(data, size.capacity).unwrap();
    let vec = SliceVec::from_slice_len(prefix, size.len);
    (vec, rest)
}

/// Get a mutable FlatGFA `SliceStore` backed by a byte buffer.
pub fn view_store(data: &mut [u8]) -> flatgfa::SliceStore {
    let (toc, rest) = read_toc_mut(data);

    // Get slices for each chunk.
    let (header, rest) = slice_vec_prefix(rest, toc.header);
    let (segs, rest) = slice_vec_prefix(rest, toc.segs);
    let (paths, rest) = slice_vec_prefix(rest, toc.paths);
    let (links, rest) = slice_vec_prefix(rest, toc.links);
    let (steps, rest) = slice_vec_prefix(rest, toc.steps);
    let (seq_data, rest) = slice_vec_prefix(rest, toc.seq_data);
    let (overlaps, rest) = slice_vec_prefix(rest, toc.overlaps);
    let (alignment, rest) = slice_vec_prefix(rest, toc.alignment);
    let (name_data, rest) = slice_vec_prefix(rest, toc.name_data);
    let (optional_data, rest) = slice_vec_prefix(rest, toc.optional_data);
    let (line_order, _) = slice_vec_prefix(rest, toc.line_order);

    flatgfa::SliceStore {
        header,
        segs,
        paths,
        links,
        steps,
        seq_data,
        overlaps,
        alignment,
        name_data,
        optional_data,
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
        header: Size::of_slice(gfa.header),
        segs: Size::of_slice(gfa.segs),
        paths: Size::of_slice(gfa.paths),
        links: Size::of_slice(gfa.links),
        steps: Size::of_slice(gfa.steps),
        seq_data: Size::of_slice(gfa.seq_data),
        overlaps: Size::of_slice(gfa.overlaps),
        alignment: Size::of_slice(gfa.alignment),
        name_data: Size::of_slice(gfa.name_data),
        optional_data: Size::of_slice(gfa.optional_data),
        line_order: Size::of_slice(gfa.line_order),
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
