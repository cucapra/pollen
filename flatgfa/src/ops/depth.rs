use crate::flatgfa;
use crate::pool::Id;
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
pub fn path_depth<I>(gfa: &flatgfa::FlatGFA, paths: I) -> (Vec<usize>, Vec<f64>)
where
    I: Iterator<Item = Id<flatgfa::Path>>,
{
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
    for path_id in paths {
        let (length, depth) = measure_path(gfa, path_id, &seg_depths);
        path_lengths.push(length);
        path_depths.push(depth);
    }

    (path_lengths, path_depths)
}

/// Get a path's length (in base pairs) and average depth.
///
/// Requires walking the path to measure its total length.
fn measure_path(
    gfa: &flatgfa::FlatGFA,
    path: Id<flatgfa::Path>,
    seg_depths: &[usize],
) -> (usize, f64) {
    let mut depth = 0;
    let mut length = 0;
    let path = gfa.paths[path];
    for step in &gfa.steps[path.steps] {
        let len = gfa.segs[step.segment()].len();
        depth += seg_depths[step.segment().index()] * len;
        length += len;
    }
    let avg_depth = (depth as f64) / (length as f64);
    (length, avg_depth)
}

/// Print a path depth table.
///
/// Format the result of `path_depth` in an odgi-style TSV.
pub fn print_path_depth<I>(gfa: &flatgfa::FlatGFA, lengths: Vec<usize>, depths: Vec<f64>, paths: I)
where
    I: Iterator<Item = Id<flatgfa::Path>>,
{
    println!("#path\tstart\tend\tmean.depth");
    for (idx, id) in paths.enumerate() {
        println!(
            "{}\t0\t{}\t{}",
            gfa.get_path_name(&gfa.paths[id]),
            lengths[idx],
            depths[idx],
        );
    }
}
