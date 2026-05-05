use std::fmt::{self, Display, Formatter};

use crate::ir;

/// A utility for printing IR values using additional context.
struct Wrapped<'a, T> {
    file_names: &'a [String],
    val: &'a T,
}

impl<'a, T> Wrapped<'a, T> {
    /// Wrap a new value with the same printing context.
    fn wrap<S>(&self, val: &'a S) -> Wrapped<'a, S> {
        Wrapped {
            file_names: self.file_names,
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
            ir::Instr::MapFile(instr) => self.wrap(instr).fmt(f),
            ir::Instr::ParseBED(instr) => self.wrap(instr).fmt(f),
            ir::Instr::MakeWindows(instr) => self.wrap(instr).fmt(f),
            ir::Instr::OdgiView(instr) => self.wrap(instr).fmt(f),
        }
    }
}

impl Display for Wrapped<'_, ir::Resource> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let index = self.val.index;
        match self.val.kind {
            ir::ResourceKind::File => write!(f, "\"{}\"", self.file_names[index as usize]),
            ir::ResourceKind::Stdin => write!(f, "stdin"),
            ir::ResourceKind::Stdout => write!(f, "stdout"),
            ir::ResourceKind::Pipe => write!(f, "pipe-{}", index),
            ir::ResourceKind::GFAStore => write!(f, "gfa-store-{}", index),
            ir::ResourceKind::Mmap => write!(f, "mmap-{}", index),
            ir::ResourceKind::BEDStore => write!(f, "bed-store-{}", index),
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

impl Display for Wrapped<'_, ir::MapFileInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "map-file({}) -> {}",
            self.wrap(&self.val.input),
            self.wrap(&self.val.output),
        )
    }
}

impl Display for Wrapped<'_, ir::ParseBEDInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse-bed({}) -> {}",
            self.wrap(&self.val.input),
            self.wrap(&self.val.output),
        )
    }
}

impl Display for Wrapped<'_, ir::MakeWindowsInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "make-windows({}, {}) -> {}",
            self.wrap(&self.val.input),
            self.val.size,
            self.wrap(&self.val.output),
        )
    }
}

impl Display for Wrapped<'_, ir::OdgiViewInstr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "odgi-view({}) -> {}",
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
                    file_names: &self.file_names,
                    val: op,
                }
            )?;
        }
        Ok(())
    }
}
