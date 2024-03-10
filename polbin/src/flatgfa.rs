use bstr::{BStr, BString};
use std::ops::Range;

/// GFA graphs consist of "segment" nodes, which are fragments of base-pair sequences
/// that can be strung together into paths.
#[derive(Debug)]
pub struct Segment {
    /// The segment's name. We assume all names are just plain numbers.
    pub name: usize,

    /// The base-pair sequence for the segment. This is a range in the `seq_data` pool.
    pub seq: Range<usize>,

    /// Segments can have optional fields. This is a range in the `optional_data` pool.
    pub optional: Range<usize>,
}

/// A path is a sequence of oriented references to segments.
#[derive(Debug)]
pub struct Path {
    /// The path's name. This can be an arbitrary string. It is a renge in the
    /// `name_data` pool.
    pub name: Range<usize>,

    /// The squence of path steps. This is a range in the `steps` pool.
    pub steps: Range<usize>,

    /// The CIGAR overlaps for each step on the path. This is a range in the
    /// `overlaps` pool.
    pub overlaps: Range<usize>,
}

/// An allowed edge between two oriented segments.
#[derive(Debug)]
pub struct Link {
    /// The source of the edge.
    pub from: Handle,

    // The destination of the edge.
    pub to: Handle,

    /// The CIGAR overlap between the segments. This is a range in the
    /// `overlaps` pool.
    pub overlap: Range<usize>,
}

/// A foroward or backward direction.
#[derive(Debug, PartialEq)]
pub enum Orientation {
    Forward,  // +
    Backward, // -
}

/// An oriented reference to a segment.
///
/// We can refer to the + (forward) or - (backward) handle for a given segment.
#[derive(Debug, PartialEq)]
pub struct Handle {
    /// The segment we're referring to. This is an index in the `segs` pool.
    pub segment: usize,

    /// The orientation (+ or -) of the reference.
    pub orient: Orientation,
}

/// The kind of each operation in a CIGAR alignment.
#[derive(Debug)]
pub enum AlignOpcode {
    Match,     // M
    Gap,       // N
    Insertion, // D
    Deletion,  // I
}

/// A single operation in a CIGAR alignment, like "3M" or "1D".
#[derive(Debug)]
pub struct AlignOp {
    /// The operation (M, I, etc.).
    pub op: AlignOpcode,

    /// The count for this operation.
    pub len: u32,
}

/// An entire CIGAR alignment string, like "3M1D2M".
#[derive(Debug)]
#[repr(transparent)]
pub struct Alignment<'a> {
    /// The sequence of operations that make up the alignment.
    pub ops: &'a [AlignOp],
}

/// A kind of GFA line. We use this in `line_order` to preserve the textual order
/// in a GFA file for round-tripping.
#[derive(Debug)]
pub enum LineKind {
    Header,
    Segment,
    Path,
    Link,
}

/// An efficient flattened representation of a GFA file.
#[derive(Debug, Default)]
pub struct FlatGFA {
    /// A GFA may optionally have a single header line, with a version number.
    pub header: Option<BString>,

    /// The segment (S) lines in the GFA file.
    pub segs: Vec<Segment>,

    /// The path (P) lines.
    pub paths: Vec<Path>,

    /// The link (L) lines.
    pub links: Vec<Link>,

    /// Paths consist of steps. This is a flat pool of steps, chunks of which are
    /// associated with each path.
    steps: Vec<Handle>,

    /// The actual base-pair sequences for the segments. This is a pool of
    /// base-pair symbols, chunks of which are associated with each segment.
    ///
    /// TODO: This could certainly use a smaller representation than `u8`
    /// (since we care only about 4 base pairs). If we want to pay the cost
    /// of bit-packing.
    seq_data: Vec<u8>,

    /// Both paths and links can have overlaps, which are CIGAR sequences. They
    /// are all stored together here in a flat pool, elements of which point
    /// to chunks of `alignment`.
    overlaps: Vec<Range<usize>>,

    /// The CIGAR aligment operations that make up the overlaps. `overlaps`
    /// contains range of indices in this pool.
    alignment: Vec<AlignOp>,

    /// The string names: currenly, just of paths. (We assume segments have integer
    /// names, so they don't need to be stored separately.)
    name_data: BString,

    /// Segments can come with optional extra fields, which we store in a flat pool
    /// as raw characters because we don't currently care about them.
    optional_data: BString,

    /// An "interleaving" order of GFA lines. This is to preserve perfect round-trip
    /// fidelity: we record the order of lines as we saw them when parsing a GFA file
    /// so we can emit them again in that order.
    pub(crate) line_order: Vec<LineKind>,
}

impl FlatGFA {
    /// Get the base-pair sequence for a segment.
    pub fn get_seq(&self, seg: &Segment) -> &BStr {
        self.seq_data[seg.seq.clone()].as_ref()
    }

    /// Get all the steps for a path.
    pub fn get_steps(&self, path: &Path) -> &[Handle] {
        &self.steps[path.steps.clone()]
    }

    /// Get all the overlaps for a path. This may be empty (`*` in the GFA file).
    pub fn get_overlaps(&self, path: &Path) -> &[Range<usize>] {
        &self.overlaps[path.overlaps.clone()]
    }

    /// Get the string name of a path.
    pub fn get_path_name(&self, path: &Path) -> &BStr {
        self.name_data[path.name.clone()].as_ref()
    }

    /// Get the optional data for a segment, as a tab-separated string.
    pub fn get_optional_data(&self, seg: &Segment) -> &BStr {
        self.optional_data[seg.optional.clone()].as_ref()
    }

    /// Look up a CIGAR alignment.
    pub fn get_alignment(&self, overlap: &Range<usize>) -> Alignment {
        Alignment {
            ops: &self.alignment[overlap.clone()],
        }
    }

    /// Add a header line for the GFA file. This may only be added once.
    pub fn add_header(&mut self, version: Vec<u8>) {
        assert!(self.header.is_none());
        self.header = Some(version.into());
    }

    /// Add a new segment to the GFA file.
    pub fn add_seg(&mut self, name: usize, seq: Vec<u8>, optional: Vec<u8>) -> usize {
        pool_push(
            &mut self.segs,
            Segment {
                name,
                seq: pool_append(&mut self.seq_data, seq),
                optional: pool_append(&mut self.optional_data, optional),
            },
        )
    }

    /// Add a new path.
    pub fn add_path(
        &mut self,
        name: Vec<u8>,
        steps: Vec<Handle>,
        overlaps: Vec<Vec<AlignOp>>,
    ) -> usize {
        let overlap_count = overlaps.len();
        let overlaps = pool_extend(
            &mut self.overlaps,
            overlaps
                .into_iter()
                .map(|align| pool_append(&mut self.alignment, align)),
            overlap_count,
        );

        pool_push(
            &mut self.paths,
            Path {
                name: pool_append(&mut self.name_data, name),
                steps: pool_append(&mut self.steps, steps),
                overlaps,
            },
        )
    }

    /// Add a link between two (oriented) segments.
    pub fn add_link(&mut self, from: Handle, to: Handle, overlap: Vec<AlignOp>) -> usize {
        pool_push(
            &mut self.links,
            Link {
                from,
                to,
                overlap: pool_append(&mut self.alignment, overlap),
            },
        )
    }

    /// Record a line type to preserve the line order.
    pub fn record_line(&mut self, kind: LineKind) {
        self.line_order.push(kind);
    }
}

/// Add an item to a "pool" vector and get the new index (ID).
fn pool_push<T>(vec: &mut Vec<T>, item: T) -> usize {
    let len = vec.len();
    vec.push(item);
    len
}

/// Add an entire vector of items to a "pool" vector and return the
/// range of new indices (IDs).
fn pool_append<T>(vec: &mut Vec<T>, items: Vec<T>) -> Range<usize> {
    let count = items.len();
    pool_extend(vec, items, count)
}

/// Like `pool_append`, for an iterator. It's pretty important that `count`
/// actually be the number of items in the iterator!
fn pool_extend<T>(
    vec: &mut Vec<T>,
    iter: impl IntoIterator<Item = T>,
    count: usize,
) -> Range<usize> {
    let range = vec.len()..(vec.len() + count);
    let old_len = vec.len();
    vec.extend(iter);
    assert_eq!(vec.len(), old_len + count);
    range
}
