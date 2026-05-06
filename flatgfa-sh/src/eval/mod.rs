mod instr;

use crate::ir::{self, Resource, ResourceKind};
use enum_map::EnumMap;
use flatgfa::FlatGFA;
use flatgfa::flatbed::HeapBEDStore;
use flatgfa::{self, emit::Emit, flatgfa::HeapGFAStore, memfile};
use memmap::Mmap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, PipeReader, PipeWriter};
use std::ops::{Index, IndexMut};

struct Env {
    /// The names of each `File` resource, by their indices.
    file_names: Vec<String>,

    /// The currently open Unix pipes for operations in this program.
    ///
    /// We lazily populate these slots when the first instruction uses the pipe.
    /// The first use of either "side" of the pipe consumes it.
    pipes: Heap<(Option<PipeReader>, Option<PipeWriter>)>,

    /// The currently available FlatGFA data stores.
    gfa_stores: Heap<Option<HeapGFAStore>>,

    /// The currently memory-mapped files.
    mmaps: Heap<Option<Mmap>>,

    /// The available FlatBED stores.
    bed_stores: Heap<Option<HeapBEDStore>>,
}

impl Env {
    fn new(file_names: Vec<String>, counts: EnumMap<ResourceKind, u16>) -> Self {
        Self {
            file_names,
            pipes: Heap::new(counts[ResourceKind::Pipe], ResourceKind::Pipe),
            gfa_stores: Heap::new(counts[ResourceKind::GFAStore], ResourceKind::GFAStore),
            mmaps: Heap::new(counts[ResourceKind::Mmap], ResourceKind::Mmap),
            bed_stores: Heap::new(counts[ResourceKind::BEDStore], ResourceKind::BEDStore),
        }
    }

    fn file_name(&self, rsrc: Resource) -> &str {
        debug_assert!(rsrc.kind == ResourceKind::File);
        &self.file_names[rsrc.index as usize]
    }

    fn read_pipe(&mut self, rsrc: Resource) -> io::Result<PipeReader> {
        let pair = &mut self.pipes[rsrc];
        if let Some(reader) = pair.0.take() {
            Ok(reader)
        } else {
            let (reader, writer) = io::pipe()?;
            pair.1 = Some(writer);
            Ok(reader)
        }
    }

    fn write_pipe(&mut self, rsrc: Resource) -> io::Result<PipeWriter> {
        let pair = &mut self.pipes[rsrc];
        if let Some(writer) = pair.1.take() {
            Ok(writer)
        } else {
            let (reader, writer) = io::pipe()?;
            pair.0 = Some(reader);
            Ok(writer)
        }
    }

    fn input(&mut self, rsrc: Resource) -> Input {
        match rsrc.kind {
            ResourceKind::Stdin => Input::Stdin(std::io::stdin().lock()),
            ResourceKind::Stdout => panic!("cannot read from stdout"),
            ResourceKind::File => Input::File(memfile::map_file(self.file_name(rsrc))),
            ResourceKind::Pipe => Input::Pipe(BufReader::new(self.read_pipe(rsrc).unwrap())),
            _ => panic!("non-bytes input"),
        }
    }

    fn output(&mut self, rsrc: Resource) -> Output {
        match rsrc.kind {
            ResourceKind::Stdin => panic!("cannot write to stdin"),
            ResourceKind::Stdout => Output::Stdout(std::io::stdout().lock()),
            ResourceKind::File => {
                Output::File(BufWriter::new(File::create(self.file_name(rsrc)).unwrap()))
            }
            ResourceKind::Pipe => Output::Pipe(BufWriter::new(self.write_pipe(rsrc).unwrap())),
            _ => panic!("non-bytes output"),
        }
    }

    /// Get a FlatGFA data structure from the heap.
    ///
    /// The resource must be either a memory-mapped file or an in-memory
    /// FlatGFA store.
    fn flatgfa<'a>(&'a self, rsrc: Resource) -> FlatGFA<'a> {
        match rsrc.kind {
            ResourceKind::GFAStore => {
                let store = self.gfa_stores[rsrc].as_ref().expect("store not populated");
                store.as_ref()
            }
            ResourceKind::Mmap => {
                let mmap = self.mmaps[rsrc].as_ref().expect("mmap not populated");
                flatgfa::file::view(mmap.as_ref())
            }
            _ => panic!("resource must be FlatGFA data"),
        }
    }

    /// Execute a subprocess hooked up to the given resources as the stdin & stdout streams.
    fn run_cmd(
        &mut self,
        command: impl AsRef<OsStr>,
        args: &[impl AsRef<OsStr>],
        stdin: Resource,
        stdout: Resource,
    ) {
        use std::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);

        match stdin.kind {
            ResourceKind::Stdin => (),
            ResourceKind::Stdout => panic!("cannot read from stdout"),
            ResourceKind::File => {
                cmd.stdin(File::open(self.file_name(stdin)).unwrap());
            }
            ResourceKind::Pipe => {
                cmd.stdin(self.read_pipe(stdin).unwrap());
            }
            _ => panic!("non-bytes input"),
        }

        match stdout.kind {
            ResourceKind::Stdin => panic!("cannot write to stdin"),
            ResourceKind::Stdout => (),
            ResourceKind::File => {
                cmd.stdout(File::create(self.file_name(stdout)).unwrap());
            }
            ResourceKind::Pipe => {
                cmd.stdout(self.write_pipe(stdout).unwrap());
            }
            _ => panic!("non-bytes output"),
        }

        // TODO: Do anything with the status?
        cmd.status().unwrap();
    }
}

/// The data storage for heap values of a given resource kind.
///
/// This is a glorified `Vec<T>` that is indexed by `Resource` objects.
struct Heap<T: Default> {
    data: Vec<T>,
    kind: ResourceKind,
}

impl<T: Default> Heap<T> {
    fn new(size: u16, kind: ResourceKind) -> Self {
        let mut data = Vec::new();
        data.resize_with(size.into(), Default::default);
        Self { data, kind }
    }
}

impl<T: Default> Index<Resource> for Heap<T> {
    type Output = T;

    fn index(&self, rsrc: Resource) -> &Self::Output {
        debug_assert!(rsrc.kind == self.kind);
        &self.data[rsrc.index as usize]
    }
}

impl<T: Default> IndexMut<Resource> for Heap<T> {
    fn index_mut(&mut self, rsrc: Resource) -> &mut Self::Output {
        debug_assert!(rsrc.kind == self.kind);
        &mut self.data[rsrc.index as usize]
    }
}

enum Input {
    Stdin(std::io::StdinLock<'static>),
    File(Mmap),
    Pipe(BufReader<PipeReader>),
}

enum Output {
    Stdout(std::io::StdoutLock<'static>),
    File(BufWriter<File>),
    Pipe(BufWriter<PipeWriter>),
}

impl Output {
    fn emit<T: Emit>(&mut self, val: T) -> std::io::Result<()> {
        match *self {
            Self::Stdout(ref mut s) => val.emit(s),
            Self::File(ref mut s) => val.emit(s),
            Self::Pipe(ref mut s) => val.emit(s),
        }
    }
}

pub fn run(prog: ir::Program) {
    let mut env = Env::new(prog.file_names, prog.rsrc_counts);
    for i in prog.instrs {
        instr::eval(&mut env, &i);
    }
}
