use crate::flatgfa;

pub fn depth(gfa: &flatgfa::FlatGFA) -> (Vec<usize>, Vec<usize>) {
    let mut depths = vec![0; gfa.segs.len()]; // The number of paths that traverse each segment.
    let mut uniq_depths = vec![0; gfa.segs.len()]; // The number of distinct paths that traverse them.

    for path in gfa.paths.all().iter() {
        let mut seen = vec![0; gfa.segs.len()]; // Has this path traversed this segment?
        for step in &gfa.steps[path.steps] {
            let seg_id = step.segment().index();
            depths[seg_id] += 1;
            if seen[seg_id] == 0 {
                uniq_depths[seg_id] += 1;
                seen[seg_id] = 1;
            }
        }
    }

    (depths, uniq_depths)
}
