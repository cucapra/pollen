use crate::ir::{self, Op, Resource, ResourceKind};
use bstr::BStr;
use enum_map::EnumMap;
use flatgfa::FlatGFA;
use flatgfa::flatbed::HeapBEDStore;
use flatgfa::{self, emit::Emit, flatgfa::HeapGFAStore, memfile, ops};
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

trait Eval {
    fn eval(&self, env: &mut Env);
}

impl Eval for ir::Instr {
    fn eval(&self, env: &mut Env) {
        match &self.op {
            Op::NodeDepth => node_depth(env, self.input, self.output),
            Op::PathDepth { path } => path_depth(env, self.input, self.output, path),
            Op::Exec { command, args } => exec(env, self.input, self.output, command, args),
            Op::ParseGFA => parse_gfa(env, self.input, self.output),
            Op::MapFile => map_file(env, self.input, self.output),
            Op::ParseBED => parse_bed(env, self.input, self.output),
            Op::MakeWindows { size } => make_windows(env, self.input, self.output, *size),
            Op::OdgiView => odgi_view(env, self.input, self.output),
        }
    }
}

fn node_depth(env: &mut Env, input: Resource, output: Resource) {
    let mut out = env.output(output);
    let gfa = env.flatgfa(input);
    let (depths, uniq_depths) = ops::depth::seg_depth(&gfa);
    out.emit(ops::depth::SegDepth {
        gfa: &gfa,
        depths,
        uniq_depths,
    })
    .unwrap();
}

fn path_depth(env: &mut Env, input: Resource, output: Resource, path: &Option<String>) {
    let mut out = env.output(output);
    let gfa = env.flatgfa(input);
    if let Some(path_name) = &path {
        // TODO More elegantly handle missing paths.
        let path_id = gfa
            .find_path(path_name.as_bytes().into())
            .expect("no such path found");
        let (lengths, depths) = ops::depth::path_depth(&gfa, std::iter::once(path_id));
        out.emit(ops::depth::PathDepth {
            gfa: &gfa,
            depths,
            lengths,
            paths: std::iter::once(path_id),
        })
        .unwrap();
    } else {
        let (lengths, depths) = ops::depth::path_depth(&gfa, gfa.paths.ids());
        out.emit(ops::depth::PathDepth {
            gfa: &gfa,
            depths,
            lengths,
            paths: gfa.paths.ids(),
        })
        .unwrap();
    }
}

fn exec(env: &mut Env, input: Resource, output: Resource, command: &String, args: &[String]) {
    env.run_cmd(&command, &args, input, output);
}

fn parse_gfa(env: &mut Env, input: Resource, output: Resource) {
    use flatgfa::parse::Parser;

    let store = match env.input(input) {
        Input::File(file) => Parser::for_heap().parse_mem(file.as_ref()),
        Input::Stdin(stream) => Parser::for_heap().parse_stream(stream),
        Input::Pipe(stream) => Parser::for_heap().parse_stream(stream),
    };

    env.gfa_stores[output] = Some(store);
}

fn map_file(env: &mut Env, input: Resource, output: Resource) {
    if let ResourceKind::File = input.kind {
        let mmap = memfile::map_file(env.file_name(input));
        env.mmaps[output] = Some(mmap);
    } else {
        panic!("can only map actual files");
    }
}

fn parse_bed(env: &mut Env, input: Resource, output: Resource) {
    use flatgfa::flatbed::BEDParser;

    let store = match env.input(input) {
        Input::File(file) => BEDParser::for_heap().parse_mem(file.as_ref()),
        Input::Stdin(stream) => BEDParser::for_heap().parse_stream(stream),
        Input::Pipe(stream) => BEDParser::for_heap().parse_stream(stream),
    };

    env.bed_stores[output] = Some(store);
}

struct WindowsBed<'a> {
    name: &'a BStr,
    start: u64,
    end: u64,
    size: u64,
}

impl<'a> Emit for WindowsBed<'a> {
    fn emit(self, f: &mut impl std::io::Write) -> io::Result<()> {
        let mut pos = self.start;
        while pos < self.end {
            let end = (pos + self.size).min(self.end);
            writeln!(f, "{}\t{}\t{}", self.name, pos, end)?;
            pos = end;
        }
        Ok(())
    }
}

fn make_windows(env: &mut Env, input: Resource, output: Resource, size: usize) {
    let store = env.bed_stores[input].take().unwrap();
    let bed = store.as_ref();
    let mut out = env.output(output);
    for entry in bed.entries.all() {
        out.emit(WindowsBed {
            name: bed.get_name_of_entry(entry),
            start: entry.start,
            end: entry.end,
            size: size.try_into().unwrap(),
        })
        .unwrap();
    }
}

fn odgi_view(env: &mut Env, input: Resource, output: Resource) {
    let og_file = env.file_name(input).to_string();
    env.run_cmd(
        "odgi",
        &["view", "-g", "-i", &og_file],
        Resource {
            kind: ResourceKind::Stdin,
            index: 0,
        },
        output,
    )
}

pub fn run(prog: ir::Program) {
    let mut env = Env::new(prog.file_names, prog.rsrc_counts);
    for op in prog.instrs {
        op.eval(&mut env);
    }
}
