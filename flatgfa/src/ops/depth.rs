use crate::emit::Emit;
use crate::flatbed::HeapBEDStore;
use crate::flatgfa;
use crate::pool::Id;
use bit_vec::BitVec;
use std::io::Write;

/// Compute the *depth* and *unique depth* of each segment in the variation graph.
///
/// The depth is defined to be the number of times that a path traverses a given
/// segment. We return two values: the ordinary depth and the *unique* depth,
/// which only counts each path that tarverses a given segment once.
///
/// Both outputs are depth values indexed by segment ID.
pub fn seg_depth_with_uniq(gfa: &flatgfa::FlatGFA) -> (Vec<usize>, Vec<usize>) {
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

/// Compute the (non-unique) depth of each segment in the graph.
///
/// Like `seg_depth_with_uniq`, but only computes the "ordinary" depth, which
/// can be cheaper.
pub fn seg_depth(gfa: &flatgfa::FlatGFA) -> Vec<usize> {
    let mut depths = vec![0; gfa.segs.len()];

    for path in gfa.paths.all().iter() {
        for step in &gfa.steps[path.steps] {
            let seg_id = step.segment().index();
            depths[seg_id] += 1;
        }
    }

    depths
}

/// A printable segment depth table.
///
/// Formats the result of `seg_depth_with_uniq` in an odgi-style TSV.
pub struct SegDepth<'a> {
    pub gfa: &'a flatgfa::FlatGFA<'a>,
    pub depths: Vec<usize>,
    pub uniq_depths: Vec<usize>,
}

impl<'a> Emit for SegDepth<'a> {
    fn emit(self, f: &mut impl Write) -> std::io::Result<()> {
        writeln!(f, "#node.id\tdepth\tdepth.uniq")?;
        for (id, seg) in self.gfa.segs.items() {
            let name: u32 = seg.name as u32;
            writeln!(
                f,
                "{}\t{}\t{}",
                name,
                self.depths[id.index()],
                self.uniq_depths[id.index()],
            )?;
        }
        Ok(())
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

/// A printable path depth table.
///
/// Formats the result of `path_depth` in an odgi-style TSV.
pub struct PathDepth<'a, I: Iterator<Item = Id<flatgfa::Path>>> {
    pub gfa: &'a flatgfa::FlatGFA<'a>,
    pub lengths: Vec<usize>,
    pub depths: Vec<f64>,
    pub paths: I,
}

impl<'a, I> Emit for PathDepth<'a, I>
where
    I: Iterator<Item = Id<flatgfa::Path>>,
{
    fn emit(self, f: &mut impl Write) -> std::io::Result<()> {
        writeln!(f, "#path\tstart\tend\tmean.depth")?;
        for (idx, id) in self.paths.enumerate() {
            writeln!(
                f,
                "{}\t0\t{}\t{}",
                self.gfa.get_path_name(&self.gfa.paths[id]),
                self.lengths[idx],
                format_float(self.depths[idx], 2),
            )?;
        }
        Ok(())
    }
}

impl<'a, I> PathDepth<'a, I>
where
    I: Iterator<Item = Id<flatgfa::Path>>,
{
    /// Emit the depth for each path as a BED table.
    ///
    /// This is currently quite weird because it throws away the actual depth
    /// information and only emits the standard BED start/end positions. This
    /// turns out to be what some workloads actually want, i.e., they don't
    /// really use the depth. But for more general use cases, we should explore
    /// letting FlatBED optionally represent more columns.
    pub fn as_bed(self) -> HeapBEDStore {
        let mut store = HeapBEDStore::default();
        for (idx, id) in self.paths.enumerate() {
            store.add_entry(
                self.gfa.get_path_name(&self.gfa.paths[id]),
                0,
                self.lengths[idx] as u64,
            );
        }
        store
    }
}

/// Format an `f64` in an odgi-like way, with limited decimal digits and without
/// trailing zeroes.
///
/// This is currently inefficient: it first formats with trailing zeroes, and
/// then it trims those zeroes. Surely there is an option out there for avoiding
/// this extra work...
pub fn format_float(x: f64, digits: usize) -> String {
    format!("{:.digits$}", x, digits = digits)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}
