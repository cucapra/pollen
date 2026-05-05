use crate::ir::{self, Resource, ResourceRef};
use bstr::BStr;
use flatgfa::FlatGFA;
use flatgfa::flatbed::HeapBEDStore;
use flatgfa::{self, emit::Emit, flatgfa::HeapGFAStore, memfile, ops};
use memmap::Mmap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, PipeReader, PipeWriter};

struct Env {
    /// All the resource descriptions used in this program.
    rsrc: Vec<Resource>,

    /// Indices into the heap vectors for resources that have them.
    ///
    /// This has the same length as `rsrc`. For each resource type that comes
    /// with an associated run-time value, this contains the index into the
    /// appropriate heap vector in this environment. For others, it's MAX.
    idx: Vec<u16>,

    /// The currently open Unix pipes for operations in this program.
    ///
    /// This is a heap vector (indexed via `idx`) for each resource that is a
    /// `Pipe`. We lazily populate these slots when the first instruction uses
    /// the pipe. The first use of either "side" of the pipe consumes it.
    pipes: Vec<(Option<PipeReader>, Option<PipeWriter>)>,

    /// A heap vector of currently available FlatGFA data stores.
    gfa_stores: Vec<Option<HeapGFAStore>>,

    /// A heap vector of currently memory-mapped files.
    mmaps: Vec<Option<Mmap>>,

    /// A heap vector of FlatBED stores.
    bed_stores: Vec<Option<HeapBEDStore>>,
}

impl Env {
    fn new(rsrc: Vec<Resource>) -> Self {
        // Initialize the heap vectors and their indices.
        // TODO Reduce some duplication here...
        let mut idx: Vec<u16> = Vec::with_capacity(rsrc.len());
        let mut pipes: Vec<(Option<PipeReader>, Option<PipeWriter>)> =
            Vec::with_capacity(rsrc.len());
        let mut mmaps: Vec<Option<Mmap>> = Vec::with_capacity(rsrc.len());
        let mut gfa_stores: Vec<Option<HeapGFAStore>> = Vec::with_capacity(rsrc.len());
        let mut bed_stores: Vec<Option<HeapBEDStore>> = Vec::with_capacity(rsrc.len());
        for r in &rsrc {
            let i = match r {
                Resource::Pipe => {
                    pipes.push((None, None));
                    (pipes.len() - 1).try_into().unwrap()
                }
                Resource::GFAStore => {
                    gfa_stores.push(None);
                    (gfa_stores.len() - 1).try_into().unwrap()
                }
                Resource::Mmap => {
                    mmaps.push(None);
                    (mmaps.len() - 1).try_into().unwrap()
                }
                Resource::BEDStore => {
                    bed_stores.push(None);
                    (bed_stores.len() - 1).try_into().unwrap()
                }
                _ => u16::MAX,
            };
            idx.push(i);
        }

        Self {
            rsrc,
            idx,
            pipes,
            gfa_stores,
            mmaps,
            bed_stores,
        }
    }

    fn get_pipe(&mut self, rsrc: ResourceRef) -> &mut (Option<PipeReader>, Option<PipeWriter>) {
        debug_assert!(matches!(self.rsrc[rsrc.0], Resource::Pipe));
        &mut self.pipes[self.idx[rsrc.0] as usize]
    }

    fn get_gfa_store(&mut self, rsrc: ResourceRef) -> &mut Option<HeapGFAStore> {
        debug_assert!(matches!(self.rsrc[rsrc.0], Resource::GFAStore));
        &mut self.gfa_stores[self.idx[rsrc.0] as usize]
    }

    fn get_mmap(&mut self, rsrc: ResourceRef) -> &mut Option<Mmap> {
        debug_assert!(matches!(self.rsrc[rsrc.0], Resource::Mmap));
        &mut self.mmaps[self.idx[rsrc.0] as usize]
    }

    fn get_bed_store(&mut self, rsrc: ResourceRef) -> &mut Option<HeapBEDStore> {
        debug_assert!(matches!(self.rsrc[rsrc.0], Resource::BEDStore));
        &mut self.bed_stores[self.idx[rsrc.0] as usize]
    }

    fn read_pipe(&mut self, rsrc: ResourceRef) -> io::Result<PipeReader> {
        let pair = self.get_pipe(rsrc);
        if let Some(reader) = pair.0.take() {
            Ok(reader)
        } else {
            let (reader, writer) = io::pipe()?;
            pair.1 = Some(writer);
            Ok(reader)
        }
    }

    fn write_pipe(&mut self, rsrc: ResourceRef) -> io::Result<PipeWriter> {
        let pair = self.get_pipe(rsrc);
        if let Some(writer) = pair.1.take() {
            Ok(writer)
        } else {
            let (reader, writer) = io::pipe()?;
            pair.0 = Some(reader);
            Ok(writer)
        }
    }

    fn input(&mut self, rsrc: ResourceRef) -> Input {
        match &self.rsrc[rsrc.0] {
            Resource::Stdin => Input::Stdin(std::io::stdin().lock()),
            Resource::Stdout => panic!("cannot read from stdout"),
            Resource::File(name) => Input::File(memfile::map_file(name)),
            Resource::Pipe => Input::Pipe(BufReader::new(self.read_pipe(rsrc).unwrap())),
            _ => panic!("non-bytes input"),
        }
    }

    fn output(&mut self, rsrc: ResourceRef) -> Output {
        match &self.rsrc[rsrc.0] {
            Resource::Stdin => panic!("cannot write to stdin"),
            Resource::Stdout => Output::Stdout(std::io::stdout().lock()),
            Resource::File(name) => Output::File(BufWriter::new(File::create(name).unwrap())),
            Resource::Pipe => Output::Pipe(BufWriter::new(self.write_pipe(rsrc).unwrap())),
            _ => panic!("non-bytes output"),
        }
    }

    /// Get a FlatGFA data structure from the heap.
    ///
    /// The resource must be either a memory-mapped file or an in-memory
    /// FlatGFA store.
    fn flatgfa<'a>(&'a self, rsrc: ResourceRef) -> FlatGFA<'a> {
        match &self.rsrc[rsrc.0] {
            Resource::GFAStore => {
                let store = self.gfa_stores[self.idx[rsrc.0] as usize]
                    .as_ref()
                    .expect("store not populated");
                store.as_ref()
            }
            Resource::Mmap => {
                let mmap = self.mmaps[self.idx[rsrc.0] as usize]
                    .as_ref()
                    .expect("mmap not populated");
                flatgfa::file::view(mmap.as_ref())
            }
            _ => panic!("resource must be FlatGFA data"),
        }
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
        match self {
            Self::NodeDepth(instr) => instr.eval(env),
            Self::PathDepth(instr) => instr.eval(env),
            Self::Exec(instr) => instr.eval(env),
            Self::ParseGFA(instr) => instr.eval(env),
            Self::MapFile(instr) => instr.eval(env),
            Self::ParseBED(instr) => instr.eval(env),
            Self::MakeWindows(instr) => instr.eval(env),
        }
    }
}

impl Eval for ir::NodeDepthInstr {
    fn eval(&self, env: &mut Env) {
        let mut out = env.output(self.output);
        let gfa = env.flatgfa(self.input);
        let (depths, uniq_depths) = ops::depth::seg_depth(&gfa);
        out.emit(ops::depth::SegDepth {
            gfa: &gfa,
            depths,
            uniq_depths,
        })
        .unwrap();
    }
}

impl Eval for ir::PathDepthInstr {
    fn eval(&self, env: &mut Env) {
        let mut out = env.output(self.output);
        let gfa = env.flatgfa(self.input);
        if let Some(path_name) = &self.path {
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
}

impl Eval for ir::ExecInstr {
    fn eval(&self, env: &mut Env) {
        use std::fs::File;
        use std::process::Command;

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        match &env.rsrc[self.input.0] {
            Resource::Stdin => (),
            Resource::Stdout => panic!("cannot read from stdout"),
            Resource::File(name) => {
                cmd.stdin(File::open(name).unwrap());
            }
            Resource::Pipe => {
                cmd.stdin(env.read_pipe(self.input).unwrap());
            }
            _ => panic!("non-bytes input"),
        }

        match &env.rsrc[self.output.0] {
            Resource::Stdin => panic!("cannot write to stdin"),
            Resource::Stdout => (),
            Resource::File(name) => {
                cmd.stdout(File::create(name).unwrap());
            }
            Resource::Pipe => {
                cmd.stdout(env.write_pipe(self.output).unwrap());
            }
            _ => panic!("non-bytes output"),
        }

        // TODO: Do anything with the status?
        cmd.status().unwrap();
    }
}

impl Eval for ir::ParseGFAInstr {
    fn eval(&self, env: &mut Env) {
        use flatgfa::parse::Parser;

        let store = match env.input(self.input) {
            Input::File(file) => Parser::for_heap().parse_mem(file.as_ref()),
            Input::Stdin(stream) => Parser::for_heap().parse_stream(stream),
            Input::Pipe(stream) => Parser::for_heap().parse_stream(stream),
        };

        *env.get_gfa_store(self.output) = Some(store);
    }
}

impl Eval for ir::MapFileInstr {
    fn eval(&self, env: &mut Env) {
        if let Resource::File(name) = &env.rsrc[self.input.0] {
            let mmap = memfile::map_file(name);
            *env.get_mmap(self.output) = Some(mmap);
        } else {
            panic!("can only map actual files");
        }
    }
}

impl Eval for ir::ParseBEDInstr {
    fn eval(&self, env: &mut Env) {
        use flatgfa::flatbed::BEDParser;

        let store = match env.input(self.input) {
            Input::File(file) => BEDParser::for_heap().parse_mem(file.as_ref()),
            Input::Stdin(stream) => BEDParser::for_heap().parse_stream(stream),
            Input::Pipe(stream) => BEDParser::for_heap().parse_stream(stream),
        };

        *env.get_bed_store(self.output) = Some(store);
    }
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

impl Eval for ir::MakeWindowsInstr {
    fn eval(&self, env: &mut Env) {
        let store = env.get_bed_store(self.input).take().unwrap();
        let bed = store.as_ref();
        let mut out = env.output(self.output);
        for entry in bed.entries.all() {
            out.emit(WindowsBed {
                name: bed.get_name_of_entry(entry),
                start: entry.start,
                end: entry.end,
                size: self.size.try_into().unwrap(),
            })
            .unwrap();
        }
    }
}

pub fn run(prog: ir::Program) {
    let mut env = Env::new(prog.rsrc);
    for op in prog.instrs {
        op.eval(&mut env);
    }
}
