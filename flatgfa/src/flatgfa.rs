use std::str::FromStr;

use crate::pool::{Index, Pool, PoolTK, Span};
use bstr::BStr;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tinyvec::SliceVec;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// An efficient flattened representation of a GFA file.
///
/// This struct *borrows* the underlying data from some other data store. Namely, the
/// `FlatGFAStore` struct contains `Vec`s as backing stores for each of the slices
/// in this struct. `FlatGFA` itself provides immutable access to the GFA data
/// structure that is agnostic to the location of the underlying bytes.
pub struct FlatGFA<'a> {
    /// A GFA may optionally have a single header line, with a version number.
    /// If this is empty, there is no header line.
    pub header: &'a [u8],

    /// The segment (S) lines in the GFA file.
    pub segs: &'a [Segment],

    /// The path (P) lines.
    pub paths: &'a [Path],

    /// The link (L) lines.
    pub links: &'a [Link],

    /// Paths consist of steps. This is a flat pool of steps, chunks of which are
    /// associated with each path.
    pub steps: &'a [Handle],

    /// The actual base-pair sequences for the segments. This is a pool of
    /// base-pair symbols, chunks of which are associated with each segment.
    ///
    /// TODO: This could certainly use a smaller representation than `u8`
    /// (since we care only about 4 base pairs). If we want to pay the cost
    /// of bit-packing.
    pub seq_data: &'a [u8],

    /// Both paths and links can have overlaps, which are CIGAR sequences. They
    /// are all stored together here in a flat pool, elements of which point
    /// to chunks of `alignment`.
    pub overlaps: &'a [Span],

    /// The CIGAR aligment operations that make up the overlaps. `overlaps`
    /// contains range of indices in this pool.
    pub alignment: &'a [AlignOp],

    /// The string names: currenly, just of paths. (We assume segments have integer
    /// names, so they don't need to be stored separately.)
    pub name_data: &'a [u8],

    /// Segments can come with optional extra fields, which we store in a flat pool
    /// as raw characters because we don't currently care about them.
    pub optional_data: &'a [u8],

    /// An "interleaving" order of GFA lines. This is to preserve perfect round-trip
    /// fidelity: we record the order of lines as we saw them when parsing a GFA file
    /// so we can emit them again in that order. Elements should be `LineKind` values
    /// (but they are checked before we use them).
    pub line_order: &'a [u8],
}

/// GFA graphs consist of "segment" nodes, which are fragments of base-pair sequences
/// that can be strung together into paths.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct Segment {
    /// The segment's name. We assume all names are just plain numbers.
    pub name: usize,

    /// The base-pair sequence for the segment. This is a range in the `seq_data` pool.
    pub seq: Span,

    /// Segments can have optional fields. This is a range in the `optional_data` pool.
    pub optional: Span,
}

impl Segment {
    pub fn len(&self) -> usize {
        self.seq.len()
    }
}

/// A path is a sequence of oriented references to segments.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct Path {
    /// The path's name. This can be an arbitrary string. It is a range in the
    /// `name_data` pool.
    pub name: Span,

    /// The sequence of path steps. This is a range in the `steps` pool.
    pub steps: Span,

    /// The CIGAR overlaps for each step on the path. This is a range in the
    /// `overlaps` pool.
    pub overlaps: Span,
}

/// An allowed edge between two oriented segments.
#[derive(Debug, FromBytes, FromZeroes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct Link {
    /// The source of the edge.
    pub from: Handle,

    // The destination of the edge.
    pub to: Handle,

    /// The CIGAR overlap between the segments. This is a range in the
    /// `alignment` pool.
    pub overlap: Span,
}

impl Link {
    /// Is either end of the link the given segment? If so, return the other end.
    pub fn incident_seg(&self, seg_id: Index) -> Option<Index> {
        if self.from.segment() == seg_id {
            Some(self.to.segment())
        } else if self.to.segment() == seg_id {
            Some(self.from.segment())
        } else {
            None
        }
    }
}

/// A forward or backward direction.
#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Orientation {
    Forward,  // +
    Backward, // -
}

impl FromStr for Orientation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "+" {
            Ok(Orientation::Forward)
        } else if s == "-" {
            Ok(Orientation::Backward)
        } else {
            Err(())
        }
    }
}

/// An oriented reference to a segment.
///
/// A Handle refers to the forward (+) or backward (-) orientation for a given segment.
/// So, logically, it consists of a pair of a segment reference (usize) and an
/// orientation (1 bit). We pack the two values into a single word.
#[derive(Debug, FromBytes, FromZeroes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct Handle(u32);

impl Handle {
    /// Create a new handle referring to a segment ID and an orientation.
    pub fn new(segment: Index, orient: Orientation) -> Self {
        assert!(segment & (1 << (u32::BITS - 1)) == 0, "index too large");
        let orient_bit: u8 = orient.into();
        assert!(orient_bit & !1 == 0, "invalid orientation");
        Self(segment << 1 | (orient_bit as u32))
    }

    /// Get the segment ID. This is an index in the `segs` pool.
    pub fn segment(&self) -> Index {
        self.0 >> 1
    }

    /// Get the orientation (+ or -) for the handle.
    pub fn orient(&self) -> Orientation {
        ((self.0 & 1) as u8).try_into().unwrap()
    }
}

/// The kind of each operation in a CIGAR alignment.
#[derive(Debug, IntoPrimitive, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum AlignOpcode {
    Match,     // M
    Gap,       // N
    Insertion, // D
    Deletion,  // I
}

/// A single operation in a CIGAR alignment, like "3M" or "1D".
///
/// Logically, this is a pair of a number and an `AlignOpcode`. We pack the two
/// into a single u32.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct AlignOp(u32);

impl AlignOp {
    /// Create a new alignment operation from an opcode and count.
    pub fn new(op: AlignOpcode, len: u32) -> Self {
        let op_byte: u8 = op.into();
        assert!(len & !0xff == 0, "length too large");
        Self((len << 8) | (op_byte as u32))
    }

    /// Get the operation (M, I, etc.) for this operation.
    pub fn op(&self) -> AlignOpcode {
        ((self.0 & 0xff) as u8).try_into().unwrap()
    }

    /// Get the length of the operation.
    pub fn len(&self) -> u32 {
        self.0 >> 8
    }
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
#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum LineKind {
    Header,
    Segment,
    Path,
    Link,
}

impl<'a> FlatGFA<'a> {
    /// Get the base-pair sequence for a segment.
    pub fn get_seq(&self, seg: &Segment) -> &BStr {
        self.seq_data[seg.seq.range()].as_ref()
    }

    /// Look up a segment by its name.
    pub fn find_seg(&self, name: usize) -> Option<Index> {
        // TODO Make this more efficient by maintaining the name index? This would not be
        // too hard; we already have the machinery in `parse.rs`...
        self.segs
            .iter()
            .position(|seg| seg.name == name)
            .map(|i| i as Index)
    }

    /// Look up a path by its name.
    pub fn find_path(&self, name: &BStr) -> Option<Index> {
        self.paths
            .iter()
            .position(|path| self.get_path_name(path) == name)
            .map(|i| i as Index)
    }

    /// Get all the steps for a path.
    pub fn get_steps(&self, path: &Path) -> &[Handle] {
        &self.steps[path.steps.range()]
    }

    /// Get all the overlaps for a path. This may be empty (`*` in the GFA file).
    pub fn get_overlaps(&self, path: &Path) -> &[Span] {
        &self.overlaps[path.overlaps.range()]
    }

    /// Get the string name of a path.
    pub fn get_path_name(&self, path: &Path) -> &BStr {
        self.name_data[path.name.range()].as_ref()
    }

    /// Get a handle's associated segment.
    pub fn get_handle_seg(&self, handle: Handle) -> &Segment {
        &self.segs[handle.segment() as usize]
    }

    /// Get the optional data for a segment, as a tab-separated string.
    pub fn get_optional_data(&self, seg: &Segment) -> &BStr {
        self.optional_data[seg.optional.range()].as_ref()
    }

    /// Look up a CIGAR alignment.
    pub fn get_alignment(&self, overlap: &Span) -> Alignment {
        Alignment {
            ops: &self.alignment[overlap.range()],
        }
    }

    /// Get the recorded order of line kinds.
    pub fn get_line_order(&self) -> impl Iterator<Item = LineKind> + 'a {
        self.line_order.iter().map(|b| (*b).try_into().unwrap())
    }
}

/// The data storage pools for a `FlatGFA`.
#[derive(Default)]
pub struct Store<'a, P: PoolFamily<'a>> {
    pub header: P::Pool<u8>,
    pub segs: P::Pool<Segment>,
    pub paths: P::Pool<Path>,
    pub links: P::Pool<Link>,
    pub steps: P::Pool<Handle>,
    pub seq_data: P::Pool<u8>,
    pub overlaps: P::Pool<Span>,
    pub alignment: P::Pool<AlignOp>,
    pub name_data: P::Pool<u8>,
    pub optional_data: P::Pool<u8>,
    pub line_order: P::Pool<u8>,
}

impl<'a, P: PoolFamily<'a>> Store<'a, P> {
    /// Add a header line for the GFA file. This may only be added once.
    pub fn add_header(&mut self, version: &[u8]) {
        assert!(self.header.count() == 0);
        self.header.add_slice(version);
    }

    /// Add a new segment to the GFA file.
    pub fn add_seg(&mut self, name: usize, seq: &[u8], optional: &[u8]) -> Index {
        self.segs.add(Segment {
            name,
            seq: self.seq_data.add_slice(seq),
            optional: self.optional_data.add_slice(optional),
        })
    }

    /// Add a new path.
    pub fn add_path(
        &mut self,
        name: &[u8],
        steps: Span,
        overlaps: impl Iterator<Item = Vec<AlignOp>>,
    ) -> Index {
        let overlaps = self.overlaps.add_iter(
            overlaps
                .into_iter()
                .map(|align| self.alignment.add_iter(align)),
        );
        let name = self.name_data.add_slice(name);
        self.paths.add(Path {
            name,
            steps,
            overlaps,
        })
    }

    /// Add a sequence of steps.
    pub fn add_steps(&mut self, steps: impl Iterator<Item = Handle>) -> Span {
        self.steps.add_iter(steps)
    }

    /// Add a single step.
    pub fn add_step(&mut self, step: Handle) -> Index {
        self.steps.add(step)
    }

    /// Add a link between two (oriented) segments.
    pub fn add_link(&mut self, from: Handle, to: Handle, overlap: Vec<AlignOp>) -> Index {
        self.links.add(Link {
            from,
            to,
            overlap: self.alignment.add_iter(overlap),
        })
    }

    /// Record a line type to preserve the line order.
    pub fn record_line(&mut self, kind: LineKind) {
        self.line_order.add(kind.into());
    }

    /// Borrow a FlatGFA view of this data store.
    pub fn view(&self) -> FlatGFA {
        FlatGFA {
            header: &self.header,
            segs: &self.segs,
            paths: &self.paths,
            links: &self.links,
            name_data: &self.name_data,
            seq_data: &self.seq_data,
            steps: &self.steps,
            overlaps: &self.overlaps,
            alignment: &self.alignment,
            optional_data: &self.optional_data,
            line_order: &self.line_order,
        }
    }
}

pub trait PoolFamily<'a> {
    type Pool<T: Clone + 'a>: crate::pool::Pool<T>;
}

#[derive(Default)]
pub struct VecPoolFamily;
impl<'a> PoolFamily<'a> for VecPoolFamily {
    type Pool<T: Clone + 'a> = Vec<T>;
}

pub struct SliceVecPoolFamily;
impl<'a> PoolFamily<'a> for SliceVecPoolFamily {
    type Pool<T: Clone + 'a> = SliceVec<'a, T>;
}

/// A store for `FlatGFA` data backed by fixed-size slices.
///
/// This store contains `SliceVec`s, which act like `Vec`s but are allocated within
/// a fixed region. This means they have a maximum size, but they can directly map
/// onto the contents of a file.
pub type SliceStore<'a> = Store<'a, SliceVecPoolFamily>;

/// A mutable, in-memory data store for `FlatGFA`.
///
/// This store contains a bunch of `Vec`s: one per array required to implement a
/// `FlatGFA`. It exposes an API for building up a GFA data structure, so it is
/// useful for creating new ones from scratch.
pub type HeapStore = Store<'static, VecPoolFamily>;
