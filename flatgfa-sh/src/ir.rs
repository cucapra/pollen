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
