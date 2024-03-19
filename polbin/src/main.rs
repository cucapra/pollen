mod file;
mod flatgfa;
mod gfaline;
mod parse;
mod pool;
mod print;
use argh::FromArgs;
use flatgfa::GFABuilder;
use memmap::{Mmap, MmapMut};
use parse::Parser;

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

fn print_stats(gfa: &flatgfa::FlatGFA) {
    eprintln!("header: {}", gfa.header.len());
    eprintln!("segs: {}", gfa.segs.len());
    eprintln!("paths: {}", gfa.paths.len());
    eprintln!("links: {}", gfa.links.len());
    eprintln!("steps: {}", gfa.steps.len());
    eprintln!("seq_data: {}", gfa.seq_data.len());
    eprintln!("overlaps: {}", gfa.overlaps.len());
    eprintln!("alignment: {}", gfa.alignment.len());
    eprintln!("name_data: {}", gfa.name_data.len());
    eprintln!("optional_data: {}", gfa.optional_data.len());
    eprintln!("line_order: {}", gfa.line_order.len());
}

#[derive(FromArgs)]
/// Convert between GFA text and FlatGFA binary formats.
struct PolBin {
    /// read from a binary FlatGFA file
    #[argh(option, short = 'i')]
    input: Option<String>,

    /// read from a text GFA file
    #[argh(option, short = 'I')]
    input_gfa: Option<String>,

    /// write to a binary FlatGFA file
    #[argh(option, short = 'o')]
    output: Option<String>,

    /// mutate the input file in place
    #[argh(switch, short = 'm')]
    mutate: bool,

    /// print statistics about the graph
    #[argh(switch, short = 's')]
    stats: bool,

    /// preallocation size factor
    #[argh(option, short = 'p', default = "32")]
    prealloc_factor: usize,
}

fn main() {
    let args: PolBin = argh::from_env();

    // A special case for converting from GFA text to an in-place FlatGFA binary.
    if args.mutate {
        if let (None, Some(out_name)) = (&args.input, &args.output) {
            // Create a file with an empty table of contents.
            let empty_toc = file::Toc::guess(args.prealloc_factor);
            let mut mmap = map_new_file(out_name, empty_toc.size() as u64);
            let (toc, store) = file::init(&mut mmap, empty_toc);

            // Parse the input into the file.
            let store = match args.input_gfa {
                Some(name) => {
                    let file = map_file(&name);
                    let store = Parser::for_slice(store).parse_mem(file.as_ref());
                    *toc = file::Toc::for_slice_store(&store);
                    store
                }
                None => {
                    let stdin = std::io::stdin();
                    let store = Parser::for_slice(store).parse_stream(stdin.lock());
                    *toc = file::Toc::for_slice_store(&store);
                    store
                }
            };
            if args.stats {
                print_stats(&store.view());
            }
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
            // Parse from stdin or a file.
            store = match args.input_gfa {
                Some(name) => {
                    let file = map_file(&name);
                    Parser::for_heap().parse_mem(file.as_ref())
                }
                None => {
                    let stdin = std::io::stdin();
                    Parser::for_heap().parse_stream(stdin.lock())
                }
            };
            store.view()
        }
    };

    // Perhaps print some statistics.
    if args.stats {
        print_stats(&gfa);
    }

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
