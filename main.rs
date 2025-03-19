static A: u8 = 0;
static C: u8 = 1;
static G: u8 = 2;
static T: u8 = 3;

struct Sequence {
    data: Vec<u8>,
    high_nibble_start: bool,
    high_nibble_end: bool,
}

struct Span {
    start: u8,
    end: u8,
}

fn get_seq(seq: &Sequence, span: Span) -> Vec<u8> {
    let mut arr: Vec<u8> = Vec::with_capacity((span.end - span.start).into());
    let mut j = 0;
    for i in span.start..=span.end {
        if i % 2 == 1 {
            arr.push((seq.data[i as usize / 2] & 0b11110000u8) >> 4);
        } else {
            arr.push(seq.data[i as usize / 2] & 0b00001111u8);
        }
        j += 1;
    }
    return arr;
}

fn create_seq(arr: Vec<u8>) -> Sequence {
    let mut high_nibble = false;
    if arr.len() % 2 == 1 {
        high_nibble = true;
    }
    let mut new_data = Vec::with_capacity(if high_nibble {
        (arr.len() / 2) + 1
    } else {
        arr.len()
    });
    let mut j = 0;
    let mut low = true;
    for i in 0..arr.len() {
        if (low == true) {
            new_data.push(arr[i]);
            low = false;
        } else {
            new_data[j] = new_data[j] | (arr[i] << 4);
            low = true;
            j += 1;
        }
    }
    return Sequence {
        data: new_data,
        high_nibble_start: false,
        high_nibble_end: high_nibble,
    };
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
    let seq = create_seq(vec![A, C, G, T, A]);
    let new_arr = get_seq(&seq, span);
    print_arr(new_arr);
}
