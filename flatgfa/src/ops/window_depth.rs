use crate::emit::Emit;
use crate::flatbed::FlatBED;
use crate::flatgfa::{self, Path};
use crate::ops::depth::{format_float, seg_depth};
use crate::pool::Id;
use crate::FlatGFA;
use bstr::BString;

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
) -> impl Iterator<Item = SegmentDepth> + 'a {
    let mut pos = 0;
    gfa.get_path_steps(&gfa.paths[path]).map(move |step| {
        let segment = gfa.segs[step.segment()];
        let old_pos = pos;
        pos += segment.len();
        let total = depth[step.segment().index()] * segment.len();
        SegmentDepth {
            depth: total as f64,
            range: (old_pos, pos),
        }
    })
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
fn assign_depths(
    seg_depth: impl IntoIterator<Item = SegmentDepth>,
    windows: &[(usize, usize)],
) -> Vec<f64> {
    let mut depths: Vec<f64> = vec![0.0; windows.len()];

    // Walk down the segments in the path.
    let mut cur_window_idx = 0;
    for seg in seg_depth {
        // Move through the windows that overlap with this segment.
        while cur_window_idx < windows.len() {
            let window = windows[cur_window_idx];
            let (start, end) = overlap(window, seg.range);

            // Is this window at least *partially* within the current segment?
            if end > start {
                // Attribute some of this segment's weight to this window.
                let overlap_amt: f64 = (end - start) as f64 / (seg.range.1 - seg.range.0) as f64;
                depths[cur_window_idx] +=
                    (seg.depth * overlap_amt) / ((window.1 - window.0) as f64);
            }

            // Pause global iteration when window overlaps with the next segment
            // and switch to the next segment
            if window.1 > seg.range.1 {
                break;
            }

            // Advance global iteration to the next window
            cur_window_idx += 1;
        }
    }
    depths
}

/// The result of a window or arbitrary interval depth computation.
///
/// The result of `window_depth` or `interval_depth` is a list of offset
/// intervals in a single path along with the average depths of those intervals.
pub struct IntervalDepth {
    path_name: BString,
    intervals: Vec<(usize, usize)>,
    depths: Vec<f64>,
}

impl Emit for IntervalDepth {
    fn emit(self, f: &mut impl std::io::prelude::Write) -> std::io::Result<()> {
        for (i, (start, end)) in self.intervals.into_iter().enumerate() {
            let depth_str = format_float(self.depths[i], 4);
            writeln!(f, "{}\t{start}\t{end}\t{depth_str}", self.path_name)?;
        }
        Ok(())
    }
}

/// Compute the depth for equally-sized windows along a given path and print a
/// BED file.
pub fn window_depth(gfa: &flatgfa::FlatGFA, path: Id<Path>, window_size: usize) -> IntervalDepth {
    let depth = seg_depth(gfa).0;
    let windows = make_windows(path_length(gfa, path), window_size);
    let seg_depths = weighted_depths(gfa, &depth, path);
    let window_depths = assign_depths(seg_depths, &windows);

    IntervalDepth {
        path_name: gfa.get_path_name(&gfa.paths[path]).to_owned(),
        intervals: windows,
        depths: window_depths,
    }
}

/// Compute the depth for arbitrary intervals and print a BED file.
///
/// The intervals must be (1) along a single path and (2) sorted in increasing
/// order along that path.
pub fn interval_depth(gfa: &flatgfa::FlatGFA, intervals: &FlatBED) -> IntervalDepth {
    // We assume that this BED interval file contains only intervals from a
    // single path. (Relaxing this assumption without sacrificing performance is
    // interesting future work.)
    let path_name = intervals.get_name_of_entry(&intervals.entries.all()[0]);
    let path = gfa.find_path(path_name).expect("path not found in graph");

    // TODO Avoid this conversion cost!
    let intervals: Vec<_> = intervals
        .entries
        .all()
        .iter()
        .map(|entry| (entry.start as usize, entry.end as usize))
        .collect();

    // TODO Share this stuff with `window_depth_bed`!
    let depth = seg_depth(gfa).0;
    let seg_depths = weighted_depths(gfa, &depth, path);
    let interval_depths = assign_depths(seg_depths, &intervals);

    IntervalDepth {
        path_name: path_name.to_owned(),
        intervals,
        depths: interval_depths,
    }
}
