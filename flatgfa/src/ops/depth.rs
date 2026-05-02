use crate::flatgfa;
use crate::pool::Id;
use bit_vec::BitVec;
use std::io::Write;

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

/// Sorta like `std::fmt::Display`, but consumes the thing being printed.
pub trait Emit {
    fn emit(self, f: &mut impl Write) -> std::io::Result<()>;

    fn print(self)
    where
        Self: Sized,
    {
        self.emit(&mut std::io::stdout().lock()).unwrap();
    }
}

/// A printable segment depth table.
///
/// Formats the result of `seg_depth` in an odgi-style TSV.
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
                format_float(self.depths[idx]),
            )?;
        }
        Ok(())
    }
}

/// Format an `f64` in an odgi-like way, with limited decimal digits and without
/// trailing zeroes.
///
/// This is currently inefficient: it first formats with trailing zeroes, and
/// then it trims those zeroes. Surely there is an option out there for avoiding
/// this extra work...
pub fn format_float(x: f64) -> String {
    format!("{:.2}", x)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}
