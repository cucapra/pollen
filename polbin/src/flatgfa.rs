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

    pub fn add_seg(&mut self, name: usize, seq: Vec<u8>) {
        self.segs.push(SegInfo {
            name,
            seq: Range {
                start: self.seqdata.len(),
                end: self.seqdata.len() + seq.len(),
            },
        });
        self.seqdata.extend(seq);
    }

    pub fn add_path(&mut self, name: Vec<u8>, steps: Vec<Handle>) {
        self.paths.push(PathInfo {
            name: BString::new(name),
            steps: Range {
                start: self.steps.len(),
                end: self.steps.len() + steps.len(),
            },
        });
        self.steps.extend(steps);

        // TODO Include the overlaps.
    }
}
