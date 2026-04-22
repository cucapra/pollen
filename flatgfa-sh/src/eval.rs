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
        let input = &env.rsrc[self.input.0];
        let output = &env.rsrc[self.output.0];
        println!(
            "here I would run a subprocess for command {:?} with args {:?}, redirecting stdin from {:?} and stdout to {:?}",
            self.command, self.args, input, output,
        );
    }
}

pub fn run(prog: ir::Program) {
    let env = Env { rsrc: prog.rsrc };
    for op in prog.instrs {
        op.eval(&env);
    }
}
