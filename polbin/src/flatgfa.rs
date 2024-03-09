use bstr::BString;

#[derive(Debug)]
pub struct SegInfo {
    pub name: usize,
    pub seq_offset: usize,
    pub seq_len: usize,
}

#[derive(Debug)]
pub struct PathInfo {
    pub name: BString,
    pub step_offset: usize,
    pub step_len: usize,
}

#[derive(Debug, PartialEq)]
pub struct Handle {
    pub segment: usize,
    pub forward: bool,
}

#[derive(Debug, Default)]
pub struct FlatGFA {
    pub seqdata: Vec<u8>,
    pub segs: Vec<SegInfo>,
    pub paths: Vec<PathInfo>,
    pub steps: Vec<Handle>,
}
