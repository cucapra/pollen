use std::fmt;
use std::ops::Index;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Nucleotide {
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
            4_u8..=u8::MAX => panic!("Not a Nucleotide!"),
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
/// data: a vector that stores a compressed encoding of this vec's sequence
/// high_nibble_end: true if the final base pair in the sequence is stored at a
///                   high nibble
struct PackedVec {
    data: Vec<u8>,
    high_nibble_end: bool,
}

impl PackedVec {
    fn new() -> Self {
        PackedVec {
            data: Vec::new(),
            high_nibble_end: true,
        }
    }
    pub fn push(&mut self, input: Nucleotide) {
        let value = input.into();
        assert!(value <= 0xF);
        if self.high_nibble_end {
            self.data.push(value);
            self.high_nibble_end = false;
        } else {
            let last_index = self.data.len() - 1;
            self.data[last_index] = (value << 4) | self.data[last_index];
            self.high_nibble_end = true;
        }
    }
}

impl fmt::Display for PackedVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let iter = PackedVecIterator::new(&self);
        let mut i = 0;
        for item in iter {
            if i == 0 {
                i = 1;
            } else {
                write!(f, ", ");
            }
            println!("u8!!! {}", item);
            let n: Nucleotide = item.into();
            let c: char = n.into();
            println!("CHAR!!! {}", c);
            write!(f, "{}", c)?;
        }
        write!(f, "]")
    }
}

struct PackedVecIterator<'a> {
    data: &'a Vec<u8>,
    cur_index: usize,
    cur_high_nibble: bool,
    high_nibble_end: bool,
    last: bool,
}

impl<'a> PackedVecIterator<'a> {
    pub fn new(vec: &'a PackedVec) -> Self {
        Self {
            data: &vec.data,
            cur_index: 0,
            cur_high_nibble: false,
            high_nibble_end: vec.high_nibble_end,
            last: false,
        }
    }
}

impl<'a> Iterator for PackedVecIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index >= self.data.len() || self.last {
            return None;
        }
        if self.cur_index == self.data.len() - 1 && !self.high_nibble_end {
            self.last = true;
        }
        let result = if self.cur_high_nibble {
            (self.data[self.cur_index] >> 4) & 0b00001111u8
        } else {
            self.data[self.cur_index] & 0b00001111u8
        };
        if self.cur_high_nibble {
            self.cur_index += 1;
        }
        self.cur_high_nibble = !self.cur_high_nibble;
        Some(result)
    }
}

/// A reference to a subsection of a nucleotide sequence stored in a PackedVec
/// data_ref: The underlying vector that stores the sequence referenced by this slice
/// span: The specific section of the sequence that this slice references
struct PackedSlice<'a> {
    vec_ref: &'a PackedVec,
    span: std::ops::Range<u8>,
}

fn get_seq(vec_ref: &PackedVec, span: std::ops::Range<u8>) -> Vec<Nucleotide> {
    let mut arr: Vec<Nucleotide> = Vec::with_capacity((span.end - span.start).into());
    for i in span.start..=span.end {
        arr.push(get_vec_elem(vec_ref, i));
    }
    return arr;
}

/// Returns the element of *seq* at index *index*
fn get_vec_elem(seq: &PackedVec, index: u8) -> Nucleotide {
    if index % 2 == 1 {
        let i: usize = (index / 2) as usize;
        return ((&seq.data[i] & 0b11110000u8) >> 4).into();
    } else {
        let i: usize = (index / 2) as usize;
        return (&seq.data[i] & 0b00001111u8).into();
    }
}

/// Sets the element of *seq* at index *index* to *elem*
fn set_vec_elem(seq: &mut PackedVec, index: u8, input: Nucleotide) {
    let elem: u8 = input.into();
    if index % 2 == 1 {
        let i: usize = (index / 2) as usize;
        println!("i: {}", i);
        seq.data[i] = (0b00001111u8 & seq.data[i]) | (elem << 4);
    } else {
        let i: usize = (index / 2) as usize;
        seq.data[i] = (0b11110000u8 & seq.data[i]) | elem;
    }
}

/// Returns a uncompressed vector that contains the sequence in *seq*
fn get_vec_seq(seq: &PackedVec) -> Vec<Nucleotide> {
    let end_index = if (seq.high_nibble_end) {
        (seq.data.len() * 2) - 1
    } else {
        (seq.data.len() * 2) - 2
    };
    let span: std::ops::Range<u8> = std::ops::Range {
        start: 0,
        end: end_index as u8,
    };
    return get_seq(seq, span);
}

/// Returns a compressed PackedVec given an uncompressed vector *arr*
fn create_vec(arr: Vec<Nucleotide>) -> PackedVec {
    let mut new_vec = PackedVec::new();
    for i in 0..arr.len() {
        new_vec.push(arr[i]);
    }
    return new_vec;
}

/// Returns a PackedSlice given a compressed PackVec *vec* that acts as a reference
/// to the section of *vec* contained within the index bounds of Span *s*.
fn create_slice<'a>(vec: &'a PackedVec, s: std::ops::Range<u8>) -> PackedSlice<'a> {
    return PackedSlice {
        vec_ref: vec,
        span: s,
    };
}

/// Returns a vector containing the base pairs referenced by *slice*
fn get_slice_seq<'a>(slice: PackedSlice<'a>) -> Vec<Nucleotide> {
    return get_seq(slice.vec_ref, slice.span);
}

#[test]
fn test_vec() {
    let mut vec = create_vec(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
        Nucleotide::A,
    ]);
    vec.push(Nucleotide::A);
    let arr = get_vec_seq(&vec);
    assert_eq!(arr[0], Nucleotide::A);
    assert_eq!(arr[1], Nucleotide::C);
    assert_eq!(arr[2], Nucleotide::G);
    assert_eq!(arr[3], Nucleotide::T);
    assert_eq!(arr[4], Nucleotide::A);
    assert_eq!(arr[5], Nucleotide::A);
}

#[test]
fn test_vec_push() {
    let mut vec = create_vec(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
    ]);
    vec.push(Nucleotide::A);
    vec.push(Nucleotide::C);
    vec.push(Nucleotide::G);
    vec.push(Nucleotide::T);
    let arr = get_vec_seq(&vec);
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
    let span: std::ops::Range<u8> = std::ops::Range { start: 1, end: 4 };
    let mut vec = create_vec(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
        Nucleotide::A,
        Nucleotide::G,
    ]);
    let mut slice = create_slice(&vec, span);
    let arr = get_slice_seq(slice);
    assert_eq!(arr[0], Nucleotide::C);
    assert_eq!(arr[1], Nucleotide::G);
    assert_eq!(arr[2], Nucleotide::T);
    assert_eq!(arr[3], Nucleotide::A);
}

#[test]
fn test_display_even() {
    let mut vec = create_vec(vec![
        Nucleotide::C,
        Nucleotide::A,
        Nucleotide::T,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::C,
    ]);
    assert_eq!("[C, A, T, C, G, C]", vec.to_string());
}

#[test]
fn test_display_single() {
    let mut vec = create_vec(vec![Nucleotide::T.into()]);
    assert_eq!("[T]", vec.to_string());
}

#[test]
fn test_display_odd() {
    let mut vec = create_vec(vec![
        Nucleotide::C,
        Nucleotide::A,
        Nucleotide::T,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::C,
        Nucleotide::C,
    ]);
    assert_eq!("[C, A, T, C, G, C, C]", vec.to_string());
}

#[test]
fn test_getter_setter() {
    let mut vec = create_vec(vec![
        Nucleotide::A,
        Nucleotide::A,
        Nucleotide::T,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::C,
        Nucleotide::C,
    ]);
    assert_eq!(get_vec_elem(&mut vec, 0), Nucleotide::A);
    assert_eq!(get_vec_elem(&mut vec, 1), Nucleotide::A);
    set_vec_elem(&mut vec, 1, Nucleotide::G);
    assert_eq!(get_vec_elem(&mut vec, 1), Nucleotide::G);
}
