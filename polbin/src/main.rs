mod cmds;
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

    /// preallocation size factor
    #[argh(option, short = 'p', default = "32")]
    prealloc_factor: usize,

    #[argh(subcommand)]
    command: Option<Command>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Toc(cmds::Toc),
    Paths(cmds::Paths),
    Stats(cmds::Stats),
    Extract(cmds::Extract),
}

fn main() {
    let args: PolBin = argh::from_env();

    // A special case for converting from GFA text to an in-place FlatGFA binary.
    if args.mutate {
        if let (None, None, Some(out_name)) = (&args.command, &args.input, &args.output) {
            prealloc_translate(args.input_gfa.as_deref(), out_name, args.prealloc_factor);
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

    match args.command {
        Some(Command::Toc(_)) => {
            cmds::toc(&gfa);
        }
        Some(Command::Paths(_)) => {
            cmds::paths(&gfa);
        }
        Some(Command::Stats(args)) => {
            cmds::stats(&gfa, args);
        }
        Some(Command::Extract(args)) => {
            cmds::extract(&gfa, args);
        }
        None => {
            // Just emit the GFA or FlatGFA file.
            dump(&gfa, &args.output);
        }
    }
}

/// Write a FlatGFA either to a GFA text file to stdout or a binary FlatGFA file given
/// with a name.
fn dump(gfa: &flatgfa::FlatGFA, output: &Option<String>) {
    match output {
        Some(name) => {
            let mut mmap = map_new_file(name, file::size(gfa) as u64);
            file::dump(gfa, &mut mmap);
            mmap.flush().unwrap();
        }
        None => print::print(gfa),
    }
}

/// A special-case fast-path transformation from a GFA text file to a *preallocated*
/// FlatGFA, with sizes based on estimates of the input counts.
fn prealloc_translate(in_name: Option<&str>, out_name: &str, prealloc_factor: usize) {
    let file;
    let (input_buf, empty_toc) = match in_name {
        // If we have an input GFA file, we can estimate its sizes for the TOC.
        Some(name) => {
            file = map_file(name);
            let toc = parse::estimate_toc(file.as_ref());
            (Some(file.as_ref()), toc)
        }

        // Otherwise, we need to guess.
        None => (None, file::Toc::guess(prealloc_factor)),
    };

    // Create a file with an empty table of contents.
    let mut mmap = map_new_file(out_name, empty_toc.size() as u64);
    let (toc, store) = file::init(&mut mmap, empty_toc);

    // Parse the input into the file.
    match input_buf {
        Some(buf) => {
            let store = Parser::for_slice(store).parse_mem(buf);
            *toc = file::Toc::for_slice_store(&store)
        }
        None => {
            let stdin = std::io::stdin();
            let store = Parser::for_slice(store).parse_stream(stdin.lock());
            *toc = file::Toc::for_slice_store(&store)
        }
    };

    mmap.flush().unwrap();
}
