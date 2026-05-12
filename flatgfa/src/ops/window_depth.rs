use crate::flatgfa::{self, Path};
use crate::ops::depth::{format_float, seg_depth};
use crate::pool::Id;
use crate::FlatGFA;

struct SegmentDepth {
    depth: f64,
    range: (usize, usize),
}

/// Create "windows" from 0 through `len` in increments of `size`.
pub fn make_windows(len: usize, size: usize) -> Vec<(usize, usize)> {
    let num_windows = len.div_ceil(size);
    let mut windows = Vec::with_capacity(num_windows);
    let mut start = 0;
    while start < len {
        let end = (start + size).min(len);
        windows.push((start, end));
        start = end;
    }
    windows
}

/// Get the total length (in base pairs) of a single path.
fn path_length(gfa: &FlatGFA, path: Id<Path>) -> usize {
    let path = gfa.paths[path];
    let mut total_len = 0;
    for step in &gfa.steps[path.steps] {
        let seg_id = step.segment().index();
        total_len += gfa.segs.all()[seg_id].len();
    }
    total_len
}

/// Compute the *weighted depths* of every step along a path.
///
/// For each segment in a path, we multiply the segment's depth (which we assume
/// is already computed) by the path's length. We also record the start and end
/// offsets for the segment.
fn weighted_depths<'a>(
    gfa: &'a FlatGFA<'a>,
    depth: &'a [usize],
    path: Id<Path>,
) -> Vec<SegmentDepth> {
    let mut pos = 0;
    gfa.get_path_steps(&gfa.paths[path])
        .map(|step| {
            let segment = gfa.segs[step.segment()];
            let old_pos = pos;
            pos += segment.len();
            let total = depth[step.segment().index()] * segment.len();
            SegmentDepth {
                depth: total as f64,
                range: (old_pos, pos),
            }
        })
        .collect()
}

/// Get the interval of overlap between two other intervals.
///
/// The pairs involved here are (min, max) intervals. Assuming the two intervals
/// overlap, return a new interval capturing that range. Otherwise, return a
/// "negative" interval (where end <= start).
fn overlap(a: (usize, usize), b: (usize, usize)) -> (usize, usize) {
    (a.0.max(b.0), a.1.min(b.1))
}

/// Compute the per-window weighted depth.
///
/// Given weighted segment depths from `weighted_depths`, assign that weight to
/// each of the base-pair ranges in `windows`.
#[allow(clippy::mut_range_bound)]
fn assign_depths(seg_depth: &Vec<SegmentDepth>, windows: &[(usize, usize)]) -> Vec<f64> {
    let mut depths: Vec<f64> = vec![0.0; windows.len()];

    // Walk down the segments in the path.
    let mut cur_window_idx = 0;
    let mut overlap_flag = false;
    for seg in seg_depth {
        // Move through the windows that overlap with this segment.
        for i in cur_window_idx..windows.len() {
            let window = windows[i];
            let (start, end) = overlap(window, seg.range);

            // Is this window at least *partially* within the current segment?
            if end > start {
                // Attribute some of this segment's weight to this window.
                let overlap_amt: f64 = (end - start) as f64 / (seg.range.1 - seg.range.0) as f64;
                depths[i] += (seg.depth * overlap_amt) / ((window.1 - window.0) as f64);

                // Advance to global iteration to this window.
                cur_window_idx = i;
                overlap_flag = true;
            } else if overlap_flag {
                // Do not advance the window; leave it to the next segment.
                overlap_flag = false;
                break;
            }
        }
    }
    depths
}

/// Compute the depth for windows along a path and print a BED file.
pub fn window_depth_bed(gfa: &flatgfa::FlatGFA, path: Id<Path>, window_size: usize) {
    // The actual depth computation.
    let depth = seg_depth(gfa).0;
    let windows = make_windows(path_length(gfa, path), window_size);
    let seg_depths = weighted_depths(gfa, &depth, path);
    let window_depths = assign_depths(&seg_depths, &windows);

    // Print a BED table with these weights.
    let name = gfa.get_path_name(&gfa.paths[path]);
    for i in 0..windows.len() {
        let start = windows[i].0;
        let end = windows[i].1;
        let depth_str = format_float(window_depths[i]);
        println!("{name}\t{start}\t{end}\t{depth_str}");
    }
}
