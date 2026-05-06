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
pub struct Instr {
    pub input: Resource,
    pub output: Resource,
    pub op: Op,
}

/// The operation that an instruction may perform.
#[derive(Debug)]
pub enum Op {
    NodeDepth,
    PathDepth { path: Option<String> },
    Exec { command: String, args: Vec<String> },
    ParseGFA,
    MapFile,
    ParseBED,
    MakeWindows { size: usize },
    OdgiView,
}

#[derive(Debug)]
pub struct Program {
    pub instrs: Vec<Instr>,
    pub file_names: Vec<String>,
    pub rsrc_counts: EnumMap<ResourceKind, u16>,
}

pub struct Builder {
    pub instrs: Vec<Instr>,
    pub files: HashMap<String, Resource>,
    pub file_names: Vec<String>,
    pub rsrc_counts: EnumMap<ResourceKind, u16>,
}

impl Builder {
    /// Start building a fresh, empty program.
    pub fn new() -> Self {
        Self {
            instrs: Vec::new(),
            files: HashMap::new(),
            file_names: Vec::new(),
            rsrc_counts: EnumMap::default(),
        }
    }

    /// Start with an existing program.
    pub fn rebuild(prog: Program) -> Self {
        let files: HashMap<_, _> = prog
            .file_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                (
                    name.clone(),
                    Resource {
                        kind: ResourceKind::File,
                        index: i.try_into().unwrap(),
                    },
                )
            })
            .collect();
        Self {
            instrs: prog.instrs,
            files,
            file_names: prog.file_names,
            rsrc_counts: prog.rsrc_counts,
        }
    }

    /// Add an instruction to the end of the program.
    pub fn instr(&mut self, input: Resource, output: Resource, op: Op) {
        self.instrs.push(Instr { input, output, op })
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
    pub fn rsrc(&mut self, kind: ResourceKind) -> Resource {
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
                let output = self.rsrc(ResourceKind::Mmap);
                self.instr(input, output, Op::MapFile);
                output
            }
            ResourceKind::File if self.file_name(input).ends_with(".og") => {
                // Use `odgi view` to dump as GFA text and then parse that.
                let pipe = self.rsrc(ResourceKind::Pipe);
                self.instr(input, pipe, Op::OdgiView);
                self.load_gfa(pipe)
            }
            ResourceKind::Pipe | ResourceKind::Stdin | ResourceKind::File => {
                // Parse as GFA text.
                let output = self.rsrc(ResourceKind::GFAStore);
                self.instr(input, output, Op::ParseGFA);
                output
            }
            _ => panic!("cannot parse this resource as GFA text"),
        }
    }

    /// Create an instruction to parse a BED file to a FlatBED resource.
    pub fn load_bed(&mut self, input: Resource) -> Resource {
        match input.kind {
            ResourceKind::Pipe | ResourceKind::Stdin | ResourceKind::File => {
                let output = self.rsrc(ResourceKind::BEDStore);
                self.instr(input, output, Op::ParseBED);
                output
            }
            _ => panic!("cannot parse this resource as BED text"),
        }
    }

    /// Replace all uses of one resource with another.
    pub fn replace_rsrc(&mut self, old: Resource, new: Resource) {
        for instr in self.instrs.iter_mut() {
            if instr.input == old {
                instr.input = new;
            }
            if instr.output == old {
                instr.output = new;
            }
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
