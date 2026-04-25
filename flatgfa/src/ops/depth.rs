use crate::flatgfa;
use bit_vec::BitVec;

/// Compute the *depth* of each segment in the variation graph.
///
/// The depth is defined to be the number of times that a path traverses a given
/// segment. We return two values: the ordinary depth and the *unique* depth,
/// which only counts each path that tarverses a given segment once.
///
/// Both outputs are depth values indexed by segment ID.
pub fn seg_depth(gfa: &flatgfa::FlatGFA) -> (Vec<usize>, Vec<usize>) {
    // Our output vectors: the ordinary and unique depths of each segment.
    let mut depths = vec![0; gfa.segs.len()];
    let mut uniq_depths = vec![0; gfa.segs.len()];

    // This bit vector keeps track of whether the current path has already
    // traversed a given segment, and therefore whether we should ignore
    // subsequent traversals (for the purpose of counting unique depth).
    let mut seen = BitVec::from_elem(gfa.segs.len(), false);

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

    (depths, uniq_depths)
}

/// Print a segment depth table.
///
/// Format the result of `seg_depth` in an odgi-style TSV.
pub fn print_seg_depth(gfa: &flatgfa::FlatGFA, depths: Vec<usize>, uniq_depths: Vec<usize>) {
    println!("#node.id\tdepth\tdepth.uniq");
    for (id, seg) in gfa.segs.items() {
        let name: u32 = seg.name as u32;
        println!(
            "{}\t{}\t{}",
            name,
            depths[id.index()],
            uniq_depths[id.index()],
        );
    }
}

/// Compute the mean depth of each *path* in the variation graph.
///
/// A path's mean depth is defined to be the average of all the segment depths
/// that appear in the path.
pub fn path_depth(gfa: &flatgfa::FlatGFA) -> (Vec<usize>, Vec<f64>) {
    // Compute (non-unique) segment depth.
    let mut seg_depths = vec![0; gfa.segs.len()];
    for path in gfa.paths.all().iter() {
        for step in &gfa.steps[path.steps] {
            let seg_id = step.segment().index();
            seg_depths[seg_id] += 1;
        }
    }

    // Weighted average across each path.
    let mut path_lengths = Vec::with_capacity(gfa.paths.len());
    let mut path_depths = Vec::with_capacity(gfa.paths.len());
    for path in gfa.paths.all().iter() {
        let mut total_depth = 0;
        let mut total_length = 0;
        for step in &gfa.steps[path.steps] {
            let len = gfa.segs[step.segment()].len();
            total_depth += seg_depths[step.segment().index()] * len;
            total_length += len;
        }

        path_depths.push((total_depth as f64) / (total_length as f64));
        path_lengths.push(total_length);
    }

    (path_lengths, path_depths)
}

/// Print a path depth table.
///
/// Format the result of `path_depth` in an odgi-style TSV.
pub fn print_path_depth(gfa: &flatgfa::FlatGFA, lengths: Vec<usize>, depths: Vec<f64>) {
    println!("#path\tstart\tend\tmean.depth");
    for (id, path) in gfa.paths.items() {
        println!(
            "{}\t0\t{}\t{}",
            gfa.get_path_name(path),
            lengths[id.index()],
            depths[id.index()],
        );
    }
}
