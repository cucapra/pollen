mod flatgfa;
mod parse;
mod print;

fn main() {
    let stdin = std::io::stdin();
    let gfa = parse::Parser::parse(stdin.lock());
    print::print(&gfa);
}
