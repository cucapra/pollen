use std::ops::Index;
use std::fmt;
static A: u8 = 0;
static C: u8 = 1;
static G: u8 = 2;
static T: u8 = 3;

///
///
///
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

impl Index<usize> for PackedVec {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

/// Prints out the sequence of data 
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

struct PackedSlice<'a> {
    data_ref: &'a Vec<u8>,
    span: Span
}

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

fn get_vec_seq(seq: &PackedVec) -> Vec<u8> {
    let end_index = if (seq.high_nibble_end) {
        (seq.data.len() * 2) - 1 
    } else {
        (seq.data.len() * 2) - 2
    };
    let span = Span {start: 0, end: end_index as u8};
    return get_seq(&seq.data, span);
}

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

fn create_slice<'a>(vec: &'a PackedVec, s: Span) -> PackedSlice<'a> {
    return PackedSlice{data_ref: &vec.data, span: s};
}

fn get_slice_seq<'a>(slice: PackedSlice<'a>, span: Span) -> Vec<u8> {
    return get_seq(slice.data_ref, span);
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
     let span = Span { start: 0, end: 4 };
    let mut seq = create_vec(vec![C, A, T, C, G, C]);
    seq.push(C);
    let arr = get_vec_seq(&seq);
    print_arr(arr);
   // println!("{}", seq);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vec_and_push() {
        let mut seq = create_vec(vec![A, C, G, T, A]);
        seq.push(A);
        let arr = get_vec_seq(seq);
        assert_eq!(arr[0], A); 
        assert_eq!(arr[1], C);
        assert_eq!(arr[2], G); 
        assert_eq!(arr[2], T); 
        assert_eq!(arr[2], A); 
        assert_eq!(arr[2], A); 
    }

    #[test]
    fn test_pushing_multiple_values() {
        let mut seq = create_vec(vec![A, C, G, T]);
        seq.push(A);
        seq.push(C);
        seq.push(G);
        seq.push(T);
        let arr = get_vec_seq(seq);
        assert_eq!(arr[0], A);
        assert_eq!(arr[1], C);
        assert_eq!(arr[2], G);
        assert_eq!(arr[3], T);
        assert_eq!(arr[4], A);
        assert_eq!(arr[5], C);
        assert_eq!(arr[6], G);
        assert_eq!(arr[7], T);
    }
}
