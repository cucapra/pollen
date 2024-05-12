use std::str::FromStr;

use crate::pool::{self, Id, Pool, Span, Store};
use bstr::BStr;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// An efficient flattened representation of a GFA file.
///
/// This struct *borrows* the underlying data from some other data store. Namely, the
/// `GFAStore` structs contain `Vec`s or `Vec`-like arenas as backing stores for each
/// of the slices in this struct. `FlatGFA` itself provides access to the GFA data
/// structure that is agnostic to the location of the underlying bytes. However, all
/// its components have a fixed size; unlike the underlying `GFAStore`, it is not
/// possible to add new objects.
pub struct FlatGFA<'a> {
    /// A GFA may optionally have a single header line, with a version number.
    /// If this is empty, there is no header line.
    pub header: Pool<'a, u8>,

    /// The segment (S) lines in the GFA file.
    pub segs: Pool<'a, Segment>,

    /// The path (P) lines.
    pub paths: Pool<'a, Path>,

    /// The link (L) lines.
    pub links: Pool<'a, Link>,

    /// Paths consist of steps. This is a flat pool of steps, chunks of which are
    /// associated with each path.
    pub steps: Pool<'a, Handle>,

    /// The actual base-pair sequences for the segments. This is a pool of
    /// base-pair symbols, chunks of which are associated with each segment.
    ///
    /// TODO: This could certainly use a smaller representation than `u8`
    /// (since we care only about 4 base pairs). If we want to pay the cost
    /// of bit-packing.
    pub seq_data: Pool<'a, u8>,

    /// Both paths and links can have overlaps, which are CIGAR sequences. They
    /// are all stored together here in a flat pool, elements of which point
    /// to chunks of `alignment`.
    pub overlaps: Pool<'a, Span<AlignOp>>,

    /// The CIGAR aligment operations that make up the overlaps. `overlaps`
    /// contains range of indices in this pool.
    pub alignment: Pool<'a, AlignOp>,

    /// The string names: currenly, just of paths. (We assume segments have integer
    /// names, so they don't need to be stored separately.)
    pub name_data: Pool<'a, u8>,

    /// Segments can come with optional extra fields, which we store in a flat pool
    /// as raw characters because we don't currently care about them.
    pub optional_data: Pool<'a, u8>,

    /// An "interleaving" order of GFA lines. This is to preserve perfect round-trip
    /// fidelity: we record the order of lines as we saw them when parsing a GFA file
    /// so we can emit them again in that order. Elements should be `LineKind` values
    /// (but they are checked before we use them).
    pub line_order: Pool<'a, u8>,
}

/// GFA graphs consist of "segment" nodes, which are fragments of base-pair sequences
/// that can be strung together into paths.
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(packed)]
pub struct Segment {
    /// The segment's name. We assume all names are just plain numbers.
    pub name: usize,

    /// The base-pair sequence for the segment. This is a range in the `seq_data` pool.
    pub seq: Span<u8>,

    /// Segments can have optional fields. This is a range in the `optional_data` pool.
    pub optional: Span<u8>,
}

impl Segment {
    #[allow(clippy::len_without_is_empty)]
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
    pub name: Span<u8>,

    /// The sequence of path steps. This is a range in the `steps` pool.
    pub steps: Span<Handle>,

    /// The CIGAR overlaps for each step on the path. This is a range in the
    /// `overlaps` pool.
    pub overlaps: Span<Span<AlignOp>>,
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
    pub overlap: Span<AlignOp>,
}

impl Link {
    /// Is either end of the link the given segment? If so, return the other end.
    pub fn incident_seg(&self, seg_id: Id<Segment>) -> Option<Id<Segment>> {
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
    pub fn new(segment: Id<Segment>, orient: Orientation) -> Self {
        let seg_num: u32 = segment.into();
        assert!(seg_num & (1 << (u32::BITS - 1)) == 0, "index too large");
        let orient_bit: u8 = orient.into();
        assert!(orient_bit & !1 == 0, "invalid orientation");
        Self(seg_num << 1 | (orient_bit as u32))
    }

    /// Get the segment ID. This is an index in the `segs` pool.
    pub fn segment(&self) -> Id<Segment> {
        (self.0 >> 1).into()
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

    /// Check whether there are zero operations in this alignment.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        self.seq_data[seg.seq].as_ref()
    }

    /// Look up a segment by its name.
    pub fn find_seg(&self, name: usize) -> Option<Id<Segment>> {
        // TODO Make this more efficient by maintaining the name index? This would not be
        // too hard; we already have the machinery in `parse.rs`...
        self.segs.search(|seg| seg.name == name)
    }

    /// Look up a path by its name.
    pub fn find_path(&self, name: &BStr) -> Option<Id<Path>> {
        self.paths.search(|path| self.get_path_name(path) == name)
    }

    /// Get the string name of a path.
    pub fn get_path_name(&self, path: &Path) -> &BStr {
        self.name_data[path.name].as_ref()
    }

    /// Get a handle's associated segment.
    pub fn get_handle_seg(&self, handle: Handle) -> &Segment {
        &self.segs[handle.segment()]
    }

    /// Get the optional data for a segment, as a tab-separated string.
    pub fn get_optional_data(&self, seg: &Segment) -> &BStr {
        self.optional_data[seg.optional].as_ref()
    }

    /// Look up a CIGAR alignment.
    pub fn get_alignment(&self, overlap: Span<AlignOp>) -> Alignment {
        Alignment {
            ops: &self.alignment[overlap],
        }
    }

    /// Get the recorded order of line kinds.
    pub fn get_line_order(&self) -> impl Iterator<Item = LineKind> + 'a {
        self.line_order
            .all()
            .iter()
            .map(|b| (*b).try_into().unwrap())
    }
}

/// The data storage pools for a `FlatGFA`.
#[derive(Default)]
pub struct GFAStore<'a, P: StoreFamily<'a>> {
    pub header: P::Store<u8>,
    pub segs: P::Store<Segment>,
    pub paths: P::Store<Path>,
    pub links: P::Store<Link>,
    pub steps: P::Store<Handle>,
    pub seq_data: P::Store<u8>,
    pub overlaps: P::Store<Span<AlignOp>>,
    pub alignment: P::Store<AlignOp>,
    pub name_data: P::Store<u8>,
    pub optional_data: P::Store<u8>,
    pub line_order: P::Store<u8>,
}

impl<'a, P: StoreFamily<'a>> GFAStore<'a, P> {
    /// Add a header line for the GFA file. This may only be added once.
    pub fn add_header(&mut self, version: &[u8]) {
        assert!(self.header.as_ref().is_empty());
        self.header.add_slice(version);
    }

    /// Add a new segment to the GFA file.
    pub fn add_seg(&mut self, name: usize, seq: &[u8], optional: &[u8]) -> Id<Segment> {
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
        steps: Span<Handle>,
        overlaps: impl Iterator<Item = Vec<AlignOp>>,
    ) -> Id<Path> {
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
    pub fn add_steps(&mut self, steps: impl Iterator<Item = Handle>) -> Span<Handle> {
        self.steps.add_iter(steps)
    }

    /// Add a single step.
    pub fn add_step(&mut self, step: Handle) -> Id<Handle> {
        self.steps.add(step)
    }

    /// Add a link between two (oriented) segments.
    pub fn add_link(&mut self, from: Handle, to: Handle, overlap: Vec<AlignOp>) -> Id<Link> {
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
    pub fn as_ref(&self) -> FlatGFA {
        FlatGFA {
            header: self.header.as_ref(),
            segs: self.segs.as_ref(),
            paths: self.paths.as_ref(),
            links: self.links.as_ref(),
            name_data: self.name_data.as_ref(),
            seq_data: self.seq_data.as_ref(),
            steps: self.steps.as_ref(),
            overlaps: self.overlaps.as_ref(),
            alignment: self.alignment.as_ref(),
            optional_data: self.optional_data.as_ref(),
            line_order: self.line_order.as_ref(),
        }
    }
}

pub trait StoreFamily<'a> {
    type Store<T: Clone + 'a>: pool::Store<T>;
}

#[derive(Default)]
pub struct HeapFamily;
impl<'a> StoreFamily<'a> for HeapFamily {
    type Store<T: Clone + 'a> = pool::HeapStore<T>;
}

pub struct FixedFamily;
impl<'a> StoreFamily<'a> for FixedFamily {
    type Store<T: Clone + 'a> = pool::FixedStore<'a, T>;
}

/// A store for `FlatGFA` data backed by fixed-size slices.
///
/// This store contains `SliceVec`s, which act like `Vec`s but are allocated within
/// a fixed region. This means they have a maximum size, but they can directly map
/// onto the contents of a file.
pub type FixedGFAStore<'a> = GFAStore<'a, FixedFamily>;

/// A mutable, in-memory data store for `FlatGFA`.
///
/// This store contains a bunch of `Vec`s: one per array required to implement a
/// `FlatGFA`. It exposes an API for building up a GFA data structure, so it is
/// useful for creating new ones from scratch.
pub type HeapGFAStore = GFAStore<'static, HeapFamily>;
