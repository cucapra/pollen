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
        // Exec instructions get formatted in a special way.
        if let ir::Op::Exec { command, args } = &self.val.op {
            write!(
                f,
                "shell({:?}, {:?}, input={}) -> {}",
                command,
                args,
                self.wrap(&self.val.input),
                self.wrap(&self.val.output)
            )?;
            return Ok(());
        }

        // Other instructions follow a `name(input, args) -> output` pattern.
        let name = match &self.val.op {
            ir::Op::NodeDepth => "node-depth",
            ir::Op::PathDepth { path: _ } => "path-depth",
            ir::Op::Exec {
                command: _,
                args: _,
            } => "exec",
            ir::Op::ParseGFA => "parse-gfa",
            ir::Op::MapFile => "map-file",
            ir::Op::ParseBED => "parse-bed",
            ir::Op::MakeWindows { size: _ } => "make-windows",
            ir::Op::OdgiView => "odgi-view",
        };
        write!(f, "{}({}", name, self.wrap(&self.val.input))?;
        match &self.val.op {
            ir::Op::PathDepth { path: Some(path) } => write!(f, ", path={:?}", path)?,
            ir::Op::Exec { command, args } => {
                write!(f, ", command={:?}, args={:?}", command, args)?
            }
            ir::Op::MakeWindows { size } => write!(f, ", size={}", size)?,
            _ => (),
        };
        write!(f, ") -> {}", self.wrap(&self.val.output))
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
