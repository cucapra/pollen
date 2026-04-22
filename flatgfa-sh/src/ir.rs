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

impl Program {
    pub fn run(&self) {
        for op in &self.ops {
            op.run();
        }
    }
}

impl Op {
    pub fn run(&self) {
        match self {
            Self::Depth(op) => op.run(),
        }
    }
}

impl DepthOp {
    pub fn run(&self) {
        println!(
            "here I would run depth with input {:?} and optional path name {:?}",
            self.input, self.path
        );
    }
}
