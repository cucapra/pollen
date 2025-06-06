use crate::file::*;
use crate::flatgfa;
use crate::memfile::map_new_file;
use crate::pool::*;
use crate::FixedFamily;
use crate::HeapFamily;
use crate::StoreFamily;
use std::fmt;

use zerocopy::*;

const MAGIC_NUMBER: u8 = 0x12;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Nucleotide {
    A,
    C,
    T,
    G,
}

impl From<u8> for Nucleotide {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::A,
            1 => Self::C,
            2 => Self::T,
            3 => Self::G,
            _ => panic!("Not a Nucleotide!"),
        }
    }
}

impl From<Nucleotide> for u8 {
    fn from(value: Nucleotide) -> Self {
        match value {
            Nucleotide::A => 0,
            Nucleotide::C => 1,
            Nucleotide::T => 2,
            Nucleotide::G => 3,
        }
    }
}

impl From<Nucleotide> for char {
    fn from(value: Nucleotide) -> Self {
        match value {
            Nucleotide::A => 'A',
            Nucleotide::C => 'C',
            Nucleotide::G => 'G',
            Nucleotide::T => 'T',
        }
    }
}

/// A compressed vector-like structure for storing nucleotide sequences
///     - Two base pairs are stored per byte
///
pub struct PackedSeqStore {
    /// A vector that stores a compressed encoding of this PackedSeqStore's sequence
    data: Vec<u8>,

    /// True if the final base pair in the sequence is stored at a
    ///                   high nibble
    high_nibble_end: bool,
}

pub struct PackedSeqView<'a> {
    data: &'a [u8],

    /// True if the final base pair in the sequence is stored at a
    ///                   high nibble
    high_nibble_end: bool,
}

#[derive(FromBytes, FromZeroes, AsBytes, Debug)]
#[repr(packed)]
pub struct PackedToc {
    magic: u8,
    data: Size,
    high_nibble_end: Size,
}

impl PackedToc {
    pub fn size(&self) -> usize {
        size_of::<Self>() + self.data.bytes::<u8>()
    }

    fn full(seq: &PackedSeqView) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            data: Size {
                len: seq.data.len(),
                capacity: seq.data.len(),
            },
            high_nibble_end: Size {
                len: 1,
                capacity: 1,
            },
        }
    }
}

fn read_packed_toc(data: &[u8]) -> (&PackedToc, &[u8]) {
    let toc = PackedToc::ref_from_prefix(data).unwrap();
    let rest = &data[size_of::<PackedToc>()..];
    let magic = toc.magic;
    assert_eq!(magic, MAGIC_NUMBER);
    (toc, rest)
}

pub fn view(data: &[u8]) -> PackedSeqView {
    let (toc, rest) = read_packed_toc(data);

    let (data, rest) = slice_prefix(rest, toc.data);

    let (high_nibble_end, _) = slice_prefix(rest, toc.high_nibble_end);

    PackedSeqView {
        data: data.into(),
        high_nibble_end: match high_nibble_end {
            [0u8] => false,
            [1u8] => true,
            _ => panic!("Invalid value in high_nibble_end"),
        },
    }
}

pub fn dump(seq: &PackedSeqView, buf: &mut [u8]) {
    let toc = PackedToc::full(seq);
    let rest = write_bump(buf, &toc).unwrap();
    let rest = write_bytes(rest, seq.data).unwrap();

    let nibble: &[u8] = if seq.high_nibble_end { &[1] } else { &[0] };

    write_bytes(rest, nibble).unwrap();
}

impl<'a> PackedSeqView<'a> {
    pub fn len(&self) -> usize {
        if self.high_nibble_end {
            self.data.len() * 2
        } else {
            self.data.len() * 2 - 1
        }
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> Nucleotide {
        let i = index / 2;
        if index % 2 == 1 {
            ((self.data[i] & 0b11110000u8) >> 4).into()
        } else {
            (self.data[i] & 0b00001111u8).into()
        }
    }

    pub fn get_range(&self, span: std::ops::Range<usize>) -> Vec<Nucleotide> {
        let mut arr: Vec<Nucleotide> = Vec::with_capacity(span.end - span.start);
        for i in span.start..=span.end {
            arr.push(self.get(i));
        }
        arr
    }

    /// Returns a uncompressed vector that contains the same sequence as this PackedSeqStore
    pub fn get_elements(&self) -> Vec<Nucleotide> {
        self.get_range(0..(self.len() - 1))
    }
}

impl<'a> fmt::Display for PackedSeqView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut i = 0;
        for item in PackedSeqViewIterator::new(self) {
            if i == 0 {
                i = 1;
            } else {
                write!(f, ", ")?;
            }
            let c: char = item.into();
            write!(f, "{}", c)?;
        }
        write!(f, "]")
    }
}

struct PackedSeqViewIterator<'a> {
    data: &'a PackedSeqView<'a>,
    cur_index: usize,
}

impl<'a> PackedSeqViewIterator<'a> {
    pub fn new(vec: &'a PackedSeqView<'a>) -> Self {
        Self {
            data: vec,
            cur_index: 0,
        }
    }
}

impl Iterator for PackedSeqViewIterator<'_> {
    type Item = Nucleotide;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index < self.data.len() {
            self.cur_index += 1;
            Some(self.data.get(self.cur_index - 1))
        } else {
            None
        }
    }
}

impl PackedSeqStore {
    /// Creates a new empty PackedSeqStore
    pub fn new() -> Self {
        PackedSeqStore {
            data: Vec::new(),
            high_nibble_end: true,
        }
    }

    /// Returns a compressed PackedSeqStore given an uncompressed vector `arr`
    pub fn create(arr: Vec<Nucleotide>) -> Self {
        let mut new_vec = PackedSeqStore::new();
        for item in arr {
            new_vec.push(item);
        }
        new_vec
    }

    /// Appends `input` to the end of this PackedSeqStore
    pub fn push(&mut self, input: Nucleotide) {
        let value = input.into();
        assert!(value <= 0xF);
        if self.high_nibble_end {
            self.data.push(value);
            self.high_nibble_end = false;
        } else {
            let last_index = self.data.len() - 1;
            self.data[last_index] |= value << 4;
            self.high_nibble_end = true;
        }
    }

    /// Sets the element of this PackedSeqStore at index `index` to `elem`
    pub fn set(&mut self, index: usize, input: Nucleotide) {
        let elem: u8 = input.into();
        let i = index / 2;
        if index % 2 == 1 {
            println!("i: {}", i);
            self.data[i] = (0b00001111u8 & self.data[i]) | (elem << 4);
        } else {
            self.data[i] = (0b11110000u8 & self.data[i]) | elem;
        }
    }

    pub fn as_ref(&self) -> PackedSeqView {
        PackedSeqView {
            data: &self.data,
            high_nibble_end: self.high_nibble_end,
        }
    }
}

impl Default for PackedSeqStore {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference to a subsection of a nucleotide sequence stored in a PackedSeqStore
pub struct PackedSlice<'a> {
    /// The underlying vector that stores the sequence referenced by this slice
    vec_ref: &'a PackedSeqStore,

    /// The specific section of the sequence that this slice references
    span: std::ops::Range<usize>,
}

/// Returns a PackedSlice given a compressed PackVec `vec` that acts as a reference
/// to the section of `vec` contained within the index bounds of Span `s`.
pub fn create_slice(vec: &PackedSeqStore, s: std::ops::Range<usize>) -> PackedSlice<'_> {
    PackedSlice {
        vec_ref: vec,
        span: s,
    }
}

/// Returns a vector containing the base pairs referenced by `slice`
pub fn get_slice_seq(slice: PackedSlice<'_>) -> Vec<Nucleotide> {
    slice.vec_ref.as_ref().get_range(slice.span)
}

pub fn total_bytes(num_elems: usize) -> usize {
    let bytes = std::mem::size_of::<usize>();
    let seq_size = 1 + if num_elems % 2 == 1 {
        (num_elems / 2) + 1
    } else {
        num_elems / 2
    }; // Sequence plus the nibble byte

    let toc_size = bytes * 4 + 1; // 2 Size types each with two 8-byte
                                  // usize types, plus the magic number
    toc_size + seq_size
}

#[test]
fn test_vec() {
    let mut vec = PackedSeqStore::create(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
        Nucleotide::A,
    ]);
    vec.push(Nucleotide::A);
    let arr = vec.as_ref().get_elements();
    assert_eq!(arr[0], Nucleotide::A);
    assert_eq!(arr[1], Nucleotide::C);
    assert_eq!(arr[2], Nucleotide::G);
    assert_eq!(arr[3], Nucleotide::T);
    assert_eq!(arr[4], Nucleotide::A);
    assert_eq!(arr[5], Nucleotide::A);
}

#[test]
fn test_vec_push() {
    let mut vec = PackedSeqStore::create(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
    ]);
    vec.push(Nucleotide::A);
    vec.push(Nucleotide::C);
    vec.push(Nucleotide::G);
    vec.push(Nucleotide::T);
    let arr = vec.as_ref().get_elements();
    assert_eq!(arr[0], Nucleotide::A);
    assert_eq!(arr[1], Nucleotide::C);
    assert_eq!(arr[2], Nucleotide::G);
    assert_eq!(arr[3], Nucleotide::T);
    assert_eq!(arr[4], Nucleotide::A);
    assert_eq!(arr[5], Nucleotide::C);
    assert_eq!(arr[6], Nucleotide::G);
    assert_eq!(arr[7], Nucleotide::T);
}

#[test]
fn test_slice() {
    let span = 1..4;
    let vec = PackedSeqStore::create(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
        Nucleotide::A,
        Nucleotide::G,
    ]);
    let slice = create_slice(&vec, span);
    let arr = get_slice_seq(slice);
    assert_eq!(arr[0], Nucleotide::C);
    assert_eq!(arr[1], Nucleotide::G);
    assert_eq!(arr[2], Nucleotide::T);
    assert_eq!(arr[3], Nucleotide::A);
}

#[test]
fn test_display_even() {
    let vec = PackedSeqStore::create(vec![
        Nucleotide::C,
        Nucleotide::A,
        Nucleotide::T,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::C,
    ]);
    assert_eq!("[C, A, T, C, G, C]", vec.as_ref().to_string());
}

#[test]
fn test_display_single() {
    let vec = PackedSeqStore::create(vec![Nucleotide::T.into()]);
    assert_eq!("[T]", vec.as_ref().to_string());
}

#[test]
fn test_display_odd() {
    let vec = PackedSeqStore::create(vec![
        Nucleotide::C,
        Nucleotide::A,
        Nucleotide::T,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::C,
        Nucleotide::C,
    ]);
    assert_eq!("[C, A, T, C, G, C, C]", vec.as_ref().to_string());
}

#[test]
fn test_getter_setter() {
    let mut vec = PackedSeqStore::create(vec![
        Nucleotide::A,
        Nucleotide::A,
        Nucleotide::T,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::C,
        Nucleotide::C,
    ]);
    assert_eq!(vec.as_ref().get(0), Nucleotide::A);
    assert_eq!(vec.as_ref().get(1), Nucleotide::A);
    vec.set(1, Nucleotide::G);
    assert_eq!(vec.as_ref().get(1), Nucleotide::G);
}

#[test]
fn test_export_import() {
    let vec = PackedSeqStore::create(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::T,
        Nucleotide::G,
    ]);
    let input = vec.as_ref();
    let filename = "capra_test_file";
    let num_bytes = total_bytes(4);
    let mut mem = map_new_file(filename, num_bytes as u64);
    let mut buf = vec![0u8; num_bytes];
    let buf_ref = &mut buf;
    dump(&input, buf_ref);
    mem[..buf_ref.len()].copy_from_slice(buf_ref);
    let result: &[u8] = &mem;
    let output = view(result);
    assert_eq!(input.get(0), output.get(0));
    assert_eq!(input.get(1), output.get(1));
    assert_eq!(input.get(2), output.get(2));
    assert_eq!(input.get(3), output.get(3));
    std::mem::drop(mem);
    let _ = std::fs::remove_file(filename);
}
