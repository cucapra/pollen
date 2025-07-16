use crate::memfile::map_new_file;
use crate::pool::{Pool, Span};
use crate::{file::*, SeqSpan};
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

impl Nucleotide {
    pub fn complement(&self) -> Nucleotide {
        match self {
            Nucleotide::A => Nucleotide::T,
            Nucleotide::T => Nucleotide::A,
            Nucleotide::C => Nucleotide::G,
            Nucleotide::G => Nucleotide::C,
        }
    }
}

impl From<char> for Nucleotide {
    fn from(value: char) -> Self {
        match value {
            'A' => Self::A,
            'C' => Self::C,
            'T' => Self::T,
            'G' => Self::G,
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

    /// The final bit of the final byte that contains Nucleotide data
    high_nibble_end: u8,

    /// The number of Nucleotide elements stored per byte of data
    pub elems_per_byte: u8,
}

pub struct PackedSeqView<'a> {
    pub data: &'a [u8],

    /// The final bit of the final byte that contains Nucleotide data
    pub high_nibble_end: u8,

    /// The first bit of the first byte that contains Nucleotide data
    pub high_nibble_begin: u8,

    /// The number of Nucleotide elements stored per byte of data
    pub elems_per_byte: u8,
}

#[derive(FromBytes, IntoBytes, Debug, KnownLayout, Immutable)]
#[repr(packed)]
pub struct PackedToc {
    magic: u64,
    data: Size,
    high_nibble_end: u8,
    high_nibble_begin: u8,
    elems_per_byte: u8,
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
            high_nibble_end: seq.high_nibble_end,
            high_nibble_begin: seq.high_nibble_begin,
            elems_per_byte: seq.elems_per_byte,
        }
    }

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
            data: data.into(),
            high_nibble_end: toc.high_nibble_end,
            high_nibble_begin: toc.high_nibble_begin,
            elems_per_byte: toc.elems_per_byte,
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
        if self.elems_per_byte == 2 {
            let begin = if self.high_nibble_begin == 4 { 1 } else { 0 };
            let end = if self.high_nibble_end == 7 { 0 } else { 1 };
            self.data.len() * 2 - begin - end
        } else if self.elems_per_byte == 4 {
            let begin = (self.high_nibble_begin / 2) as usize;
            let end = ((7 - self.high_nibble_end) / 2) as usize;
            self.data.len() * 4 - begin - end
        } else {
            panic!("Invalid number of elems per byte");
        }
    }

    /// Returns true if this PackedSeqView references an empty sequence, returns false otherwise
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the element of this PackedSeqView at index `index`
    pub fn get(&self, index: usize) -> Nucleotide {
        if self.elems_per_byte == 2 {
            let real_idx = index + (self.high_nibble_begin as usize / 4); // Add 1 to index if first data is at a high nibble
            let i = real_idx / 2;
            if real_idx % 2 == 1 {
                ((self.data[i] & 0b11110000u8) >> 4).into()
            } else {
                (self.data[i] & 0b00001111u8).into()
            }
        } else if self.elems_per_byte == 4 {
            let real_idx = index + (self.high_nibble_begin as usize / 2);
            let i = real_idx / 4;
            let j = real_idx % 4;
            ((self.data[i] >> (6 - 2 * j)) & 0b00000011u8).into()
        } else {
            panic!("Invalid number of elems per byte");
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

    pub fn slice(&self, span: SeqSpan) -> Self {
        let new_data = &self.data[span.start..span.end + 1];

        Self {
            data: new_data,
            high_nibble_begin: span.high_nibble_begin,
            high_nibble_end: span.high_nibble_end,
            elems_per_byte: span.elems_per_byte,
        }
    }

    pub fn from_pool(pool: Pool<'a, u8>, span: SeqSpan) -> Self {
        let slice = &pool.all()[span.start..span.end];
        Self {
            data: slice,
            high_nibble_begin: span.high_nibble_begin,
            high_nibble_end: span.high_nibble_end,
            elems_per_byte: span.elems_per_byte, // TODO!!!
        }
    }

    pub fn iter(&'a self) -> PackedSeqViewIterator<'a> {
        PackedSeqViewIterator {
            data: self,
            cur_index: 0,
            back_index: self.len(),
        }
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
        if self.cur_index < self.back_index {
            self.back_index -= 1;
            Some(self.data.get(self.back_index))
        } else {
            None
        }
    }
}

impl PackedSeqStore {
    /// Creates a new empty PackedSeqStore
    pub fn new(compression_val: u8) -> Self {
        assert!(compression_val == 2 || compression_val == 4);
        PackedSeqStore {
            data: Vec::new(),
            high_nibble_end: 7,
            elems_per_byte: compression_val,
        }
    }

    /// Returns a compressed PackedSeqStore given an uncompressed slice `arr`
    pub fn create_from_nucleotides(arr: &[Nucleotide], compression_val: u8) -> Self {
        let mut new_vec = PackedSeqStore::new(compression_val);
        for item in arr {
            new_vec.push(*item);
        }
        new_vec
    }

    /// Appends `input` to the end of this PackedSeqStore
    pub fn push(&mut self, input: Nucleotide) {
        let value = input.into();
        if self.elems_per_byte == 2 {
            if self.high_nibble_end == 7 {
                self.data.push(value);
                self.high_nibble_end = 4;
            } else {
                // self.high_nibble_end == 3
                let last_index = self.data.len() - 1;
                self.data[last_index] |= value << 4;
                self.high_nibble_end = 7;
            }
        } else {
            // self.elems_per_byte == 4
            if self.high_nibble_end == 7 {
                self.data.push(value << 6);
                self.high_nibble_end = 1;
            } else {
                // self.high_nibble_end == 3
                let last_index = self.data.len() - 1;
                self.data[last_index] |= value << (5 - self.high_nibble_end);
                self.high_nibble_end += 2;
            }
        }
    }

    /// Sets the element of this PackedSeqStore at index `index` to `elem`
    pub fn set(&mut self, index: usize, input: Nucleotide) {
        let elem: u8 = input.into();
        if self.elems_per_byte == 2 {
            let i = index / 2;
            if index % 2 == 1 {
                self.data[i] = (0b00001111u8 & self.data[i]) | (elem << 4);
            } else {
                self.data[i] = (0b11110000u8 & self.data[i]) | elem;
            }
        } else {
            // self.elems_per_byte == 4
            let i = index / 4;
            let j = index % 4;
            if j == 0 {
                self.data[i] = (0b00111111u8 & self.data[i]) | (elem << 6);
            } else if j == 1 {
                self.data[i] = (0b11001111u8 & self.data[i]) | (elem << 4);
            } else if j == 2 {
                self.data[i] = (0b11110011u8 & self.data[i]) | (elem << 2);
            } else if j == 3 {
                self.data[i] = (0b11111100u8 & self.data[i]) | elem;
            }
        }
    }

    pub fn as_ref(&self) -> PackedSeqView {
        PackedSeqView {
            data: &self.data,
            high_nibble_end: self.high_nibble_end,
            high_nibble_begin: 0,
            elems_per_byte: self.elems_per_byte,
        }
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

pub fn export(seq: PackedSeqView, filename: &str) {
    let num_bytes = seq.file_size();
    let mut mem = map_new_file(filename, num_bytes as u64);
    seq.write_file(&mut mem);
}

pub fn compress_into_buffer(input: &[u8], output: &mut Vec<u8>) -> bool {
    let mut high_nibble_end = true;
    for item in input {
        if high_nibble_end {
            output.push(*item);
            high_nibble_end = false;
        } else {
            let last_index = output.len() - 1;
            output[last_index] |= item << 4;
            high_nibble_end = true;
        }
    }
    high_nibble_end
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::ThreadRng, thread_rng, Rng};

    #[test]
    fn test_vec() {
        let mut vec = PackedSeqStore::create_from_nucleotides(
            &[
                Nucleotide::A,
                Nucleotide::C,
                Nucleotide::G,
                Nucleotide::T,
                Nucleotide::A,
            ],
            2,
        );
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
        let mut vec = PackedSeqStore::create_from_nucleotides(
            &[Nucleotide::A, Nucleotide::C, Nucleotide::G, Nucleotide::T],
            2,
        );
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
        let vec = PackedSeqStore::create_from_nucleotides(
            &[
                Nucleotide::A,
                Nucleotide::C,
                Nucleotide::G,
                Nucleotide::T,
                Nucleotide::A,
                Nucleotide::G,
            ],
            2,
        );
        let slice = create_slice(&vec, span);
        let arr = get_slice_seq(slice);
        assert_eq!(arr[0], Nucleotide::C);
        assert_eq!(arr[1], Nucleotide::G);
        assert_eq!(arr[2], Nucleotide::T);
        assert_eq!(arr[3], Nucleotide::A);
    }

    #[test]
    fn test_display_even() {
        let vec = PackedSeqStore::create_from_nucleotides(
            &[
                Nucleotide::C,
                Nucleotide::A,
                Nucleotide::T,
                Nucleotide::C,
                Nucleotide::G,
                Nucleotide::C,
            ],
            2,
        );
        assert_eq!("[C, A, T, C, G, C]", vec.as_ref().to_string());
    }

    #[test]
    fn test_display_single() {
        let vec = PackedSeqStore::create_from_nucleotides(&[Nucleotide::T.into()], 2);
        assert_eq!("[T]", vec.as_ref().to_string());
    }

    #[test]
    fn test_display_odd() {
        let vec = PackedSeqStore::create_from_nucleotides(
            &[
                Nucleotide::C,
                Nucleotide::A,
                Nucleotide::T,
                Nucleotide::C,
                Nucleotide::G,
                Nucleotide::C,
                Nucleotide::C,
            ],
            2,
        );
        assert_eq!("[C, A, T, C, G, C, C]", vec.as_ref().to_string());
    }

    #[test]
    fn test_getter_setter() {
        let mut vec = PackedSeqStore::create_from_nucleotides(
            &[
                Nucleotide::A,
                Nucleotide::A,
                Nucleotide::T,
                Nucleotide::C,
                Nucleotide::G,
                Nucleotide::C,
                Nucleotide::C,
            ],
            2,
        );
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
            let store = PackedSeqStore::create_from_nucleotides(&vec, 2);
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
            let store = PackedSeqStore::create_from_nucleotides(&vec, 2);

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

    #[test]
    fn test_subslice() {
        let store = PackedSeqStore::create_from_nucleotides(
            &[Nucleotide::A, Nucleotide::C, Nucleotide::T, Nucleotide::G],
            2,
        );
        let view = store.as_ref();
        let subslice = view.slice(SeqSpan {
            start: 0,
            end: 1,
            high_nibble_begin: 4,
            high_nibble_end: 3,
            elems_per_byte: 2,
        });
        assert_eq!(2, subslice.len());
        assert_eq!(Nucleotide::C, subslice.get(0));
        assert_eq!(Nucleotide::T, subslice.get(1));
    }

    #[test]
    fn test_from_pool() {
        let store = PackedSeqStore::create_from_nucleotides(
            &[Nucleotide::A, Nucleotide::C, Nucleotide::T, Nucleotide::G],
            2,
        );
        let view = store.as_ref();
        let pool = Pool::from(view.data);
        let span = SeqSpan {
            start: 0,
            end: 1,
            high_nibble_begin: 0,
            high_nibble_end: 3,
            elems_per_byte: 2,
        };
        let sub_view = PackedSeqView::from_pool(pool, span);
        assert_eq!(3, sub_view.len());
        assert_eq!(Nucleotide::A, sub_view.get(0));
        assert_eq!(Nucleotide::C, sub_view.get(1));
        assert_eq!(Nucleotide::T, sub_view.get(2));
    }
}
