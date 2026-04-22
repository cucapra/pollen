use crate::ir;

pub trait Eval {
    fn eval(&self);
}

impl Eval for ir::Program {
    fn eval(&self) {
        for op in &self.ops {
            op.eval();
        }
    }
}

impl Eval for ir::Op {
    fn eval(&self) {
        match self {
            Self::Depth(op) => op.eval(),
        }
    }
}

impl Eval for ir::DepthOp {
    fn eval(&self) {
        println!(
            "here I would run depth with input {:?} and optional path name {:?}",
            self.input, self.path
        );
    }
}
