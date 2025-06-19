use crate::file::*;
use crate::flatgfa;
use crate::memfile;
use crate::memfile::map_new_file;
use crate::pool::*;
use crate::FixedFamily;
use crate::HeapFamily;
use crate::StoreFamily;
use std::fmt;

use zerocopy::*;

const MAGIC_NUMBER: u64 = 0x12;

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
    magic: u64,
    data: Size,
    high_nibble_end: u8,
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
            high_nibble_end: if seq.high_nibble_end { 1u8 } else { 0u8 },
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

impl<'a> PackedSeqView<'a> {
    pub fn view(data: &'a [u8]) -> Self {
        let (toc, rest) = read_packed_toc(data);

        let (data, _) = slice_prefix(rest, toc.data);
        Self {
            data: data.into(),
            high_nibble_end: match toc.high_nibble_end {
                0u8 => false,
                1u8 => true,
                _ => panic!("Invalid value in high_nibble_end"),
            },
        }
    }

    pub fn dump(&self, buf: &mut [u8]) {
        let toc = PackedToc::full(self);
        let rest = write_bump(buf, &toc).unwrap();
        write_bytes(rest, self.data).unwrap();
    }

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

    /// Returns a uncompressed vector that contains the same sequence as this PackedSeqView
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

pub fn total_bytes(seq: &PackedSeqView) -> usize {
    let num_elems = seq.len();
    let seq_size = if num_elems % 2 == 1 {
        (num_elems / 2) + 1
    } else {
        num_elems / 2
    };

    let toc_size = std::mem::size_of::<PackedToc>();
    toc_size + seq_size
}

pub fn import(filename: &str) -> Vec<Nucleotide> {
    let mmap = memfile::map_file(filename); // args[2] is the filename
    PackedSeqView::view(&mmap).get_elements()
}

pub fn export(seq: PackedSeqView, filename: &str) {
    let num_bytes = total_bytes(&seq);
    let mut mem = map_new_file(filename, num_bytes as u64);
    let mut buf = vec![0u8; num_bytes];
    let buf_ref = &mut buf;
    PackedSeqView::dump(&seq, buf_ref);
    mem[..buf_ref.len()].copy_from_slice(buf_ref);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

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
    fn test_export_import_simple() {
        let vec = PackedSeqStore::create(vec![
            Nucleotide::A,
            Nucleotide::C,
            Nucleotide::T,
            Nucleotide::G,
        ]);
        let input = vec.as_ref();
        let filename = "capra_test_file";
        let num_bytes = total_bytes(&input);
        let mut mem = map_new_file(filename, num_bytes as u64);
        let mut buf = vec![0u8; num_bytes];
        let buf_ref = &mut buf;
        input.dump(buf_ref);
        mem[..buf_ref.len()].copy_from_slice(buf_ref);
        let result: &[u8] = &mem;
        let output = PackedSeqView::view(result);
        assert_eq!(input.get(0), output.get(0));
        assert_eq!(input.get(1), output.get(1));
        assert_eq!(input.get(2), output.get(2));
        assert_eq!(input.get(3), output.get(3));
        std::mem::drop(mem);
        let _ = std::fs::remove_file(filename);
    }

    #[test]
    fn test_export_import() {
        let len = 10;
        let num_trials = 10;
        let mut rng = rand::thread_rng();
        for _ in 0..num_trials {
            let mut vec: Vec<Nucleotide> = Vec::new();
            for _ in 0..len {
                let rand_num = rng.gen_range(0..=3);
                match rand_num {
                    0 => vec.push(Nucleotide::A),
                    1 => vec.push(Nucleotide::C),
                    2 => vec.push(Nucleotide::T),
                    3 => vec.push(Nucleotide::G),
                    _ => panic!("Incorrect item appended!"),
                }
            }
            let old_vec = vec.clone();
            let store = PackedSeqStore::create(vec);
            let view = store.as_ref();
            let filename = "capra_test_file_2";
            export(view, filename);
            let new_vec = import(filename);
            let _ = std::fs::remove_file(filename);
            assert_eq!(old_vec, new_vec);
        }
    }
}
