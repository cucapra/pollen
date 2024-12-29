use crate::flatgfa;
use std::collections::HashSet;

pub fn depth(gfa: &flatgfa::FlatGFA) -> (Vec<usize>, Vec<usize>) {
    let mut depths = vec![0; gfa.segs.len()];
    // Initialize uniq_paths
    let mut uniq_paths = Vec::<HashSet<usize>>::new();
    uniq_paths.resize(gfa.segs.len(), HashSet::new());
    // do not assume that each handle in `gfa.steps()` is unique
    for (idx, path) in gfa.paths.all().iter().enumerate() {
        for step in &gfa.steps[path.steps] {
            let seg_id = step.segment().index();
            // Increment depths
            depths[seg_id] += 1;
            // Update uniq_paths
            uniq_paths[seg_id].insert(idx);
        }
    }

    let uniq_depths = uniq_paths.iter().map(|set| set.len()).collect();
    (depths, uniq_depths)
}
