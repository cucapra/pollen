#[derive(Debug)]
pub enum Resource {
    File(String),
    Memory,
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

#[derive(Debug)]
pub struct ResourceRef(pub usize);

#[derive(Debug)]
pub struct Program {
    pub rsrc: Vec<Resource>,
    pub ops: Vec<Op>,
}

pub struct Builder {
    rsrc: Vec<Resource>,
    ops: Vec<Op>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            rsrc: vec![Resource::Stdin, Resource::Stdout],
            ops: vec![],
        }
    }

    pub fn add_op(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn add_rsrc(&mut self, rsrc: Resource) -> ResourceRef {
        self.rsrc.push(rsrc);
        ResourceRef(self.rsrc.len() - 1)
    }

    pub fn add_file(&mut self, name: String) -> ResourceRef {
        self.add_rsrc(Resource::File(name))
    }

    pub fn add_mem(&mut self) -> ResourceRef {
        self.add_rsrc(Resource::Memory)
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
