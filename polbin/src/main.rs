mod file;
mod flatgfa;
mod parse;
mod print;
use memmap::{Mmap, MmapMut};

fn map_file(name: &str) -> Mmap {
    let file = std::fs::File::open(name).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

fn map_file_mut(name: &str) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(name)
        .unwrap();
    file.set_len(8092).unwrap(); // TODO Estimate the size?
    unsafe { MmapMut::map_mut(&file) }.unwrap()
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

        // TODO Just try dumping to a file.
        let mut mmap = map_file_mut("hello.flatgfa");
        file::dump(&gfa, &mut mmap);
    }
}
