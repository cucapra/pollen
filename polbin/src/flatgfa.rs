use bstr::{BStr, BString};
use std::ops::Range;

#[derive(Debug)]
pub struct SegInfo {
    pub name: usize,
    pub seq: Range<usize>,
}

#[derive(Debug)]
pub struct PathInfo {
    pub name: BString,
    pub steps: Range<usize>,
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

impl FlatGFA {
    pub fn get_seq(&self, seg: &SegInfo) -> &BStr {
        self.seqdata[seg.seq.clone()].as_ref()
    }
}
