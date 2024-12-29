use crate::flatgfa;
use bit_vec::BitVec;

pub fn depth(gfa: &flatgfa::FlatGFA) -> (Vec<usize>, Vec<usize>) {
    let mut depths = vec![0; gfa.segs.len()]; // The number of paths that traverse each segment.
    let mut uniq_depths = vec![0; gfa.segs.len()]; // The number of distinct paths that traverse them.

    let mut seen = BitVec::from_elem(gfa.segs.len(), false); // Has the current path traversed each segment?
    for path in gfa.paths.all().iter() {
        seen.clear();
        for step in &gfa.steps[path.steps] {
            let seg_id = step.segment().index();
            depths[seg_id] += 1;
            if seen[seg_id] {
                uniq_depths[seg_id] += 1;
                seen.set(seg_id, true);
            }
        }
    }

    (depths, uniq_depths)
}
