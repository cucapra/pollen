mod flatgfa;
mod parse;

fn main() {
    let stdin = std::io::stdin();
    let flat = parse::parse(stdin.lock());
    dbg!(flat);
}
