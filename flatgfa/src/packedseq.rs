use std::ops::Index;
use std::fmt;
static A: u8 = 0;
static C: u8 = 1;
static G: u8 = 2;
static T: u8 = 3;

/// A compressed vector-like structure for storing nucleotide sequences
///     - Two base pairs are stored per byte
///
/// data: a vector that stores a compressed encoding of this vec's sequence
/// high_nibble_end: whether the final base pair in the sequence is stored at a 
///                   high or low nibble 
struct PackedVec {
    data: Vec<u8>,
    high_nibble_end: bool,
}

impl PackedVec {
    fn new() -> Self {
        PackedVec {
            data: Vec::new(),
            high_nibble_end: false,
        }
    }
    pub fn push(&mut self, value: u8) {

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

// impl Index<usize> for PackedVec {
//     type Output = u8;
//     fn index(&self, index: usize) -> &Self::Output {
//         if index % 2 == 1 {
//             (&self.data[index as usize / 2] & 0b11110000u8) >> 4
//         } else {
//             &self.data[index as usize / 2] & 0b00001111u8
//         }
//     }
// }

impl fmt::Display for PackedVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let iter = PackedVecIterator {
            data: &self.data,
            cur_index: 0,
            cur_high_nibble: false,
            high_nibble_end: self.high_nibble_end,
            last: false
        };
        let mut i = 0;
        for item in iter {
            if i == 0 {
                write!(f, "{}", match item {0 => 'A', 1 => 'C', 2 => 'G', 3 => 'T', _ => ' '})?; 
                i = 1; 
            } else {
            write!(f, ", {}",  match item {0 => 'A', 1 => 'C', 2 => 'G', 3 => 'T', _ => ' '})?;
            }
        }
        write!(f, "]")
    }
}

struct PackedVecIterator<'a> {
    data: &'a Vec<u8>,
    cur_index: usize,
    cur_high_nibble: bool,
    high_nibble_end: bool,
    last: bool
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
        }
        else {
            self.data[self.cur_index]& 0b00001111u8  
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
    data_ref: &'a Vec<u8>,
    span: Span
}


/// Used by PackedSlices to reference a subsection of a nucleotide sequence  
struct Span {
    start: u8,
    end: u8,
}


fn get_seq(compressed_data: &Vec<u8> , span: Span)  -> Vec<u8> {
    let mut arr: Vec<u8> = Vec::with_capacity((span.end - span.start).into());
    for i in span.start..=span.end {
        if i % 2 == 1 {
            arr.push((compressed_data[i as usize / 2] & 0b11110000u8) >> 4);
        } else {
            arr.push(compressed_data[i as usize / 2] & 0b00001111u8);
        }
    }
    return arr;
}

/// Returns the element of *seq* at index *index*
fn get_vec_elem(seq: &mut PackedVec, index: u8) -> u8 {
    if index % 2 == 1 {
        let i: usize = (index / 2) as usize;
        (&seq.data[i] & 0b11110000u8) >> 4
    } else {
        let i: usize = (index / 2) as usize;
        &seq.data[i] & 0b00001111u8
    }
}
 
/// Sets the element of *seq* at index *index* to *elem*
fn set_vec_elem(seq: &mut PackedVec, index: u8, elem: u8) {
    if index % 2 == 1 {
        let i: usize = (index / 2) as usize;
        println!("i: {}",i);
        seq.data[i] =  (0b00001111u8 & seq.data[i]) | (elem << 4);
    } else {
        let i: usize = (index / 2) as usize;
        seq.data[i] = (0b11110000u8 & seq.data[i]) | elem;
    }
}

/// Returns a uncompressed vector that contains the sequence in *seq*
fn get_vec_seq(seq: &PackedVec) -> Vec<u8> {
    let end_index = if (seq.high_nibble_end) {
        (seq.data.len() * 2) - 1 
    } else {
        (seq.data.len() * 2) - 2
    };
    let span = Span {start: 0, end: end_index as u8};
    return get_seq(&seq.data, span);
}

/// Returns a compressed PackedVec given an uncompressed vector *arr*
fn create_vec(arr: Vec<u8>) -> PackedVec {
    let mut high_nibble = true;
    if arr.len() % 2 == 1 {
        high_nibble = false;
    }
    let mut new_data = Vec::with_capacity(if high_nibble {
        (arr.len() / 2) + 1
    } else {
        arr.len()
    });
    let mut j = 0;
    let mut low = true;
    for i in 0..arr.len() {
        if low == true {
            new_data.push(arr[i]);
            low = false;
        } else {
            new_data[j] = new_data[j] | (arr[i] << 4);
            low = true;
            j += 1;
        }
    }
    return PackedVec {
        data: new_data,
        high_nibble_end: high_nibble,
    };
}

/// Returns a PackedSlice given a compressed PackVec *vec* that acts as a reference 
/// to the section of *vec* contained within the index bounds of Span *s*.
fn create_slice<'a>(vec: &'a PackedVec, s: Span) -> PackedSlice<'a> {
    return PackedSlice{data_ref: &vec.data, span: s};
}

/// Returns a vector containing the base pairs referenced by *slice*
fn get_slice_seq<'a>(slice: PackedSlice<'a>) -> Vec<u8> {
    return get_seq(slice.data_ref, slice.span);
}

fn print_arr(arr: Vec<u8>) {
    let new_arr: Vec<char> = arr
        .into_iter()
        .map(|e| match e {
            0 => 'A',
            1 => 'C',
            2 => 'T',
            3 => 'G',
            _ => ' ',
        })
        .collect();
    println!("{:?}", new_arr);
}

fn main() {
    let mut vec = create_vec(vec![C, A, T, C, G, C]);
    vec.push(C);
    let arr = get_vec_seq(&vec);
    print_arr(arr);
    println!("{}", vec);
}


#[test]
fn test_vec() {
    let mut vec = create_vec(vec![A, C, G, T, A]);
    vec.push(A);
    let arr = get_vec_seq(&vec);
    assert_eq!(arr[0], A); 
    assert_eq!(arr[1], C);
    assert_eq!(arr[2], G); 
    assert_eq!(arr[3], T); 
    assert_eq!(arr[4], A); 
    assert_eq!(arr[5], A); 
}

#[test]
fn test_vec_push() {
    let mut vec = create_vec(vec![A, C, G, T]);
    vec.push(A);
    vec.push(C);
    vec.push(G);
    vec.push(T);
    let arr = get_vec_seq(&vec);
    assert_eq!(arr[0], A);
    assert_eq!(arr[1], C);
    assert_eq!(arr[2], G);
    assert_eq!(arr[3], T);
    assert_eq!(arr[4], A);
    assert_eq!(arr[5], C);
    assert_eq!(arr[6], G);
    assert_eq!(arr[7], T);
}

#[test]
fn test_slice() {
    let span = Span {start: 1, end: 4};
    let mut vec = create_vec(vec![A, C, G, T, A, G]);
    let mut slice = create_slice(&vec, span);
    let arr = get_slice_seq(slice);
    assert_eq!(arr[0], C);
    assert_eq!(arr[1], G);
    assert_eq!(arr[2], T);
    assert_eq!(arr[3], A);
}

#[test]
fn test_display_even() {
    let mut vec = create_vec(vec![C, A, T, C, G, C]);
    assert_eq!("[C, A, T, C, G, C]", vec.to_string());
}

#[test]
fn test_display_odd() {
    let mut vec = create_vec(vec![C, A, T, C, G, C, C]);
    assert_eq!("[C, A, T, C, G, C, C]", vec.to_string());
}

#[test]
fn test_getter_setter() {
    let mut vec = create_vec(vec![A, A, T, C, G, C, C]);
    assert_eq!(get_vec_elem(&mut vec, 0), A);
    assert_eq!(get_vec_elem(&mut vec, 1), A);
    set_vec_elem(&mut vec, 1, G);
    assert_eq!(get_vec_elem(&mut vec, 1), G);
}
