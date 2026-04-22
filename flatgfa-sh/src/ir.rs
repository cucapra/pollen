#[derive(Debug)]
pub enum GraphResource {
    File(String),
}

#[derive(Debug)]
pub struct DepthOp {
    pub input: GraphResource,
    pub path: Option<String>,
}

#[derive(Debug)]
pub enum Op {
    Depth(DepthOp),
}

#[derive(Debug)]
pub struct Program {
    pub ops: Vec<Op>,
}

pub struct Builder {
    ops: Vec<Op>,
}

impl Builder {
    pub fn new() -> Self {
        Self { ops: vec![] }
    }

    pub fn add_op(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn build(self) -> Program {
        Program { ops: self.ops }
    }
}
