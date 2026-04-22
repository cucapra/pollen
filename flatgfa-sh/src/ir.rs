use std::collections::HashMap;

#[derive(Debug)]
pub enum Resource {
    File(String),
    Stdin,
    Stdout,
}

#[derive(Debug)]
pub struct DepthInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
    pub path: Option<String>,
}

/// An instruction that just runs an external shell command.
#[derive(Debug)]
pub struct ShellInstr {
    pub input: ResourceRef,
    pub output: ResourceRef,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum Instr {
    Depth(DepthInstr),
    Shell(ShellInstr),
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

    pub fn add_instr(&mut self, op: Instr) {
        self.instrs.push(op);
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

    pub fn stdin(&self) -> ResourceRef {
        ResourceRef(0)
    }

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
