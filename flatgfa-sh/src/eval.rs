use crate::ir;

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

impl Eval for ir::DepthInstr {
    fn eval(&self, env: &Env) {
        let input = &env.rsrc[self.input.0];
        let output = &env.rsrc[self.output.0];
        println!(
            "here I would run depth with input {:?} and optional path name {:?}, sending output to {:?}",
            input, self.path, output
        );
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
