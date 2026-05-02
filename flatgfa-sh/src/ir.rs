use std::collections::HashMap;

#[derive(Debug)]
pub enum Resource {
    File(String),
    Stdin,
    Stdout,
    Pipe,
}

/// An instruction performs one imperative action.
#[derive(Debug)]
pub enum Instr {
    NodeDepth(NodeDepthInstr),
    PathDepth(PathDepthInstr),
    Exec(ExecInstr),
}

#[derive(Debug)]
pub struct NodeDepthInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
}

impl From<NodeDepthInstr> for Instr {
    fn from(value: NodeDepthInstr) -> Self {
        Self::NodeDepth(value)
    }
}

#[derive(Debug)]
pub struct PathDepthInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
    pub path: Option<String>,
}

impl From<PathDepthInstr> for Instr {
    fn from(value: PathDepthInstr) -> Self {
        Self::PathDepth(value)
    }
}

/// An instruction that just runs an external shell command.
#[derive(Debug)]
pub struct ExecInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
    pub command: String,
    pub args: Vec<String>,
}

impl From<ExecInstr> for Instr {
    fn from(value: ExecInstr) -> Self {
        Self::Exec(value)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ResourceRef(pub usize);

#[derive(Debug)]
pub struct Program {
    pub rsrc: Vec<Resource>,
    pub instrs: Vec<Instr>,
}

pub struct Builder {
    rsrc: Vec<Resource>,
    instrs: Vec<Instr>,
    files: HashMap<String, ResourceRef>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            rsrc: vec![Resource::Stdin, Resource::Stdout],
            instrs: vec![],
            files: HashMap::new(),
        }
    }

    pub fn add_instr<I: Into<Instr>>(&mut self, instr: I) {
        self.instrs.push(instr.into());
    }

    pub fn add_rsrc(&mut self, rsrc: Resource) -> ResourceRef {
        self.rsrc.push(rsrc);
        ResourceRef(self.rsrc.len() - 1)
    }

    /// Add a file resource, or get an existing one if we've already added it.
    pub fn file(&mut self, name: String) -> ResourceRef {
        if let Some(&rsrc) = self.files.get(&name) {
            rsrc
        } else {
            let rsrc = self.add_rsrc(Resource::File(name.clone()));
            self.files.insert(name, rsrc);
            rsrc
        }
    }

    /// Create a new pipe resource.
    pub fn pipe(&mut self) -> ResourceRef {
        self.add_rsrc(Resource::Pipe)
    }

    /// Get the standard input resource.
    pub fn stdin(&self) -> ResourceRef {
        ResourceRef(0)
    }

    /// Get the standard output resource.
    pub fn stdout(&self) -> ResourceRef {
        ResourceRef(1)
    }

    pub fn build(self) -> Program {
        Program {
            rsrc: self.rsrc,
            instrs: self.instrs,
        }
    }
}
