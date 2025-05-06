use flatgfa::namemap::NameMap;
use flatgfa::ops::gaf::{ChunkEvent, GAFParser};
use flatgfa::pool::Id;
use flatgfa::{self, file, memfile, print, FlatGFA, Handle, HeapGFAStore};
use memmap::Mmap;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PySlice};
use std::io::Write;
use std::str;
use std::sync::Arc;

/// Storage for a FlatGFA.
///
/// This may be either an in-memory data structure or a memory-mapped file. It exposes a
/// uniform interface to the FlatGFA data via `view`.
///
enum Store {
    Heap(Box<HeapGFAStore>),
    File(memmap::Mmap),
}

impl Store {
    /// Parse a text GFA file.
    fn parse_file(filename: &str) -> Self {
        let file = memfile::map_file(filename);
        Self::parse_gfa(file.as_ref())
    }

    /// Parse a GFA graph from a byte buffer.
    fn parse_gfa(data: &[u8]) -> Self {
        let store = flatgfa::parse::Parser::for_heap().parse_mem(data);
        Self::Heap(Box::new(store))
    }

    /// Load a FlatGFA binary file.
    fn load(filename: &str) -> Self {
        let mmap = memfile::map_file(filename);
        Self::File(mmap)
    }

    /// Get the FlatGFA stored here.
    fn view(&self) -> FlatGFA {
        // TK It seems wasteful to check the type of store every time... and to construct
        // the view every time. It's probably possible to fix this with a self-reference,
        // e.g., with the `owning_ref` crate.
        match self {
            Store::Heap(ref store) => (**store).as_ref(),
            Store::File(ref mmap) => file::view(mmap),
        }
    }
}

/// An efficient representation of a Graphical Fragment Assembly (GFA) file.
#[pyclass(frozen)]
#[pyo3(name = "FlatGFA", module = "flatgfa")]
struct PyFlatGFA(Arc<Store>);

/// Parse a GFA file into our FlatGFA representation.
#[pyfunction]
fn parse(filename: &str) -> PyFlatGFA {
    PyFlatGFA(Arc::new(Store::parse_file(filename)))
}

/// Parse a GFA file from a bytestring into our FlatGFA representation.
#[pyfunction]
fn parse_bytes(bytes: &[u8]) -> PyFlatGFA {
    PyFlatGFA(Arc::new(Store::parse_gfa(bytes)))
}

/// Load a binary FlatGFA file.
///
/// This function should be fast to call because it does not actually read the file's data.
/// It memory-maps the file so subsequent accesses will actually read the data "on demand."
/// You can produce these files with :meth:`FlatGFA.write_flatgfa`.
#[pyfunction]
fn load(filename: &str) -> PyFlatGFA {
    PyFlatGFA(Arc::new(Store::load(filename)))
}

#[pymethods]
impl PyFlatGFA {
    /// The segments (nodes) in the graph, as a :class:`SegmentList`.
    #[getter]
    fn segments(&self) -> SegmentList {
        SegmentList(ListRef {
            store: self.0.clone(),
            start: 0,
            end: self.0.view().segs.len() as u32,
        })
    }

    /// The paths in the graph, as a :class:`PathList`.
    #[getter]
    fn paths(&self) -> PathList {
        PathList(ListRef {
            store: self.0.clone(),
            start: 0,
            end: self.0.view().paths.len() as u32,
        })
    }

    /// The links (edges) in the graph, as a :class:`LinkList`.
    #[getter]
    fn links(&self) -> LinkList {
        LinkList(ListRef {
            store: self.0.clone(),
            start: 0,
            end: self.0.view().links.len() as u32,
        })
    }

    fn __str__(&self) -> String {
        format!("{}", &self.0.view())
    }

    /// Write the graph as a GFA text file.
    fn write_gfa(&self, filename: &str) -> PyResult<()> {
        let mut file = std::fs::File::create(filename)?;
        write!(file, "{}", &self.0.view())?;
        Ok(())
    }

    /// Write the graph as a binary FlatGFA file.
    ///
    /// You can read the resulting file with :func:`load`.
    fn write_flatgfa(&self, filename: &str) -> PyResult<()> {
        let gfa = self.0.view();
        let mut mmap = memfile::map_new_file(filename, file::size(&gfa) as u64);
        file::dump(&gfa, &mut mmap);
        mmap.flush()?;
        Ok(())
    }
    fn all_reads(&self, gaf: &str) -> PyGAFParser {
        let gfa = self.0.view();
        let name_map = flatgfa::namemap::NameMap::build(&gfa);
        let gaf_buf = Arc::new(flatgfa::memfile::map_file(gaf));
        PyGAFParser {
            gaf_buf: OwnedGAFParser { mmap: gaf_buf },
            store: self.0.clone(),
            name_map,
            pos: 0,

    #[getter]
    fn size(&self) -> usize {
        let gfa = self.0.view();
        file::size(&gfa)
    }

    fn print_gaf_lookup(&self, gaf: &str) {
        let gfa = self.0.view();

        let name_map = flatgfa::namemap::NameMap::build(&gfa);

        let gaf_buf = flatgfa::memfile::map_file(gaf);
        let parser = flatgfa::ops::gaf::GAFParser::new(&gaf_buf);

        // Print the actual sequences for each chunk in the GAF.
        for read in parser {
            print!("{}\t", read.name);
            for event in flatgfa::ops::gaf::PathChunker::new(&gfa, &name_map, read) {
                event.print_seq(&gfa);
            }
            println!();
        }
    }

}

#[pyclass(frozen)]
#[pyo3(name = "ChunkEvent", module = "flatgfa")]
struct PyChunkEvent {
    chunk_event: Arc<ChunkEvent>,
    gfa: Arc<Store>,
}

#[pymethods]
impl PyChunkEvent {
    #[getter]
    fn handle(&self) -> PyHandle {
        PyHandle {
            store: self.gfa.clone(),
            handle: self.chunk_event.handle,
        }
    }

    #[getter]
    fn range(&self) -> (usize, usize) {
        match self.chunk_event.range {
            flatgfa::ops::gaf::ChunkRange::None => (1, 0),
            flatgfa::ops::gaf::ChunkRange::All => {
                let inner_gfa = self.gfa.view();
                let seg = inner_gfa.segs[self.chunk_event.handle.segment()];
                (0, seg.len() - 1)
            }
            flatgfa::ops::gaf::ChunkRange::Partial(start, end) => (start, end),
        }
    }

    fn sequence(&self) -> String {
        let inner_gfa = self.gfa.view();
        let seq = inner_gfa.get_seq_oriented(self.chunk_event.handle);

        match self.chunk_event.range {
            flatgfa::ops::gaf::ChunkRange::Partial(start, end) => seq.slice(start..end).to_string(),
            flatgfa::ops::gaf::ChunkRange::All => seq.to_string(),
            flatgfa::ops::gaf::ChunkRange::None => "".to_string(),
        }
    }
}

/// A reference to a list of *any* type within a FlatGFA.
///
/// We expose various type-specific "XList" types to Python, and they are all wrappers
/// over this data. They just have to access different fields in the underlying store.
struct ListRef {
    store: Arc<Store>,
    start: u32,
    end: u32,
}

impl ListRef {
    fn len(&self) -> u32 {
        self.end - self.start
    }

    fn index(&self, i: u32) -> EntityRef {
        assert!(i < self.len());
        EntityRef {
            store: self.store.clone(),
            index: self.start + i,
        }
    }

    fn slice(&self, start: u32, end: u32) -> Self {
        assert!(start <= end);
        assert!(end <= self.len());
        Self {
            store: self.store.clone(),
            start: self.start + start,
            end: self.start + end,
        }
    }

    /// A suitable implementation of `__getitem__` for Python classes.
    fn py_getitem<L, E>(&self, arg: SliceOrInt, py: Python) -> PyResult<PyObject>
    where
        L: From<ListRef> + IntoPy<PyObject>,
        E: From<EntityRef> + IntoPy<PyObject>,
    {
        match arg {
            SliceOrInt::Slice(slice) => {
                let indices = py_slice_indices(slice, self.len())?;
                if indices.step == 1 {
                    Ok(L::from(self.slice(indices.start as u32, indices.stop as u32)).into_py(py))
                } else {
                    Err(PyIndexError::new_err("only unit step is supported"))
                }
            }
            SliceOrInt::Int(int) => Ok(E::from(self.index(int as u32)).into_py(py)),
        }
    }
}

/// A reference to a specific thing within a FlatGFA.
///
/// Types like `PySegment` and `PyPath` all wrap one of these.
struct EntityRef {
    store: Arc<Store>,
    index: u32,
}

impl PartialEq for EntityRef {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.store) == Arc::as_ptr(&other.store) && self.index == other.index
    }
}

impl Eq for EntityRef {}

impl EntityRef {
    /// Produce a suitable Python `__repr__` string for this reference.
    fn py_repr(&self, class: &str) -> String {
        format!("<{} {}>", class, self.index)
    }

    /// Get the ID object (for use in pool lookup).
    fn id<T>(&self) -> Id<T> {
        self.index.into()
    }
}

/// A suitable argument to `__getitem__`
///
/// Stolen from this GitHub discussion:
/// https://github.com/PyO3/pyo3/issues/1855#issuecomment-962573796
#[derive(FromPyObject)]
enum SliceOrInt<'a> {
    Slice(&'a PySlice),
    Int(isize),
}

/// Generate the Python types for an iterable container of GFA objects.
macro_rules! gen_container {
    ($type: ident, $field: ident, $pytype: ident, $list: ident, $iter: ident) => {
        #[pymethods]
        impl $list {
            fn __getitem__(&self, arg: SliceOrInt, py: Python) -> PyResult<PyObject> {
                self.0.py_getitem::<$list, $pytype>(arg, py)
            }

            fn __iter__(&self) -> $iter {
                $iter {
                    store: self.0.store.clone(),
                    index: self.0.start,
                    end: self.0.end,
                }
            }

            fn __len__(&self) -> usize {
                self.0.len() as usize
            }
        }

        #[pyclass]
        #[pyo3(module = "flatgfa")]
        struct $iter {
            store: Arc<Store>,
            index: u32,
            end: u32,
        }

        #[pymethods]
        impl $iter {
            fn __iter__(self_: Py<Self>) -> Py<Self> {
                self_
            }

            fn __next__(&mut self) -> Option<$pytype> {
                if self.index < self.end as u32 {
                    let obj = $pytype(EntityRef {
                        store: self.store.clone(),
                        index: self.index,
                    });
                    self.index += 1;
                    Some(obj)
                } else {
                    None
                }
            }
        }
    };
}

gen_container!(Segment, segs, PySegment, SegmentList, SegmentIter);
gen_container!(Path, paths, PyPath, PathList, PathIter);
gen_container!(Link, links, PyLink, LinkList, LinkIter);

/// A segment in a GFA graph.
///
/// Segments are the nodes in the GFA graph. They have a unique ID and an associated
/// nucleotide sequence.
#[pyclass(frozen)]
#[pyo3(name = "Segment", module = "flatgfa")]
struct PySegment(EntityRef);

impl From<EntityRef> for PySegment {
    fn from(entity: EntityRef) -> Self {
        Self(entity)
    }
}

#[pymethods]
impl PySegment {
    /// Get the nucleotide sequence for the segment as a byte string.
    ///
    /// This copies the underlying sequence data to contruct the Python bytes object,
    /// so it is slow to use for large sequences.
    fn sequence<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let gfa = self.0.store.view();
        let seg = &gfa.segs[self.0.id()];
        let seq = gfa.get_seq(seg);
        PyBytes::new_bound(py, seq)
    }

    /// The segment's name as declared in the GFA file, an `int`.
    #[getter]
    fn name(&self) -> usize {
        let seg = self.0.store.view().segs[self.0.id()];
        seg.name
    }

    /// The unique identifier for the segment, an `int`.
    #[getter]
    fn id(&self) -> u32 {
        self.0.index
    }

    fn __repr__(&self) -> String {
        self.0.py_repr("Segment")
    }

    fn __str__(&self) -> String {
        let gfa = self.0.store.view();
        let seg = gfa.segs[self.0.id()];
        format!("{}", print::Display(&gfa, &seg))
    }

    fn __eq__(&self, other: &PySegment) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> isize {
        self.0.index as isize
    }

    fn __len__(&self) -> usize {
        let seg = self.0.store.view().segs[self.0.id()];
        seg.len()
    }
}

/// A sequence of :class:`Segment` objects.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct SegmentList(ListRef);

impl From<ListRef> for SegmentList {
    fn from(list: ListRef) -> Self {
        Self(list)
    }
}

#[pymethods]
impl SegmentList {
    /// Find a segment by its name (an `int`), or return `None` if not found.
    fn find(&self, name: usize) -> Option<PySegment> {
        let gfa = self.0.store.view();
        let id = gfa.find_seg(name)?;
        Some(PySegment(EntityRef {
            store: self.0.store.clone(),
            index: id.into(),
        }))
    }
}

/// A path in a GFA graph.
///
/// Paths are walks through the GFA graph, where each step is an oriented segment.
/// This class is an iterable over the segments in the path, so use something
/// like this::
///
///     for step in path:
///         print(step.segment.name)
///
/// to walk through a path's steps.
#[pyclass(frozen)]
#[pyo3(name = "Path", module = "flatgfa")]
struct PyPath(EntityRef);

impl From<EntityRef> for PyPath {
    fn from(entity: EntityRef) -> Self {
        Self(entity)
    }
}

#[pymethods]
impl PyPath {
    /// The unique identifier for the path, an `int`.
    #[getter]
    fn id(&self) -> u32 {
        self.0.index
    }

    /// Get the name of this path as declared in the GFA file, as a string.
    #[getter]
    fn name(&self) -> String {
        let gfa = self.0.store.view();
        let path = &gfa.paths[self.0.id()];
        let name = gfa.get_path_name(path);
        name.try_into().unwrap()
    }

    fn __repr__(&self) -> String {
        self.0.py_repr("Path")
    }

    fn __str__(&self) -> String {
        let gfa = self.0.store.view();
        let path = gfa.paths[self.0.id()];
        format!("{}", print::Display(&gfa, &path))
    }

    fn __eq__(&self, other: &PyPath) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> isize {
        self.0.index as isize
    }

    /// Get a list of steps in this path.
    ///
    /// For convenience, the path itself provides direct access to the step list. So, for
    /// example, ``path.steps[4]`` is the same as ``path[4]``.
    #[getter]
    fn steps(&self) -> StepList {
        let path = self.0.store.view().paths[self.0.id()];
        StepList(ListRef {
            store: self.0.store.clone(),
            start: path.steps.start.into(),
            end: path.steps.end.into(),
        })
    }

    fn __iter__(&self) -> StepIter {
        self.steps().__iter__()
    }

    fn __getitem__(&self, arg: SliceOrInt, py: Python) -> PyResult<PyObject> {
        self.steps().__getitem__(arg, py)
    }

    fn __len__(&self) -> usize {
        self.steps().__len__()
    }
}
#[derive(Clone)]
#[pyclass(frozen)]
#[pyo3(name = "ChunkEvent", module = "flatgfa")]
struct PyChunkEvent {
    chunk_event: ChunkEvent,
    store: Arc<Store>,
}

#[pymethods]
impl PyChunkEvent {
    #[getter]
    fn handle(&self) -> PyHandle {
        let handle: Handle = self.chunk_event.handle;
        PyHandle {
            store: (self.store.clone()),
            handle: (handle),
        }
    }

    #[getter]
    fn range(&self) -> (usize, usize) {
        match self.chunk_event.range {
            flatgfa::ops::gaf::ChunkRange::None => (1, 0),
            flatgfa::ops::gaf::ChunkRange::All => {
                let inner_gfa = self.store.view();
                let seg = inner_gfa.segs[self.chunk_event.handle.segment()];
                (0, seg.len() - 1)
            }
            flatgfa::ops::gaf::ChunkRange::Partial(start, end) => (start, end),
        }
    }
    fn sequence(&self) -> String {
        let inner_gfa = self.store.view();
        self.chunk_event.get_seq_string(&inner_gfa)
    }
}

#[pyclass]
struct PyGAFLineIter {
    chunks: Vec<PyChunkEvent>,
    index: usize,
}

#[pymethods]
impl PyGAFLineIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyChunkEvent> {
        if slf.index < slf.chunks.len() {
            let item = slf.chunks[slf.index].clone();
            slf.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[pyclass]
#[pyo3(name = "GAFLine", module = "flatgfa")]

struct PyGAFLine {
    store: Arc<Store>,
    chunks: Vec<PyChunkEvent>,
    gaf: String,
}
#[pymethods]
impl PyGAFLine {
    #[getter]
    fn name(&self) -> String {
        self.gaf.clone()
    }

    #[getter]
    fn chunks(&self) -> Vec<PyChunkEvent> {
        self.chunks.clone()
    }
    fn sequence(&self) -> String {
        let gfa = self.store.view();
        let mut res: String = "".to_string();
        for part in self.chunks.iter() {
            res = res + &part.chunk_event.get_seq_string(&gfa);
        }
        res
    }
    fn segment_ranges(&self) -> String {
        let gfa = self.store.view();
        let mut res: String = "".to_string();
        for part in self.chunks.iter() {
            res = res + "\n" + &part.chunk_event.get_seg(&gfa);
        }
        res
    }
    fn __iter__(slf: PyRef<Self>) -> PyGAFLineIter {
        PyGAFLineIter {
            chunks: slf.chunks.clone(),
            index: 0,
        }
    }
}
struct OwnedGAFParser {
    mmap: Arc<Mmap>,
}
impl OwnedGAFParser {
    fn view_gafparser(&self, position: usize) -> GAFParser<'_> {
        let sub_slice = &self.mmap[position..];
        flatgfa::ops::gaf::GAFParser::new(sub_slice)
    }
}

#[pyclass]
#[pyo3(name = "GAFParser", module = "flatgfa")]

struct PyGAFParser {
    gaf_buf: OwnedGAFParser,
    store: Arc<Store>,
    name_map: NameMap,
    pos: usize,
}
#[pymethods]
impl PyGAFParser {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<PyGAFLine> {
        let mut parser = self.gaf_buf.view_gafparser(self.pos);
        match parser.next() {
            Some(chunk) => {
                let position = parser.split.pos;
                self.pos += position;
                let res = Some(PyGAFLine {
                    store: self.store.clone(),
                    gaf: chunk.name.to_string(),
                    chunks: flatgfa::ops::gaf::PathChunker::new(
                        &self.store.view(),
                        &self.name_map,
                        chunk,
                    )
                    .map(|c| PyChunkEvent {
                        store: self.store.clone(),
                        chunk_event: c,
                    })
                    .collect(),
                });
                res
            }
            None => None,
        }
    }
}
/// A sequence of :class:`Path` objects.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct PathList(ListRef);

impl From<ListRef> for PathList {
    fn from(list: ListRef) -> Self {
        Self(list)
    }
}

#[pymethods]
impl PathList {
    /// Find a path by its name (a string), or return `None` if not found.
    fn find(&self, name: &str) -> Option<PyPath> {
        let gfa = self.0.store.view();
        let id = gfa.find_path(name.as_ref())?;
        Some(PyPath(EntityRef {
            store: self.0.store.clone(),
            index: id.into(),
        }))
    }
}

/// An oriented segment reference.
///
/// Because both paths and links connect *oriented* segments rather than the segments themselves,
/// they use this class to distinguish between (for example) ``5+`` and ``5-``.
#[pyclass(frozen)]
#[pyo3(name = "Handle", module = "flatgfa")]
struct PyHandle {
    store: Arc<Store>,
    handle: flatgfa::Handle,
}

#[pymethods]
impl PyHandle {
    /// The segment ID, an `int`.
    #[getter]
    fn seg_id(&self) -> u32 {
        self.handle.segment().into()
    }

    /// The orientation.
    #[getter]
    fn is_forward(&self) -> bool {
        self.handle.orient() == flatgfa::Orientation::Forward
    }

    /// The segment, as a :class:`Segment` object.
    #[getter]
    fn segment(&self) -> PySegment {
        PySegment(EntityRef {
            store: self.store.clone(),
            index: self.handle.segment().into(),
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "<Handle {}{}>",
            u32::from(self.handle.segment()),
            self.handle.orient()
        )
    }

    fn __str__(&self) -> String {
        let gfa = self.store.view();
        format!("{}", print::Display(&gfa, self.handle))
    }

    fn __eq__(&self, other: &PyHandle) -> bool {
        Arc::as_ptr(&self.store) == Arc::as_ptr(&other.store) && self.handle == other.handle
    }

    fn __hash__(&self) -> isize {
        (u32::from(self.handle.segment()) as isize) ^ ((self.handle.orient() as isize) << 16)
    }
}

/// Get the components of a Python slice object.
///
/// This wraps an underlying PyO3 utility but supports a `usize` length.
fn py_slice_indices(slice: &PySlice, len: u32) -> PyResult<pyo3::types::PySliceIndices> {
    // Depending on the size of a C `long`, this may or may not need a fallible
    // conversion. This is a workaround to avoid either errors or Clippy
    // warnings, depending on the platform.
    #[allow(clippy::unnecessary_fallible_conversions)]
    slice.indices(len.try_into().unwrap())
}

/// A list of :class:`Handle` objects, such as a sequence of path steps.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct StepList(ListRef);

#[pymethods]
impl StepList {
    fn __len__(&self) -> usize {
        self.0.len() as usize
    }

    fn __iter__(&self) -> StepIter {
        StepIter {
            store: self.0.store.clone(),
            index: self.0.start,
            end: self.0.end,
        }
    }

    fn __getitem__(&self, arg: SliceOrInt, py: Python) -> PyResult<PyObject> {
        match arg {
            SliceOrInt::Slice(slice) => {
                let indices = py_slice_indices(slice, self.0.len())?;
                if indices.step == 1 {
                    let list = self.0.slice(indices.start as u32, indices.stop as u32);
                    Ok(Self(list).into_py(py))
                } else {
                    Err(PyIndexError::new_err("only unit step is supported"))
                }
            }
            SliceOrInt::Int(int) => {
                let index = self.0.start + (int as u32);
                let handle = self.0.store.view().steps[Id::from(index)];
                Ok(PyHandle {
                    store: self.0.store.clone(),
                    handle,
                }
                .into_py(py))
            }
        }
    }
}

/// An iterator over the steps in a path.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct StepIter {
    store: Arc<Store>,
    index: u32,
    end: u32,
}

#[pymethods]
impl StepIter {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<PyHandle> {
        let gfa = self.store.view();
        if self.index < self.end {
            let handle = PyHandle {
                store: self.store.clone(),
                handle: gfa.steps[Id::from(self.index)],
            };
            self.index += 1;
            Some(handle)
        } else {
            None
        }
    }
}

/// A link in a GFA graph.
///
/// Links are directed edges between oriented segments. The source and sink are both
/// `Handle` objects, i.e., the "forward" or "backward" direction of a given segment.
#[pyclass(frozen)]
#[pyo3(name = "Link", module = "flatgfa")]
struct PyLink(EntityRef);

impl From<EntityRef> for PyLink {
    fn from(entity: EntityRef) -> Self {
        Self(entity)
    }
}

#[pymethods]
impl PyLink {
    /// The unique identifier for the link.
    #[getter]
    fn id(&self) -> u32 {
        self.0.index
    }

    fn __repr__(&self) -> String {
        self.0.py_repr("Link")
    }

    fn __str__(&self) -> String {
        let gfa = self.0.store.view();
        let link = gfa.links[self.0.id()];
        format!("{}", print::Display(&gfa, &link))
    }

    fn __eq__(&self, other: &PyLink) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> isize {
        self.0.index as isize
    }

    /// The edge's source handle.
    #[getter]
    fn from_(&self) -> PyHandle {
        PyHandle {
            store: self.0.store.clone(),
            handle: self.0.store.view().links[self.0.id()].from,
        }
    }

    /// The edge's sink handle.
    #[getter]
    fn to(&self) -> PyHandle {
        PyHandle {
            store: self.0.store.clone(),
            handle: self.0.store.view().links[self.0.id()].to,
        }
    }
}

/// A sequence of :class:`Link` objects.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct LinkList(ListRef);

impl From<ListRef> for LinkList {
    fn from(list: ListRef) -> Self {
        Self(list)
    }
}

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFlatGFA>()?;
    m.add_class::<PyGAFParser>()?;
    m.add_class::<PyGAFLine>()?;
    m.add_class::<PyChunkEvent>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_class::<PySegment>()?;
    m.add_class::<PyPath>()?;
    m.add_class::<PyHandle>()?;
    m.add_class::<PyLink>()?;
    m.add_class::<SegmentList>()?;
    m.add_class::<PathList>()?;
    m.add_class::<LinkList>()?;
    m.add_class::<StepList>()?;
    m.add_class::<PyChunkEvent>()?;
    Ok(())
}
