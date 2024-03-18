mod file;
mod flatgfa;
mod gfaline;
mod parse;
mod pool;
mod print;
use argh::FromArgs;
use flatgfa::GFABuilder;
use memmap::{Mmap, MmapMut};

fn map_file(name: &str) -> Mmap {
    let file = std::fs::File::open(name).unwrap();
    unsafe { Mmap::map(&file) }.unwrap()
}

fn map_new_file(name: &str, size: u64) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(name)
        .unwrap();
    file.set_len(size).unwrap();
    unsafe { MmapMut::map_mut(&file) }.unwrap()
}

fn map_file_mut(name: &str) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name)
        .unwrap();
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

    /// mutate the input file in place
    #[argh(switch, short = 'm')]
    mutate: bool,
}

fn main() {
    let args: PolBin = argh::from_env();

    // A special case for converting from GFA text to an in-place FlatGFA binary.
    if args.mutate {
        if let (None, Some(out_name)) = (&args.input, &args.output) {
            let stdin = std::io::stdin();

            // Create a file with an empty table of contents.
            let empty_toc = file::Toc::guess(5);
            let mut mmap = map_new_file(out_name, empty_toc.size() as u64);
            let (toc, store) = file::init(&mut mmap, empty_toc);

            // Parse the input.
            let store = {
                let parser = parse::Parser::new(store);
                parser.parse(stdin.lock())
            };

            // Update the table of contents.
            *toc = file::Toc::for_slice_store(&store);

            mmap.flush().unwrap();
            return;
        }
    }

    // Load the input from a file (binary) or stdin (text).
    let mmap;
    let mut mmap_mut;
    let store;
    let slice_store;
    let gfa = match args.input {
        Some(name) => {
            if args.mutate {
                mmap_mut = map_file_mut(&name);
                slice_store = file::view_store(&mut mmap_mut);
                slice_store.view()
            } else {
                mmap = map_file(&name);
                file::view(&mmap)
            }
        }
        None => {
            let stdin = std::io::stdin();
            let parser = parse::heap_parser();
            store = parser.parse(stdin.lock());
            store.view()
        }
    };

    // Write the output to a file (binary) or stdout (text).
    match args.output {
        Some(name) => {
            let mut mmap = map_new_file(&name, file::size(&gfa) as u64);
            file::dump(&gfa, &mut mmap);
            mmap.flush().unwrap();
        }
        None => print::print(&gfa),
    }
}
