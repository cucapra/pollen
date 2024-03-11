mod file;
mod flatgfa;
mod parse;
mod print;
use argh::FromArgs;
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

#[derive(FromArgs)]
/// Convert between GFA text and FlatGFA binary formats.
struct PolBin {
    /// read from a binary FlatGFA file
    #[argh(option, short = 'i')]
    input: Option<String>,

    /// write to a binary FlatGFA file
    #[argh(option, short = 'o')]
    output: Option<String>,
}

fn main() {
    let args: PolBin = argh::from_env();

    // Load the input from a file (binary) or stdin (text).
    let mmap;
    let store;
    let gfa = match args.input {
        Some(name) => {
            mmap = map_file(&name);
            file::view(&mmap)
        }
        None => {
            let stdin = std::io::stdin();
            store = parse::Parser::parse(stdin.lock());
            store.view()
        }
    };

    // Write the output to a file (binary) or stdout (text).
    match args.output {
        Some(name) => {
            let mut mmap = map_file_mut(&name);
            file::dump(&gfa, &mut mmap);
        }
        None => print::print(&gfa),
    }
}
