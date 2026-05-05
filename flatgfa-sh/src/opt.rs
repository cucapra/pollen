use crate::ir::{Builder, Program};

pub fn optimize(prog: Program) -> Program {
    let builder = Builder::rebuild(prog);
    dbg!("optimize!");
    builder.build()
}
