use std::fmt::{self, Display, Formatter};

use crate::ir;

struct Context<'a> {
    rsrc: &'a [ir::Resource],
}

trait Print {
    fn print(&self, ctx: &Context, f: &mut Formatter<'_>) -> fmt::Result;
}

impl Print for ir::Instr {
    fn print(&self, ctx: &Context, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Depth(op) => op.print(ctx, f),
        }
    }
}

impl Print for ir::ResourceRef {
    fn print(&self, ctx: &Context, f: &mut Formatter<'_>) -> fmt::Result {
        let rsrc = &ctx.rsrc[self.0];
        write!(f, "{}", rsrc)
    }
}

impl Print for ir::DepthInstr {
    fn print(&self, ctx: &Context, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "depth(")?;
        self.input.print(ctx, f)?;
        if let Some(path) = &self.path {
            write!(f, ", path=\"{}\"", path)?;
        }
        write!(f, ") -> ")?;
        self.output.print(ctx, f)?;
        writeln!(f)
    }
}

impl Display for ir::Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ctx = Context { rsrc: &self.rsrc };
        for op in &self.instrs {
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
        }
    }
}
