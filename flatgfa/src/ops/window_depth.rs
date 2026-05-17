use crate::emit::Emit;
use crate::flatbed::{BEDEntry, FlatBED, HeapBEDStore};
use crate::flatgfa::{self, Path};
use crate::ops::depth::{format_float, seg_depth};
use crate::pool::Id;
use crate::pool::Store;
use crate::FlatGFA;
use bstr::BStr;

struct SegmentDepth {
    depth: f64,
    range: (usize, usize),
}

/// Create a BED with "windows" from 0 through `len` in increments of `size`.
///
/// Every row in the resulting BED has the same `name`.
pub fn make_windows(name: &BStr, len: u64, size: u64) -> HeapBEDStore {
    let num_windows = len.div_ceil(size) as usize;
    let mut store = HeapBEDStore {
        name_data: Vec::with_capacity(name.len()).into(),
        entries: Vec::with_capacity(num_windows).into(),
    };
    let name = store.name_data.add_slice(name.as_ref());

    let mut start: u64 = 0;
    while start < len {
        let end = (start + size).min(len);
        store.entries.add(BEDEntry { name, start, end });
        start = end;
    }
    store
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
fn assign_depths(seg_depth: impl IntoIterator<Item = SegmentDepth>, windows: &FlatBED) -> Vec<f64> {
    let mut depths: Vec<f64> = vec![0.0; windows.get_num_entries()];

    // Walk down the segments in the path.
    let mut cur_window_idx = 0;
    for seg in seg_depth {
        // Move through the windows that overlap with this segment.
        while cur_window_idx < windows.get_num_entries() {
            let entry = windows.entries.all()[cur_window_idx];
            let window = (entry.start as usize, entry.end as usize);
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
pub struct IntervalDepth<'a> {
    pub intervals: FlatBED<'a>,
    pub depths: Vec<f64>,
}

impl Emit for IntervalDepth<'_> {
    fn emit(self, f: &mut impl std::io::prelude::Write) -> std::io::Result<()> {
        for (i, entry) in self.intervals.entries.all().iter().enumerate() {
            let depth_str = format_float(self.depths[i], 4);
            let name = self.intervals.get_name_of_entry(entry);
            let start = entry.start;
            let end = entry.end;
            writeln!(f, "{name}\t{start}\t{end}\t{depth_str}")?;
        }
        Ok(())
    }
}

/// Compute the depth for equally-sized windows along a given path.
pub fn window_depth(
    gfa: &flatgfa::FlatGFA,
    path: Id<Path>,
    window_size: usize,
) -> (HeapBEDStore, Vec<f64>) {
    let path_name = gfa.get_path_name(&gfa.paths[path]).to_owned();
    let depth = seg_depth(gfa).0;
    let windows = make_windows(
        path_name.as_ref(),
        path_length(gfa, path) as u64,
        window_size as u64,
    );
    let seg_depths = weighted_depths(gfa, &depth, path);
    let depths = assign_depths(seg_depths, &windows.as_ref());
    (windows, depths)
}

/// Compute the depth for arbitrary intervals and print a BED file.
///
/// The intervals must be (1) along a single path and (2) sorted in increasing
/// order along that path.
pub fn interval_depth(gfa: &flatgfa::FlatGFA, intervals: &FlatBED) -> Vec<f64> {
    // We assume that this BED interval file contains only intervals from a
    // single path. (Relaxing this assumption without sacrificing performance is
    // interesting future work.)
    let path_name = intervals.get_name_of_entry(&intervals.entries.all()[0]);
    let path = gfa.find_path(path_name).expect("path not found in graph");

    // TODO Share this stuff with `window_depth_bed`!
    let depth = seg_depth(gfa).0;
    let seg_depths = weighted_depths(gfa, &depth, path);
    assign_depths(seg_depths, intervals)
}
