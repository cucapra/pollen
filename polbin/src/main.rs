mod file;
mod flatgfa;
mod parse;
mod print;
use memmap::Mmap;

fn map_file(name: &str) -> Mmap {
    let file = std::fs::File::open(name).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

fn main() {
    // Read either GFA text from stdin or a binary file from the first argument.
    if let Some(name) = std::env::args().nth(1) {
        let mmap = map_file(&name);
        let gfa = file::view(&mmap);
        print::print(&gfa);
    } else {
        let stdin = std::io::stdin();
        let store = parse::Parser::parse(stdin.lock());
        let gfa = store.view();
        print::print(&gfa);
    }
}
