use std::fmt::{self, Display, Formatter};

use crate::ir;

/// A utility for printing IR values using additional context.
struct Wrapped<'a, T> {
    rsrc: &'a [ir::Resource],
    val: &'a T,
}

impl<'a, T> Wrapped<'a, T> {
    /// Wrap a new value with the same printing context.
    fn wrap<S>(&self, val: &'a S) -> Wrapped<'a, S> {
        Wrapped {
            rsrc: self.rsrc,
            val,
        }
    }
}

impl Display for Wrapped<'_, ir::Instr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.val {
            ir::Instr::NodeDepth(instr) => self.wrap(instr).fmt(f),
            ir::Instr::PathDepth(instr) => self.wrap(instr).fmt(f),
            ir::Instr::Exec(instr) => self.wrap(instr).fmt(f),
            ir::Instr::ParseGFA(instr) => self.wrap(instr).fmt(f),
        }
    }
}

impl Display for Wrapped<'_, ir::ResourceRef> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let idx = self.val.0;
        let rsrc = &self.rsrc[idx];
        match rsrc {
            ir::Resource::File(name) => write!(f, "\"{}\"", name),
            ir::Resource::Stdin => write!(f, "stdin"),
            ir::Resource::Stdout => write!(f, "stdout"),
            ir::Resource::Pipe => write!(f, "pipe-{}", idx),
            ir::Resource::GFAStore => write!(f, "gfa-store-{}", idx),
        }
    }
}

impl Display for Wrapped<'_, ir::NodeDepthInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "node-depth({}) -> {}",
            self.wrap(&self.val.input),
            self.wrap(&self.val.output),
        )
    }
}

impl Display for Wrapped<'_, ir::PathDepthInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "path-depth({}", self.wrap(&self.val.input))?;
        if let Some(path) = &self.val.path {
            write!(f, ", path=\"{}\"", path)?;
        }
        write!(f, ") -> {}", self.wrap(&self.val.output))
    }
}

impl Display for Wrapped<'_, ir::ExecInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "shell({:?}, {:?}, input={}) -> {}",
            self.val.command,
            self.val.args,
            self.wrap(&self.val.input),
            self.wrap(&self.val.output),
        )
    }
}

impl Display for Wrapped<'_, ir::ParseGFAInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse-gfa({}) -> {}",
            self.wrap(&self.val.input),
            self.wrap(&self.val.output),
        )
    }
}

impl Display for ir::Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for op in &self.instrs {
            writeln!(
                f,
                "{}",
                Wrapped {
                    rsrc: &self.rsrc,
                    val: op,
                }
            )?;
        }
        Ok(())
    }
}
