use crate::ir::{self, ResourceRef};
use flatgfa::{self, flatgfa::HeapGFAStore, memfile, ops};
use std::io::{self, BufReader, PipeReader, PipeWriter};

struct Env {
    /// All the resource descriptions used in this program.
    rsrc: Vec<ir::Resource>,

    /// The currently open Unix pipes for operations in this program.
    ///
    /// This has the same length as `rsrc`. For every resource index that is a
    /// `Pipe`, this may contain the currently open pipe. We lazily populate
    /// these slots when the first instruction uses the pipe. The first use of
    /// either "side" of the pipe consumes it.
    pipes: Vec<(Option<PipeReader>, Option<PipeWriter>)>,
}

impl Env {
    fn new(rsrc: Vec<ir::Resource>) -> Self {
        let mut pipes = Vec::with_capacity(rsrc.len());
        pipes.resize_with(rsrc.len(), Default::default);
        Self { rsrc, pipes }
    }

    fn read_pipe(&mut self, rsrc: ResourceRef) -> io::Result<PipeReader> {
        let idx = rsrc.0;
        debug_assert!(matches!(self.rsrc[idx], ir::Resource::Pipe));
        if let Some(reader) = self.pipes[idx].0.take() {
            Ok(reader)
        } else {
            let (reader, writer) = io::pipe()?;
            self.pipes[idx].1 = Some(writer);
            Ok(reader)
        }
    }

    fn write_pipe(&mut self, rsrc: ResourceRef) -> io::Result<PipeWriter> {
        let idx = rsrc.0;
        debug_assert!(matches!(self.rsrc[idx], ir::Resource::Pipe));
        if let Some(writer) = self.pipes[idx].1.take() {
            Ok(writer)
        } else {
            let (reader, writer) = io::pipe()?;
            self.pipes[idx].0 = Some(reader);
            Ok(writer)
        }
    }

    /// Parse a (text) GFA file from a resource.
    fn parse_gfa(&mut self, rsrc: ir::ResourceRef) -> HeapGFAStore {
        use flatgfa::parse::Parser;
        match &self.rsrc[rsrc.0] {
            ir::Resource::File(name) => {
                let file = memfile::map_file(name);
                Parser::for_heap().parse_mem(file.as_ref())
            }
            ir::Resource::Stdin => {
                let stdin = std::io::stdin();
                Parser::for_heap().parse_stream(stdin.lock())
            }
            ir::Resource::Stdout => panic!("cannot read from stdout"),
            ir::Resource::Pipe => {
                let read = self.read_pipe(rsrc).unwrap();
                Parser::for_heap().parse_stream(BufReader::new(read))
            }
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
        }
    }
}

impl Eval for ir::NodeDepthInstr {
    fn eval(&self, env: &mut Env) {
        let store = env.parse_gfa(self.input);
        let gfa = store.as_ref();
        // TODO Do something about the output resource...
        let (depths, uniq_depths) = ops::depth::seg_depth(&gfa);
        ops::depth::print_seg_depth(&gfa, depths, uniq_depths);
    }
}

impl Eval for ir::PathDepthInstr {
    fn eval(&self, env: &mut Env) {
        let store = env.parse_gfa(self.input);
        let gfa = store.as_ref();
        if let Some(path_name) = &self.path {
            // TODO More elegantly handle missing paths.
            let path_id = gfa
                .find_path(path_name.as_bytes().into())
                .expect("no such path found");
            let (depths, uniq_depths) = ops::depth::path_depth(&gfa, std::iter::once(path_id));
            ops::depth::print_path_depth(&gfa, depths, uniq_depths, std::iter::once(path_id));
        } else {
            let (depths, uniq_depths) = ops::depth::path_depth(&gfa, gfa.paths.ids());
            ops::depth::print_path_depth(&gfa, depths, uniq_depths, gfa.paths.ids());
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
            ir::Resource::Stdin => (),
            ir::Resource::Stdout => panic!("cannot read from stdout"),
            ir::Resource::File(name) => {
                cmd.stdin(File::open(name).unwrap());
            }
            ir::Resource::Pipe => {
                cmd.stdin(env.read_pipe(self.input).unwrap());
            }
        }

        match &env.rsrc[self.output.0] {
            ir::Resource::Stdin => panic!("cannot write to stdin"),
            ir::Resource::Stdout => (),
            ir::Resource::File(name) => {
                cmd.stdout(File::create(name).unwrap());
            }
            ir::Resource::Pipe => {
                cmd.stdout(env.write_pipe(self.output).unwrap());
            }
        }

        // TODO: Do anything with the status?
        cmd.status().unwrap();
    }
}

pub fn run(prog: ir::Program) {
    let mut env = Env::new(prog.rsrc);
    for op in prog.instrs {
        op.eval(&mut env);
    }
}
