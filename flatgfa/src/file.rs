use crate::flatgfa;
use crate::pool::{FixedStore, Pool, Span, Store};
use memmap::{Mmap, MmapMut};
use std::mem::{size_of, size_of_val};
use tinyvec::SliceVec;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

const MAGIC_NUMBER: u64 = 0xB101_1054;

/// A table of contents for the FlatGFA file.
#[derive(FromBytes, FromZeroes, AsBytes, Debug)]
#[repr(packed)]
pub struct Toc {
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
#[derive(FromBytes, FromZeroes, AsBytes, Clone, Copy, Debug)]
#[repr(packed)]
struct Size {
    /// The number of actual elements in the pool.
    len: usize,

    // The allocated space for the pool. `capacity - len` slots are "empty."
    capacity: usize,
}

impl Size {
    fn of_pool<T>(pool: Pool<T>) -> Self {
        Size {
            len: pool.len(),
            capacity: pool.len(),
        }
    }

    fn of_store<T: Clone>(store: &FixedStore<'_, T>) -> Self {
        Size {
            len: store.len(),
            capacity: store.capacity(),
        }
    }

    fn bytes<T>(&self) -> usize {
        self.capacity * size_of::<T>()
    }

    fn empty(capacity: usize) -> Self {
        Size { len: 0, capacity }
    }
}

impl Toc {
    /// Get the total size in bytes of the file described.
    pub fn size(&self) -> usize {
        size_of::<Self>()
            + self.header.bytes::<u8>()
            + self.segs.bytes::<flatgfa::Segment>()
            + self.paths.bytes::<flatgfa::Path>()
            + self.links.bytes::<flatgfa::Link>()
            + self.steps.bytes::<flatgfa::Handle>()
            + self.seq_data.bytes::<u8>()
            + self.overlaps.bytes::<Span<flatgfa::AlignOp>>()
            + self.alignment.bytes::<flatgfa::AlignOp>()
            + self.name_data.bytes::<u8>()
            + self.optional_data.bytes::<u8>()
            + self.line_order.bytes::<u8>()
    }

    /// Get a table of contents that fits a FlatGFA with no spare space.
    fn full(gfa: &flatgfa::FlatGFA) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            header: Size::of_pool(gfa.header),
            segs: Size::of_pool(gfa.segs),
            paths: Size::of_pool(gfa.paths),
            links: Size::of_pool(gfa.links),
            steps: Size::of_pool(gfa.steps),
            seq_data: Size::of_pool(gfa.seq_data),
            overlaps: Size::of_pool(gfa.overlaps),
            alignment: Size::of_pool(gfa.alignment),
            name_data: Size::of_pool(gfa.name_data),
            optional_data: Size::of_pool(gfa.optional_data),
            line_order: Size::of_pool(gfa.line_order),
        }
    }

    pub fn for_fixed_store(store: &flatgfa::FixedGFAStore) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            header: Size::of_store(&store.header),
            segs: Size::of_store(&store.segs),
            paths: Size::of_store(&store.paths),
            links: Size::of_store(&store.links),
            steps: Size::of_store(&store.steps),
            seq_data: Size::of_store(&store.seq_data),
            overlaps: Size::of_store(&store.overlaps),
            alignment: Size::of_store(&store.alignment),
            name_data: Size::of_store(&store.name_data),
            optional_data: Size::of_store(&store.optional_data),
            line_order: Size::of_store(&store.line_order),
        }
    }

    /// Guess a reasonable set of capacities for a fresh file.
    pub fn guess(factor: usize) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            header: Size::empty(128),
            segs: Size::empty(32 * factor * factor),
            paths: Size::empty(factor),
            links: Size::empty(32 * factor * factor),
            steps: Size::empty(1024 * factor * factor),
            seq_data: Size::empty(512 * factor * factor),
            overlaps: Size::empty(256 * factor),
            alignment: Size::empty(64 * factor * factor),
            name_data: Size::empty(64 * factor),
            optional_data: Size::empty(512 * factor * factor),
            line_order: Size::empty(64 * factor * factor),
        }
    }

    /// Estimate a reasonable set of capacities for a fresh file based on some
    /// measurements of the GFA text.
    pub fn estimate(
        segs: usize,
        links: usize,
        paths: usize,
        header_bytes: usize,
        seg_bytes: usize,
        path_bytes: usize,
    ) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            header: Size::empty(header_bytes),
            segs: Size::empty(segs),
            paths: Size::empty(paths),
            links: Size::empty(links),
            steps: Size::empty(path_bytes / 3),
            seq_data: Size::empty(seg_bytes),
            overlaps: Size::empty((links + paths) * 2),
            alignment: Size::empty(links * 2 + paths * 4),
            name_data: Size::empty(paths * 512),
            optional_data: Size::empty(links * 16),
            line_order: Size::empty(segs + links + paths + 8),
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
        header: header.into(),
        segs: segs.into(),
        paths: paths.into(),
        links: links.into(),
        steps: steps.into(),
        seq_data: seq_data.into(),
        overlaps: overlaps.into(),
        alignment: alignment.into(),
        name_data: name_data.into(),
        optional_data: optional_data.into(),
        line_order: line_order.into(),
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

/// Get a FlatGFA `SliceStore` from the suffix of a file just following the table of contents.
fn slice_store<'a>(data: &'a mut [u8], toc: &Toc) -> flatgfa::FixedGFAStore<'a> {
    let (header, rest) = slice_vec_prefix(data, toc.header);
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

    flatgfa::FixedGFAStore {
        header: header.into(),
        segs: segs.into(),
        paths: paths.into(),
        links: links.into(),
        steps: steps.into(),
        seq_data: seq_data.into(),
        overlaps: overlaps.into(),
        alignment: alignment.into(),
        name_data: name_data.into(),
        optional_data: optional_data.into(),
        line_order: line_order.into(),
    }
}

/// Get a mutable FlatGFA `SliceStore` backed by a byte buffer.
pub fn view_store(data: &mut [u8]) -> flatgfa::FixedGFAStore {
    let (toc, rest) = read_toc_mut(data);
    slice_store(rest, toc)
}

/// Initialize a buffer with an empty FlatGFA store.
pub fn init(data: &mut [u8], toc: Toc) -> (&mut Toc, flatgfa::FixedGFAStore) {
    // Write the table of contents.
    assert!(data.len() == toc.size());
    toc.write_to_prefix(data).unwrap();

    // Get a mutable reference to the embedded TOC.
    let (toc_bytes, rest) = data.split_at_mut(size_of::<Toc>());
    let toc_mut = Toc::mut_from(toc_bytes).unwrap();

    // Extract a store from the remaining bytes.
    (toc_mut, slice_store(rest, &toc))
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
    let toc = Toc::full(gfa);
    let rest = write_bump(buf, &toc).unwrap();

    // All the slices.
    let rest = write_bytes(rest, gfa.header.all()).unwrap();
    let rest = write_bump(rest, gfa.segs.all()).unwrap();
    let rest = write_bump(rest, gfa.paths.all()).unwrap();
    let rest = write_bump(rest, gfa.links.all()).unwrap();
    let rest = write_bump(rest, gfa.steps.all()).unwrap();
    let rest = write_bytes(rest, gfa.seq_data.all()).unwrap();
    let rest = write_bump(rest, gfa.overlaps.all()).unwrap();
    let rest = write_bump(rest, gfa.alignment.all()).unwrap();
    let rest = write_bytes(rest, gfa.name_data.all()).unwrap();
    let rest = write_bytes(rest, gfa.optional_data.all()).unwrap();
    write_bytes(rest, gfa.line_order.all()).unwrap();
}

/// Get the total size in bytes of a FlatGFA structure. This should result in a big
/// enough buffer to write the entire FlatGFA into with `dump`.
pub fn size(gfa: &flatgfa::FlatGFA) -> usize {
    Toc::full(gfa).size()
}

pub fn map_file(name: &str) -> Mmap {
    let file = std::fs::File::open(name).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

pub fn map_new_file(name: &str, size: u64) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(name)
        .unwrap();
    file.set_len(size).unwrap();
    unsafe { MmapMut::map_mut(&file) }.unwrap()
}

pub fn map_file_mut(name: &str) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name)
        .unwrap();
    unsafe { MmapMut::map_mut(&file) }.unwrap()
}
