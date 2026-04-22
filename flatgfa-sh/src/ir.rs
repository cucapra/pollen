use std::collections::HashMap;

#[derive(Debug)]
pub enum Resource {
    File(String),
    Stdin,
    Stdout,
}

#[derive(Debug)]
pub struct DepthOp {
    pub input: ResourceRef,
    pub output: ResourceRef,
    pub path: Option<String>,
}

#[derive(Debug)]
pub enum Op {
    Depth(DepthOp),
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ResourceRef(pub usize);

#[derive(Debug)]
pub struct Program {
    pub rsrc: Vec<Resource>,
    pub ops: Vec<Op>,
}

pub struct Builder {
    rsrc: Vec<Resource>,
    ops: Vec<Op>,
    files: HashMap<String, ResourceRef>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            rsrc: vec![Resource::Stdin, Resource::Stdout],
            ops: vec![],
            files: HashMap::new(),
        }
    }

    pub fn add_op(&mut self, op: Op) {
        self.ops.push(op);
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
            ops: self.ops,
        }
    }
}
