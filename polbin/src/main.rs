mod flatgfa;
mod parse;

fn main() {
    let stdin = std::io::stdin();
    let flat = parse::parse(stdin.lock());

    for seg in &flat.segs {
        println!("S\t{}\t{}", seg.name, flat.get_seq(seg));
    }
}
