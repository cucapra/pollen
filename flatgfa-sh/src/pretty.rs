use std::fmt::{self, Display, Formatter};

use crate::ir;

/// A utility for printing IR values using additional context.
struct Wrapped<'a, T> {
    rsrc: &'a [ir::Resource],
    val: &'a T,
}

impl<'a, T> Wrapped<'a, T> {
    fn wrap<S>(&self, val: &'a S) -> Wrapped<'a, S> {
        Wrapped {
            rsrc: self.rsrc,
            val,
        }
    }
}

impl<'a> Display for Wrapped<'a, ir::Instr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.val {
            ir::Instr::NodeDepth(instr) => self.wrap(instr).fmt(f),
            ir::Instr::PathDepth(instr) => self.wrap(instr).fmt(f),
            ir::Instr::Exec(instr) => self.wrap(instr).fmt(f),
        }
    }
}

impl<'a> Display for Wrapped<'a, ir::ResourceRef> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let rsrc = &self.rsrc[self.val.0];
        write!(f, "{}", rsrc)
    }
}

impl<'a> Display for Wrapped<'a, ir::NodeDepthInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "node_depth(")?;
        self.wrap(&self.val.input).fmt(f)?;
        write!(f, ") -> ")?;
        self.wrap(&self.val.output).fmt(f)?;
        writeln!(f)
    }
}

impl<'a> Display for Wrapped<'a, ir::PathDepthInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "path_depth(")?;
        self.wrap(&self.val.input).fmt(f)?;
        if let Some(path) = &self.val.path {
            write!(f, ", path=\"{}\"", path)?;
        }
        write!(f, ") -> ")?;
        self.wrap(&self.val.output).fmt(f)?;
        writeln!(f)
    }
}

impl<'a> Display for Wrapped<'a, ir::ExecInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "shell({:?}, {:?}, input=",
            self.val.command, self.val.args
        )?;
        self.wrap(&self.val.input).fmt(f)?;
        write!(f, ") -> ")?;
        self.wrap(&self.val.output).fmt(f)?;
        writeln!(f)
    }
}

impl Display for ir::Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for op in &self.instrs {
            Wrapped {
                rsrc: &self.rsrc,
                val: op,
            }
            .fmt(f)?;
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
