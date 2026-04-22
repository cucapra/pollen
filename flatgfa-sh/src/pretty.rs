use std::fmt::{self, Display, Formatter};

use crate::ir;

struct Context<'a> {
    rsrc: &'a [ir::Resource],
}

trait Print {
    fn print(&self, ctx: &Context, f: &mut Formatter<'_>) -> fmt::Result;
}

impl Print for ir::Op {
    fn print(&self, ctx: &Context, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Depth(op) => op.print(ctx, f),
        }
    }
}

impl Print for ir::DepthOp {
    fn print(&self, ctx: &Context, f: &mut Formatter<'_>) -> fmt::Result {
        let input = &ctx.rsrc[self.input.0];
        let output = &ctx.rsrc[self.output.0];
        writeln!(
            f,
            "depth({:?}, path={:?}) -> {:?}",
            input, self.path, output
        )
    }
}

impl Display for ir::Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ctx = Context { rsrc: &self.rsrc };
        for op in &self.ops {
            op.print(&ctx, f)?;
        }
        Ok(())
    }
}

impl Display for ir::Resource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ir::Resource::File(name) => write!(f, "\"{}\"", name),
            ir::Resource::Stdin => write!(f, "stdin"),
            ir::Resource::Stdout => write!(f, "stdout"),
            ir::Resource::Memory => write!(f, "mem"),
        }
    }
}
