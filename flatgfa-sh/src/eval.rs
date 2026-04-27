use crate::ir;
use flatgfa::{self, flatgfa::HeapGFAStore, memfile, ops};

struct Env {
    rsrc: Vec<ir::Resource>,
}

trait Eval {
    fn eval(&self, env: &Env);
}

impl Eval for ir::Instr {
    fn eval(&self, env: &Env) {
        match self {
            Self::NodeDepth(instr) => instr.eval(env),
            Self::PathDepth(instr) => instr.eval(env),
            Self::Exec(instr) => instr.eval(env),
        }
    }
}

/// Parse a (text) GFA file from a resource.
fn parse_gfa(rsrc: &ir::Resource) -> HeapGFAStore {
    use flatgfa::parse::Parser;
    match rsrc {
        ir::Resource::File(name) => {
            let file = memfile::map_file(name);
            Parser::for_heap().parse_mem(file.as_ref())
        }
        ir::Resource::Stdin => {
            let stdin = std::io::stdin();
            Parser::for_heap().parse_stream(stdin.lock())
        }
        _ => unimplemented!(),
    }
}

impl Eval for ir::NodeDepthInstr {
    fn eval(&self, env: &Env) {
        let store = parse_gfa(&env.rsrc[self.input.0]);
        let gfa = store.as_ref();
        // TODO Do something about the output resource...
        let (depths, uniq_depths) = ops::depth::seg_depth(&gfa);
        ops::depth::print_seg_depth(&gfa, depths, uniq_depths);
    }
}

impl Eval for ir::PathDepthInstr {
    fn eval(&self, env: &Env) {
        let store = parse_gfa(&env.rsrc[self.input.0]);
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
    fn eval(&self, env: &Env) {
        use std::fs::File;
        use std::process::Command;

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);
        if let ir::Resource::File(name) = &env.rsrc[self.input.0] {
            cmd.stdin(File::open(name).unwrap());
        }
        if let ir::Resource::File(name) = &env.rsrc[self.output.0] {
            cmd.stdout(File::create(name).unwrap());
        }

        // TODO: Do anything with the status?
        cmd.status().unwrap();
    }
}

pub fn run(prog: ir::Program) {
    let env = Env { rsrc: prog.rsrc };
    for op in prog.instrs {
        op.eval(&env);
    }
}
