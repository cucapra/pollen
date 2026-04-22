use crate::ir;
use flatgfa::{self, cli, flatgfa::HeapGFAStore, memfile};

struct Env {
    rsrc: Vec<ir::Resource>,
}

trait Eval {
    fn eval(&self, env: &Env);
}

impl Eval for ir::Instr {
    fn eval(&self, env: &Env) {
        match self {
            Self::Depth(instr) => instr.eval(env),
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

impl Eval for ir::DepthInstr {
    fn eval(&self, env: &Env) {
        let store = parse_gfa(&env.rsrc[self.input.0]);
        let gfa = store.as_ref();
        // TODO Do something about the output resource...
        match self.mode {
            ir::DepthOutputMode::NodeTable => cli::cmds::depth(&gfa),
            _ => unimplemented!("only node depth is supported"),
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
