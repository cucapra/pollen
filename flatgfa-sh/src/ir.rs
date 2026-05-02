use std::collections::HashMap;

#[derive(Debug)]
pub enum Resource {
    File(String),
    Stdin,
    Stdout,
    Pipe,
    GFAStore,
}

/// An instruction performs one imperative action.
#[derive(Debug)]
pub enum Instr {
    NodeDepth(NodeDepthInstr),
    PathDepth(PathDepthInstr),
    Exec(ExecInstr),
    ParseGFA(ParseGFAInstr),
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

/// Parse a GFA file or stream  in text foramt.
#[derive(Debug)]
pub struct ParseGFAInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
}

impl From<ParseGFAInstr> for Instr {
    fn from(value: ParseGFAInstr) -> Self {
        Self::ParseGFA(value)
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

    /// Create a new FlatGFA data store resource.
    pub fn gfa_store(&mut self) -> ResourceRef {
        self.add_rsrc(Resource::GFAStore)
    }

    /// Get the standard input resource.
    pub fn stdin(&self) -> ResourceRef {
        ResourceRef(0)
    }

    /// Get the standard output resource.
    pub fn stdout(&self) -> ResourceRef {
        ResourceRef(1)
    }

    /// Create an instruction to parse GFA text into a FlatGFA store.
    ///
    /// The input should be a plain-bytes resource. We return the output, which
    /// is a FlatGFA store resource.
    pub fn parse_gfa(&mut self, input: ResourceRef) -> ResourceRef {
        let output = self.gfa_store();
        self.add_instr(Instr::ParseGFA(ParseGFAInstr { input, output }));
        output
    }

    pub fn build(self) -> Program {
        Program {
            rsrc: self.rsrc,
            instrs: self.instrs,
        }
    }
}
