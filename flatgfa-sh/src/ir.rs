use enum_map::{Enum, EnumMap};
use smallvec::SmallVec;

/// A reference to a value that instructions read or write.
///
/// Resources have per-kind namespaces, so we can refer to them with their kind
/// and their index. For byte-stream resources, references can also specify how
/// to encode/decode bytes when accessing the resource.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ResourceRef {
    pub kind: ResourceKind,
    pub index: u16,
    pub encoding: Encoding,
}

/// The type of resource. Each kind has a different index space.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Enum)]
pub enum ResourceKind {
    File,
    Stdin,
    Stdout,
    Pipe,
    GFAStore,
    Mmap,
    BEDStore,
}

/// The data encoding to be used when reading or writing the resource.
///
/// This should only be non-`None` for byte-stream resources (File, Stdin,
/// Stdout, Pipe, and Mmap).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Encoding {
    None,
    Gzip,
}

/// An instruction performs one imperative action.
#[derive(Debug)]
pub struct Instr {
    pub inputs: SmallVec<[ResourceRef; 2]>,
    pub output: ResourceRef,
    pub op: Op,
}

/// The operation that an instruction may perform.
#[derive(Debug)]
pub enum Op {
    NodeDepth,
    PathDepth {
        path: Option<String>,
    },
    PathLength {
        path: String,
    },
    Exec {
        command: String,
        args: SmallVec<[String; 4]>,
    },
    ParseGFA,
    MapFile,
    ParseBED,
    MakeWindows {
        size: usize,
    },
    OdgiView,
    IntervalDepth,
    GzipDecompress,
}

#[derive(Debug)]
pub struct Program {
    pub instrs: Vec<Instr>,
    pub file_names: Vec<String>,
    pub rsrc_counts: EnumMap<ResourceKind, u16>,
}

impl ResourceRef {
    /// Refer to a given resource, with no encoding.
    pub fn new(kind: ResourceKind, index: u16) -> Self {
        ResourceRef {
            kind,
            index,
            encoding: Encoding::None,
        }
    }

    /// The (unencoded) standard input resource.
    pub fn stdin() -> Self {
        Self::new(ResourceKind::Stdin, 0)
    }

    /// The (unencoded) standard output resource.
    pub fn stdout() -> Self {
        Self::new(ResourceKind::Stdout, 0)
    }

    /// Get a version of the resource reference marked with a given encoding.
    /// The resource must be a byte stream.
    pub fn encoded(&self, encoding: Encoding) -> Self {
        use ResourceKind::*;
        assert!(matches!(self.kind, File | Mmap | Pipe | Stdin | Stdout));
        Self {
            kind: self.kind,
            index: self.index,
            encoding,
        }
    }
}
