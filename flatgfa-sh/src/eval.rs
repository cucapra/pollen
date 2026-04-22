use crate::ir;

struct Env {
    rsrc: Vec<ir::Resource>,
}

trait Eval {
    fn eval(&self, env: &Env);
}

impl Eval for ir::Op {
    fn eval(&self, env: &Env) {
        match self {
            Self::Depth(op) => op.eval(env),
        }
    }
}

impl Eval for ir::DepthOp {
    fn eval(&self, env: &Env) {
        let input = &env.rsrc[self.input.0];
        let output = &env.rsrc[self.output.0];
        println!(
            "here I would run depth with input {:?} and optional path name {:?}, sending output to {:?}",
            input, self.path, output
        );
    }
}

pub fn run(prog: ir::Program) {
    let env = Env { rsrc: prog.rsrc };
    for op in prog.ops {
        op.eval(&env);
    }
}
