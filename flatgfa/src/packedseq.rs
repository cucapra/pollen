#![allow(clippy::repr_packed_without_abi)]

use crate::memfile::map_new_file;
use crate::pool::Pool;
use crate::{file::*, SeqSpan};
use std::fmt::{self, Write};
use zerocopy::*;

const MAGIC_NUMBER: u64 = 0x12;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Nucleotide {
    A,
    C,
    T,
    G,
    N,
}

impl From<char> for Nucleotide {
    fn from(value: char) -> Self {
        match value {
            'A' => Self::A,
            'C' => Self::C,
            'T' => Self::T,
            'G' => Self::G,
            'N' => Self::N,
            _ => panic!("Not a Nucleotide!"),
        }
    }
}

impl From<u8> for Nucleotide {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::A,
            1 => Self::C,
            2 => Self::T,
            3 => Self::G,
            4 => Self::N,
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
            Nucleotide::N => 4,
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
            Nucleotide::N => 'N',
        }
    }
}

impl Nucleotide {
    /// Get a nucleotide from an ASCII byte. This is separate from the
    /// `From<u8>` impl, which converts to and from compact values.
    pub fn from_ascii(value: u8) -> Self {
        match value {
            b'A' => Self::A,
            b'C' => Self::C,
            b'T' => Self::T,
            b'G' => Self::G,
            b'N' => Self::N,
            _ => panic!("Not a Nucleotide!"),
        }
    }

    /// Get the ASCII character for this nucleotide.
    pub fn to_ascii(&self) -> u8 {
        match self {
            Nucleotide::A => b'A',
            Nucleotide::C => b'C',
            Nucleotide::T => b'T',
            Nucleotide::G => b'G',
            Nucleotide::N => b'N',
        }
    }

    pub fn complement(&self) -> Nucleotide {
        match self {
            Nucleotide::A => Nucleotide::T,
            Nucleotide::T => Nucleotide::A,
            Nucleotide::C => Nucleotide::G,
            Nucleotide::G => Nucleotide::C,
            Nucleotide::N => Nucleotide::N,
        }
    }
}

/// A compressed vector-like structure for storing nucleotide sequences
///     - Two base pairs are stored per byte
///
pub struct PackedSeqStore {
    /// A vector that stores a compressed encoding of this PackedSeqStore's sequence
    pub data: Vec<u8>,

    /// True if the final base pair in the sequence is stored at a
    ///                   high nibble
    pub high_nibble_end: bool,
}

pub struct PackedSeqView<'a> {
    pub data: &'a [u8],

    /// True if the final base pair in the sequence is stored at a
    ///                   high nibble
    pub high_nibble_end: bool,

    /// True if the first base pair in the sequence is stored at a
    ///                   high nibble
    pub high_nibble_begin: bool,
}

#[derive(FromBytes, IntoBytes, Debug, KnownLayout, Immutable)]
#[repr(packed)]
pub struct PackedToc {
    magic: u64,
    data: Size,
    high_nibble_end: u8,
    high_nibble_begin: u8,
}

impl PackedToc {
    pub fn size(&self) -> usize {
        size_of::<Self>() + self.data.bytes::<u8>()
    }

    /// Returns a PackededToc with data corresponding to `seq`
    fn full(seq: &PackedSeqView) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            data: Size {
                len: seq.data.len(),
                capacity: seq.data.len(),
            },
            high_nibble_end: if seq.high_nibble_end { 1u8 } else { 0u8 },
            high_nibble_begin: if seq.high_nibble_begin { 1u8 } else { 0u8 },
        }
    }

    /// Returns whether `nibble` represents a high or low nibble
    fn get_nibble_bool(nibble: u8) -> bool {
        match nibble {
            0u8 => false,
            1u8 => true,
            _ => panic!("Invalid high_nibble_end value"),
        }
    }

    /// Given a reference to a memory-mapped file `data` containing a compressed
    /// sequence of nucleotides, return a PackedToc along with the rest of the data
    fn read(data: &[u8]) -> (&Self, &[u8]) {
        let toc = PackedToc::ref_from_prefix(data).unwrap().0;
        let rest = &data[size_of::<PackedToc>()..];
        let magic = toc.magic;
        assert_eq!(magic, MAGIC_NUMBER);
        (toc, rest)
    }
}

impl<'a> PackedSeqView<'a> {
    /// Returns the necessary size of a file for storing the data (and associated PackedToc)
    /// for this PackedSeqView
    pub fn file_size(&self) -> usize {
        let num_elems = self.len();
        let seq_size = if num_elems % 2 == 1 {
            (num_elems / 2) + 1
        } else {
            num_elems / 2
        };

        let toc_size = std::mem::size_of::<PackedToc>();
        toc_size + seq_size
    }

    /// Given a reference to a memory-mapped file `data` containing a compressed
    /// sequence of nucleotides, return a corresponding PackedSeqView
    pub fn read_file(data: &'a [u8]) -> Self {
        let (toc, rest) = PackedToc::read(data);

        let (data, _) = slice_prefix(rest, toc.data);
        Self {
            data,
            high_nibble_end: PackedToc::get_nibble_bool(toc.high_nibble_end),
            high_nibble_begin: PackedToc::get_nibble_bool(toc.high_nibble_begin),
        }
    }

    /// Given a mutable reference to a memory-mapped file `buf`, write the compressed sequence
    /// referenced by this PackedSeqView to `buf`
    pub fn write_file(&self, buf: &mut [u8]) {
        let toc = PackedToc::full(self);
        let rest = write_bump(buf, &toc).unwrap();
        write_bytes(rest, self.data).unwrap();
    }

    /// Returns the number of nucleotides in this PackedSeqView
    pub fn len(&self) -> usize {
        if self.data.is_empty() {
            0
        } else {
            let begin = if self.high_nibble_begin { 1 } else { 0 };
            let end = if self.high_nibble_end { 0 } else { 1 };
            self.data.len() * 2 - begin - end
        }
    }

    /// Returns true if this PackedSeqView references an empty sequence, returns false otherwise
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the element of this PackedSeqView at index `index`
    pub fn get(&self, index: usize) -> Nucleotide {
        let real_idx = if self.high_nibble_begin {
            index + 1
        } else {
            index
        };
        let i = real_idx / 2;
        if real_idx % 2 == 1 {
            ((self.data[i] & 0b11110000u8) >> 4).into()
        } else {
            (self.data[i] & 0b00001111u8).into()
        }
    }

    /// Returns an uncompressed vector that contains the same sequence as
    /// this PackedSeqView, in range `span`
    pub fn get_range(&self, span: std::ops::Range<usize>) -> Vec<Nucleotide> {
        let mut arr: Vec<Nucleotide> = Vec::with_capacity(span.end - span.start);
        for i in span.start..=span.end {
            arr.push(self.get(i));
        }
        arr
    }

    /// Returns an uncompressed vector that contains the same sequence as this PackedSeqView
    pub fn get_elements(&self) -> Vec<Nucleotide> {
        self.get_range(0..(self.len() - 1))
    }

    /// Creates a subslice of this PackedSeqView in the range of `span`
    pub fn slice(&self, span: SeqSpan) -> Self {
        let new_data = &self.data[span.start..span.end];

        Self {
            data: new_data,
            high_nibble_begin: PackedToc::get_nibble_bool(span.high_nibble_begin),
            high_nibble_end: PackedToc::get_nibble_bool(span.high_nibble_end),
        }
    }

    /// Given a pool of compressed data (`pool`), create a PackedSeqView in the range of `span`
    pub fn from_pool(pool: Pool<'a, u8>, span: SeqSpan) -> Self {
        let slice = &pool.all()[span.start..span.end];
        Self {
            data: slice,
            high_nibble_begin: PackedToc::get_nibble_bool(span.high_nibble_begin),
            high_nibble_end: PackedToc::get_nibble_bool(span.high_nibble_end),
        }
    }

    /// Iterate over the nucleotides in this sequence.
    pub fn iter(&self) -> PackedSeqViewIterator<'_> {
        PackedSeqViewIterator::new(self)
    }
}

impl fmt::Display for PackedSeqView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.iter() {
            f.write_char(item.into())?;
        }
        Ok(())
    }
}

pub struct PackedSeqViewIterator<'a> {
    data: &'a PackedSeqView<'a>,
    cur_index: usize,
    back_index: usize,
}

impl<'a> PackedSeqViewIterator<'a> {
    pub fn new(vec: &'a PackedSeqView<'a>) -> Self {
        Self {
            data: vec,
            cur_index: 0,
            back_index: vec.len(),
        }
    }
}

impl Iterator for PackedSeqViewIterator<'_> {
    type Item = Nucleotide;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index < self.back_index {
            self.cur_index += 1;
            Some(self.data.get(self.cur_index - 1))
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for PackedSeqViewIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cur_index < self.back_index && self.back_index > 0 {
            self.back_index -= 1;
            Some(self.data.get(self.back_index))
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

    /// Create a compressed PackedSeqStore given an uncompressed nucleotide
    /// sequence `arr`.
    pub fn from_slice(arr: &[Nucleotide]) -> Self {
        let mut new_vec = Self::new();
        for item in arr {
            new_vec.push(*item);
        }
        new_vec
    }

    pub fn from_ascii<T: Iterator<Item = u8>>(bytes: T) -> Self {
        let mut new_vec = Self::new();
        for b in bytes {
            new_vec.push(Nucleotide::from_ascii(b));
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
            self.data[i] = (0b00001111u8 & self.data[i]) | (elem << 4);
        } else {
            self.data[i] = (0b11110000u8 & self.data[i]) | elem;
        }
    }

    /// Creates a PackedSeqView with the same data as this PackedSeqStore
    pub fn as_ref(&self) -> PackedSeqView {
        PackedSeqView {
            data: &self.data,
            high_nibble_end: self.high_nibble_end,
            high_nibble_begin: false,
        }
    }
}

impl std::iter::FromIterator<Nucleotide> for PackedSeqStore {
    /// Create a new compressed sequence from any iterator that produces
    /// individual nucleotides.
    fn from_iter<I: IntoIterator<Item = Nucleotide>>(iter: I) -> Self {
        let mut new_vec = Self::new();
        for n in iter {
            new_vec.push(n);
        }
        new_vec
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

/// Writes `seq` into a file with name `filename`
pub fn export(seq: PackedSeqView, filename: &str) {
    let num_bytes = seq.file_size();
    let mut mem = map_new_file(filename, num_bytes as u64);
    seq.write_file(&mut mem);
}

/// Takes a slice of uncompressed ASCII-encoded base pairs, compresses them and pushes them into `output`
pub fn compress_into_buffer(input: &[u8], output: &mut Vec<u8>) -> bool {
    let mut high_nibble_end = true;
    for item in input {
        let converted: u8 = match item {
            65 => 0,
            67 => 1,
            84 => 2,
            71 => 3,
            78 => 4,
            _ => panic!("Not a Nucleotide!"),
        };
        if high_nibble_end {
            output.push(converted);
            high_nibble_end = false;
        } else {
            let last_index = output.len() - 1;
            output[last_index] |= converted << 4;
            high_nibble_end = true;
        }
    }
    high_nibble_end
}

/// Takes a slice of compressed base pairs, decompresses them and pushes them into `output`
pub fn decompress_into_buffer(input: PackedSeqView, output: &mut Vec<u8>) {
    if !input.high_nibble_begin {
        output.push(convert_to_ascii(input.data[0] & 0b00001111u8));
    }
    output.push(convert_to_ascii((input.data[0] & 0b11110000u8) >> 4));
    for item in &input.data[1..input.data.len() - 1] {
        output.push(convert_to_ascii(item & 0b00001111u8));
        output.push(convert_to_ascii((item & 0b11110000u8) >> 4));
    }
    output.push(convert_to_ascii(
        input.data[input.data.len() - 1] & 0b00001111u8,
    ));
    if input.high_nibble_end {
        output.push(convert_to_ascii(
            (input.data[input.data.len() - 1] & 0b11110000u8) >> 4,
        ));
    }
}

fn convert_to_ascii(elem: u8) -> u8 {
    match elem {
        0 => 65,
        1 => 67,
        2 => 84,
        3 => 71,
        4 => 78,
        _ => panic!("Not a Nucleotide!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::ThreadRng, thread_rng, Rng};

    #[test]
    fn test_vec() {
        let mut vec = PackedSeqStore::from_slice(&[
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
        let mut vec = PackedSeqStore::from_slice(&[
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
        let vec = PackedSeqStore::from_slice(&[
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
        let vec = PackedSeqStore::from_slice(&[
            Nucleotide::C,
            Nucleotide::A,
            Nucleotide::T,
            Nucleotide::C,
            Nucleotide::G,
            Nucleotide::C,
        ]);
        assert_eq!("CATCGC", vec.as_ref().to_string());
    }

    #[test]
    fn test_display_single() {
        let vec = PackedSeqStore::from_slice(&[Nucleotide::T.into()]);
        assert_eq!("T", vec.as_ref().to_string());
    }

    #[test]
    fn test_display_odd() {
        let vec = PackedSeqStore::from_slice(&[
            Nucleotide::C,
            Nucleotide::A,
            Nucleotide::T,
            Nucleotide::C,
            Nucleotide::G,
            Nucleotide::C,
            Nucleotide::C,
        ]);
        assert_eq!("CATCGCC", vec.as_ref().to_string());
    }

    #[test]
    fn test_getter_setter() {
        let mut vec = PackedSeqStore::from_slice(&[
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

    /// Randomly generate an uncompressed nucleotide sequence.
    fn random_seq(rng: &mut ThreadRng, len: usize) -> Vec<Nucleotide> {
        (0..len)
            .map(|_| Nucleotide::from(rng.gen_range(0..=3)))
            .collect()
    }

    /// Test the `get_elements` method that decompresses data to a
    /// `Vec<Nucleotide>` "in bulk."
    #[test]
    fn test_get_elements() {
        let len = 10;
        let num_trials = 10;
        let mut rng = thread_rng();

        for _ in 0..num_trials {
            let vec = random_seq(&mut rng, len);

            // "Round trip" through a compressed representation, producing a new
            // decompressed vector.
            let store = PackedSeqStore::from_slice(&vec);
            let new_vec = store.as_ref().get_elements();

            assert_eq!(vec, new_vec);
        }
    }

    /// Test conversion to and from a byte buffer (which we use to read and
    /// write files).
    #[test]
    fn test_bytes_export_import() {
        let len = 10;
        let num_trials = 10;
        let mut rng = rand::thread_rng();

        for _ in 0..num_trials {
            let vec = random_seq(&mut rng, len);
            let store = PackedSeqStore::from_slice(&vec);

            // Copy the compressed representation to a byte buffer.
            let seq = store.as_ref();
            let num_bytes = seq.file_size();
            let mut mem = vec![0u8; num_bytes];
            seq.write_file(&mut mem);

            // "Reawaken" a sequence from this byte buffer.
            let new_seq = PackedSeqView::read_file(&mem);

            assert_eq!(vec, new_seq.get_elements());
        }
    }
}
