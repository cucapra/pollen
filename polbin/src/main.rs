mod file;
mod flatgfa;
mod parse;
mod print;

fn main() {
    let stdin = std::io::stdin();
    let store = parse::Parser::parse(stdin.lock());
    let gfa = store.view();
    print::print(&gfa);
}
