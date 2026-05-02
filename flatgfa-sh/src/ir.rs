use std::collections::HashMap;

#[derive(Debug)]
pub enum Resource {
    File(String),
    Stdin,
    Stdout,
    Pipe,
    GFAStore,
    Mmap,
    BEDStore,
}

/// An instruction performs one imperative action.
#[derive(Debug)]
pub enum Instr {
    NodeDepth(NodeDepthInstr),
    PathDepth(PathDepthInstr),
    Exec(ExecInstr),
    ParseGFA(ParseGFAInstr),
    MapFile(MapFileInstr),
    ParseBED(ParseBEDInstr),
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

/// Parse a GFA file or stream in text foramt.
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

/// Memory-map a file: for instance, a FlatGFA binary file.
#[derive(Debug)]
pub struct MapFileInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
}

impl From<MapFileInstr> for Instr {
    fn from(value: MapFileInstr) -> Self {
        Self::MapFile(value)
    }
}

/// Parse a text BED file or stream.
#[derive(Debug)]
pub struct ParseBEDInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
}

impl From<ParseBEDInstr> for Instr {
    fn from(value: ParseBEDInstr) -> Self {
        Self::ParseBED(value)
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

    /// Create a new memory-mapped file resource.
    pub fn mmap(&mut self) -> ResourceRef {
        self.add_rsrc(Resource::Mmap)
    }

    /// Get the standard input resource.
    pub fn stdin(&self) -> ResourceRef {
        ResourceRef(0)
    }

    /// Get the standard output resource.
    pub fn stdout(&self) -> ResourceRef {
        ResourceRef(1)
    }

    /// Create an instruction to load a FlatGFA data structure as a resource.
    ///
    /// Either parse GFA text or memory-map a FlatGFA binary file. If `input` is
    /// a byte stream, we treat it as GFA text. If it's a file, we use the
    /// filename to decide whether to treat it as GFA text or FlatGFA binary.
    pub fn load_gfa(&mut self, input: ResourceRef) -> ResourceRef {
        match &self.rsrc[input.0] {
            Resource::File(name) if name.ends_with(".flatgfa") => {
                // Memory-map the FlatGFA binary file.
                let output = self.mmap();
                self.add_instr(Instr::MapFile(MapFileInstr { input, output }));
                output
            }
            Resource::Pipe | Resource::Stdin | Resource::File(_) => {
                // Parse as GFA text.
                let output = self.gfa_store();
                self.add_instr(Instr::ParseGFA(ParseGFAInstr { input, output }));
                output
            }
            _ => panic!("cannot parse this resource as GFA text"),
        }
    }

    pub fn build(self) -> Program {
        Program {
            rsrc: self.rsrc,
            instrs: self.instrs,
        }
    }
}
