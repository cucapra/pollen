use crate::ir::{Instr, Op, Program, Resource, ResourceKind};
use enum_map::EnumMap;
use std::collections::HashMap;

pub struct Builder {
    pub instrs: Vec<Instr>,
    pub files: HashMap<String, u16>,
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
            .map(|(i, name)| (name.clone(), i.try_into().unwrap()))
            .collect();
        Self {
            instrs: prog.instrs,
            files,
            file_names: prog.file_names,
            rsrc_counts: prog.rsrc_counts,
        }
    }

    /// Add an instruction to the end of the program.
    pub fn instr(&mut self, inputs: &[Resource], output: Resource, op: Op) {
        self.instrs.push(Instr {
            inputs: inputs.into(),
            output,
            op,
        })
    }

    /// Add a file resource, or get an existing one if we've already added it.
    pub fn file(&mut self, name: String) -> Resource {
        if let Some(&index) = self.files.get(&name) {
            Resource::new(ResourceKind::File, index)
        } else {
            let index: u16 = self.files.len().try_into().unwrap();
            self.files.insert(name.clone(), index);
            self.file_names.push(name);
            Resource::new(ResourceKind::File, index)
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
        Resource::new(kind, index)
    }

    /// Create an instruction to load a FlatGFA data structure as a resource.
    ///
    /// Either parse GFA text, memory-map a FlatGFA binary file, or convert an
    /// odgi native file. If `input` is a byte stream, we treat it as GFA text.
    /// If it's a file, we use the filename to decide whether to treat it as GFA
    /// text, compressed GFA text, FlatGFA binary, or an odgi graph.
    pub fn load_gfa(&mut self, input: Resource) -> Resource {
        match input.kind {
            ResourceKind::File if self.file_name(input).ends_with(".flatgfa") => {
                // Memory-map the FlatGFA binary file.
                let output = self.rsrc(ResourceKind::Mmap);
                self.instr(&[input], output, Op::MapFile);
                output
            }
            ResourceKind::File if self.file_name(input).ends_with(".og") => {
                // Use `odgi view` to dump as GFA text and then parse that.
                let pipe = self.rsrc(ResourceKind::Pipe);
                self.instr(&[input], pipe, Op::OdgiView);
                self.load_gfa(pipe)
            }
            ResourceKind::Pipe | ResourceKind::Stdin | ResourceKind::File => {
                // Parse as GFA text.
                let input = self.maybe_decompress(input);
                let output = self.rsrc(ResourceKind::GFAStore);
                self.instr(&[input], output, Op::ParseGFA);
                output
            }
            _ => panic!("cannot parse this resource as GFA text"),
        }
    }

    /// Create an instruction to parse a (possibly compressed) text BED file to
    /// a FlatBED resource.
    pub fn load_bed(&mut self, input: Resource) -> Resource {
        match input.kind {
            ResourceKind::Pipe | ResourceKind::Stdin | ResourceKind::File => {
                let input = self.maybe_decompress(input);
                let output = self.rsrc(ResourceKind::BEDStore);
                self.instr(&[input], output, Op::ParseBED);
                output
            }
            _ => panic!("cannot parse this resource as BED text"),
        }
    }

    /// If the input is a gzip-compressed file, create an instruction to
    /// decompress it. Otherwise, return it unchanged.
    ///
    /// This uses an OS pipe for the decompressed data, which is at least
    /// general, but it's probably not the most efficient.
    pub fn maybe_decompress(&mut self, input: Resource) -> Resource {
        match input.kind {
            ResourceKind::File if self.file_name(input).ends_with(".gz") => {
                let pipe = self.rsrc(ResourceKind::Pipe);
                self.instr(&[input], pipe, Op::GzipDecompress);
                pipe
            }
            _ => input,
        }
    }

    /// Replace all uses of one resource with another.
    pub fn replace_rsrc(&mut self, old: Resource, new: Resource) {
        for instr in self.instrs.iter_mut() {
            for input in instr.inputs.iter_mut() {
                if *input == old {
                    *input = new;
                }
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
