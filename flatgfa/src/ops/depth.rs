use crate::flatgfa;
use crate::index;
use bit_vec::BitVec;

/// Compute the *depth* of each segment in the variation graph.
///
/// The depth is defined to be the number of times that a path traverses a given
/// segment. We return two values: the ordinary depth and the *unique* depth,
/// which only counts each path that tarverses a given segment once.
///
/// Both outputs are depth values indexed by segment ID.
pub fn depth(gfa: &flatgfa::FlatGFA, use_index: bool) -> (Vec<usize>, Vec<usize>) {
    // Our output vectors: the ordinary and unique depths of each segment.
    let mut depths = vec![0; gfa.segs.len()];
    let mut uniq_depths = vec![0; gfa.segs.len()];

    // This bit vector keeps track of whether the current path has already
    // traversed a given segment, and therefore whether we should ignore
    // subsequent traversals (for the purpose of counting unique depth).
    let mut seen = BitVec::from_elem(gfa.segs.len(), false);

    if use_index {
        // Build the index
        let step_seg_index = index::StepsBySegIndex::new(gfa);

        // This bit vector keeps track of whether the current *path* has already been seen
        // by the current working *segment*. A reverse approach to the non-indexed option
        let mut path_seen: BitVec = BitVec::from_elem(gfa.paths.len(), false);

        // iterate over each segment and populate the output vectors
        for (seg_id, _) in gfa.segs.items() {
            // clear the path_seen vector, since all paths should be unseen
            path_seen.clear();

            // get the actual offset of the id of thesegment
            let ind = seg_id.index();

            // get the span of StepRefs for this segment
            let span = step_seg_index.get_steps_slice(seg_id);

            // use that offset to directly modify the depths vector
            depths[ind] = span.len();

            // iterate over the span to populate the uniq_depths vector
            for stepref in span.iter() {
                // extract the path index
                let path_id = stepref.path.index();

                // if the path has not been seen, increment the unique depth
                if !path_seen[path_id] {
                    uniq_depths[ind] += 1;

                    // set the path to seen in the path_seen bitvec
                    path_seen.set(path_id, true);
                }
            }
        }
    } else {
        for path in gfa.paths.all().iter() {
            seen.clear(); // All segments are unseen.
            for step in &gfa.steps[path.steps] {
                let seg_id = step.segment().index();
                depths[seg_id] += 1;
                if !seen[seg_id] {
                    // The first traversal of this path over this segment.
                    uniq_depths[seg_id] += 1;
                    seen.set(seg_id, true);
                }
            }
        }
    }

    (depths, uniq_depths)
}
