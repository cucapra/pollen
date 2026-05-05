use enum_map::{Enum, EnumMap};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Enum)]
pub enum ResourceKind {
    File,
    Stdin,
    Stdout,
    Pipe,
    GFAStore,
    Mmap,
    BEDStore,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Resource {
    pub kind: ResourceKind,
    pub index: u16,
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
    MakeWindows(MakeWindowsInstr),
}

#[derive(Debug)]
pub struct NodeDepthInstr {
    pub input: Resource,
    pub output: Resource,
}

impl From<NodeDepthInstr> for Instr {
    fn from(value: NodeDepthInstr) -> Self {
        Self::NodeDepth(value)
    }
}

#[derive(Debug)]
pub struct PathDepthInstr {
    pub input: Resource,
    pub output: Resource,
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
    pub input: Resource,
    pub output: Resource,
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
    pub input: Resource,
    pub output: Resource,
}

impl From<ParseGFAInstr> for Instr {
    fn from(value: ParseGFAInstr) -> Self {
        Self::ParseGFA(value)
    }
}

/// Memory-map a file: for instance, a FlatGFA binary file.
#[derive(Debug)]
pub struct MapFileInstr {
    pub input: Resource,
    pub output: Resource,
}

impl From<MapFileInstr> for Instr {
    fn from(value: MapFileInstr) -> Self {
        Self::MapFile(value)
    }
}

/// Parse a text BED file or stream.
#[derive(Debug)]
pub struct ParseBEDInstr {
    pub input: Resource,
    pub output: Resource,
}

impl From<ParseBEDInstr> for Instr {
    fn from(value: ParseBEDInstr) -> Self {
        Self::ParseBED(value)
    }
}

/// Create strided windows within chromosome spans.
///
/// Behaves like `bedtools makewindows`. Takes in a BED file with chromosome
/// sizes, and generates widows of the given `size` as a BED output.
#[derive(Debug)]
pub struct MakeWindowsInstr {
    pub input: Resource,
    pub output: Resource,
    pub size: usize,
}

impl From<MakeWindowsInstr> for Instr {
    fn from(value: MakeWindowsInstr) -> Self {
        Self::MakeWindows(value)
    }
}

#[derive(Debug)]
pub struct Program {
    pub instrs: Vec<Instr>,
    pub file_names: Vec<String>,
    pub rsrc_counts: EnumMap<ResourceKind, u16>,
}

pub struct Builder {
    instrs: Vec<Instr>,
    files: HashMap<String, Resource>,
    file_names: Vec<String>,
    rsrc_counts: EnumMap<ResourceKind, u16>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            instrs: Vec::new(),
            files: HashMap::new(),
            file_names: Vec::new(),
            rsrc_counts: EnumMap::default(),
        }
    }

    pub fn add_instr<I: Into<Instr>>(&mut self, instr: I) {
        self.instrs.push(instr.into());
    }

    /// Add a file resource, or get an existing one if we've already added it.
    pub fn file(&mut self, name: String) -> Resource {
        if let Some(&rsrc) = self.files.get(&name) {
            rsrc
        } else {
            let rsrc = Resource {
                kind: ResourceKind::File,
                index: self.files.len().try_into().unwrap(),
            };
            self.files.insert(name.clone(), rsrc);
            self.file_names.push(name);
            rsrc
        }
    }

    pub fn file_name(&self, rsrc: Resource) -> &str {
        debug_assert!(rsrc.kind == ResourceKind::File);
        &self.file_names[rsrc.index as usize]
    }

    /// Create a new "normal" resource (not a file, stdin, or stdout).
    pub fn add_rsrc(&mut self, kind: ResourceKind) -> Resource {
        let index = self.rsrc_counts[kind];
        self.rsrc_counts[kind] += 1;
        Resource { kind, index }
    }

    /// Get the standard input resource.
    pub fn stdin(&self) -> Resource {
        Resource {
            kind: ResourceKind::Stdin,
            index: 0,
        }
    }

    /// Get the standard output resource.
    pub fn stdout(&self) -> Resource {
        Resource {
            kind: ResourceKind::Stdout,
            index: 0,
        }
    }

    /// Create an instruction to load a FlatGFA data structure as a resource.
    ///
    /// Either parse GFA text, memory-map a FlatGFA binary file, or convert an
    /// odgi native file. If `input` is a byte stream, we treat it as GFA text.
    /// If it's a file, we use the filename to decide whether to treat it as GFA
    /// text or FlatGFA binary.
    pub fn load_gfa(&mut self, input: Resource) -> Resource {
        match input.kind {
            ResourceKind::File if self.file_name(input).ends_with(".flatgfa") => {
                // Memory-map the FlatGFA binary file.
                let output = self.add_rsrc(ResourceKind::Mmap);
                self.add_instr(Instr::MapFile(MapFileInstr { input, output }));
                output
            }
            ResourceKind::File if self.file_name(input).ends_with(".og") => {
                // Use `odgi view -g -i <file>` to dump as GFA text and then parse that.
                let pipe = self.add_rsrc(ResourceKind::Pipe);
                self.add_instr(ExecInstr {
                    input: self.stdin(),
                    output: pipe,
                    command: "odgi".into(),
                    args: vec![
                        "view".into(),
                        "-g".into(),
                        "-i".into(),
                        self.file_name(input).into(),
                    ],
                });
                self.load_gfa(pipe)
            }
            ResourceKind::Pipe | ResourceKind::Stdin | ResourceKind::File => {
                // Parse as GFA text.
                let output = self.add_rsrc(ResourceKind::GFAStore);
                self.add_instr(ParseGFAInstr { input, output });
                output
            }
            _ => panic!("cannot parse this resource as GFA text"),
        }
    }

    /// Create an instruction to parse a BED file to a FlatBED resource.
    pub fn load_bed(&mut self, input: Resource) -> Resource {
        match input.kind {
            ResourceKind::Pipe | ResourceKind::Stdin | ResourceKind::File => {
                let output = self.add_rsrc(ResourceKind::BEDStore);
                self.add_instr(ParseBEDInstr { input, output });
                output
            }
            _ => panic!("cannot parse this resource as BED text"),
        }
    }

    pub fn build(self) -> Program {
        Program {
            instrs: self.instrs,
            file_names: self.file_names,
            rsrc_counts: self.rsrc_counts,
        }
    }
}
