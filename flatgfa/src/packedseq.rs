use std::fmt;

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
struct PackedVec {
    /// A vector that stores a compressed encoding of this PackedVec's sequence
    data: Vec<u8>,

    /// True if the final base pair in the sequence is stored at a
    ///                   high nibble
    high_nibble_end: bool,
}

impl PackedVec {
    /// Creates a new empty PackedVec
    pub fn new() -> Self {
        PackedVec {
            data: Vec::new(),
            high_nibble_end: true,
        }
    }

    /// Returns a compressed PackedVec given an uncompressed vector `arr`
    fn create(arr: Vec<Nucleotide>) -> Self {
        let mut new_vec = PackedVec::new();
        for i in 0..arr.len() {
            new_vec.push(arr[i]);
        }
        return new_vec;
    }

    /// Appends `input` to the end of this PackedVec
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

    pub fn len(&self) -> usize {
        if self.high_nibble_end {
            return self.data.len() * 2;
        } else {
            return self.data.len() * 2 - 1;
        }
    }

    /// Returns the element of this PackedVec at index `index`
    pub fn get(&self, index: usize) -> Nucleotide {
        let i = index / 2;
        if index % 2 == 1 {
            return ((self.data[i] & 0b11110000u8) >> 4).into();
        } else {
            return (self.data[i] & 0b00001111u8).into();
        }
    }

    /// Sets the element of this PackedVec at index `index` to `elem`
    fn set(&mut self, index: usize, input: Nucleotide) {
        let elem: u8 = input.into();
        let i = index / 2;
        if index % 2 == 1 {
            println!("i: {}", i);
            self.data[i] = (0b00001111u8 & self.data[i]) | (elem << 4);
        } else {
            self.data[i] = (0b11110000u8 & self.data[i]) | elem;
        }
    }

    fn get_range(&self, span: std::ops::Range<usize>) -> Vec<Nucleotide> {
        let mut arr: Vec<Nucleotide> = Vec::with_capacity((span.end - span.start).into());
        for i in span.start..=span.end {
            arr.push(self.get(i));
        }
        return arr;
    }

    /// Returns a uncompressed vector that contains the same sequence as this PackedVec
    fn get_elements(&self) -> Vec<Nucleotide> {
        return self.get_range(0..(self.len() - 1));
    }
}

impl fmt::Display for PackedVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut i = 0;
        for item in PackedVecIterator::new(&self) {
            if i == 0 {
                i = 1;
            } else {
                write!(f, ", ");
            }
            let n: Nucleotide = item.into();
            let c: char = n.into();
            write!(f, "{}", c)?;
        }
        write!(f, "]")
    }
}

struct PackedVecIterator<'a> {
    data: &'a PackedVec,
    cur_index: usize,
}

impl<'a> PackedVecIterator<'a> {
    pub fn new(vec: &'a PackedVec) -> Self {
        Self {
            data: vec,
            cur_index: 0,
        }
    }
}

impl<'a> Iterator for PackedVecIterator<'a> {
    type Item = Nucleotide;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index < self.data.len() {
            self.cur_index += 1;
            Some(self.data.get(self.cur_index - 1))
        } else {
            return None;
        }
    }
}

/// A reference to a subsection of a nucleotide sequence stored in a PackedVec
struct PackedSlice<'a> {
    /// The underlying vector that stores the sequence referenced by this slice
    vec_ref: &'a PackedVec,

    /// The specific section of the sequence that this slice references
    span: std::ops::Range<usize>,
}

/// Returns a PackedSlice given a compressed PackVec `vec` that acts as a reference
/// to the section of `vec` contained within the index bounds of Span `s`.
fn create_slice<'a>(vec: &'a PackedVec, s: std::ops::Range<usize>) -> PackedSlice<'a> {
    return PackedSlice {
        vec_ref: vec,
        span: s,
    };
}

/// Returns a vector containing the base pairs referenced by `slice`
fn get_slice_seq<'a>(slice: PackedSlice<'a>) -> Vec<Nucleotide> {
    return slice.vec_ref.get_range(slice.span);
}

#[test]
fn test_vec() {
    let mut vec = PackedVec::create(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
        Nucleotide::A,
    ]);
    vec.push(Nucleotide::A);
    let arr = vec.get_elements();
    assert_eq!(arr[0], Nucleotide::A);
    assert_eq!(arr[1], Nucleotide::C);
    assert_eq!(arr[2], Nucleotide::G);
    assert_eq!(arr[3], Nucleotide::T);
    assert_eq!(arr[4], Nucleotide::A);
    assert_eq!(arr[5], Nucleotide::A);
}

#[test]
fn test_vec_push() {
    let mut vec = PackedVec::create(vec![
        Nucleotide::A,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::T,
    ]);
    vec.push(Nucleotide::A);
    vec.push(Nucleotide::C);
    vec.push(Nucleotide::G);
    vec.push(Nucleotide::T);
    let arr = vec.get_elements();
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
    let vec = PackedVec::create(vec![
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
    let vec = PackedVec::create(vec![
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
    let vec = PackedVec::create(vec![Nucleotide::T.into()]);
    assert_eq!("[T]", vec.to_string());
}

#[test]
fn test_display_odd() {
    let vec = PackedVec::create(vec![
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
    let mut vec = PackedVec::create(vec![
        Nucleotide::A,
        Nucleotide::A,
        Nucleotide::T,
        Nucleotide::C,
        Nucleotide::G,
        Nucleotide::C,
        Nucleotide::C,
    ]);
    assert_eq!(vec.get(0), Nucleotide::A);
    assert_eq!(vec.get(1), Nucleotide::A);
    vec.set(1, Nucleotide::G);
    assert_eq!(vec.get(1), Nucleotide::G);
}
