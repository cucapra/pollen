use flatgfa::pool::{Id, Span};
use flatgfa::{self, file, print, FlatGFA, HeapGFAStore};
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PySlice};
use std::io::Write;
use std::sync::Arc;

/// Storage for a FlatGFA.
///
/// This may be either an in-memory data structure or a memory-mapped file. It exposes a
/// uniform interface to the FlatGFA data via `view`.
enum Store {
    Heap(Box<HeapGFAStore>),
    File(memmap::Mmap),
}

impl Store {
    /// Parse a text GFA file.
    fn parse_file(filename: &str) -> Self {
        let file = file::map_file(filename);
        Self::parse_gfa(file.as_ref())
    }

    /// Parse a GFA graph from a byte buffer.
    fn parse_gfa(data: &[u8]) -> Self {
        let store = flatgfa::parse::Parser::for_heap().parse_mem(data);
        Self::Heap(Box::new(store))
    }

    /// Load a FlatGFA binary file.
    fn load(filename: &str) -> Self {
        let mmap = file::map_file(filename);
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
        SegmentList {
            store: self.0.clone(),
            start: 0,
            end: self.0.view().segs.len() as u32,
        }
    }

    /// The paths in the graph, as a :class:`PathList`.
    #[getter]
    fn paths(&self) -> PathList {
        PathList {
            store: self.0.clone(),
            start: 0,
            end: self.0.view().paths.len() as u32,
        }
    }

    /// The links (edges) in the graph, as a :class:`LinkList`.
    #[getter]
    fn links(&self) -> LinkList {
        LinkList {
            store: self.0.clone(),
            start: 0,
            end: self.0.view().links.len() as u32,
        }
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
        let mut mmap = file::map_new_file(filename, file::size(&gfa) as u64);
        file::dump(&gfa, &mut mmap);
        mmap.flush()?;
        Ok(())
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
                match arg {
                    SliceOrInt::Slice(slice) => {
                        let indices = slice.indices(self.__len__() as i64)?;
                        if indices.step == 1 {
                            assert!(indices.start >= 0);
                            assert!(indices.stop <= self.__len__() as isize);
                            Ok(Self {
                                store: self.store.clone(),
                                start: self.start + indices.start as u32,
                                end: self.start + indices.stop as u32,
                            }
                            .into_py(py))
                        } else {
                            Err(PyIndexError::new_err("only unit step is supported"))
                        }
                    }
                    SliceOrInt::Int(int) => {
                        if int >= 0 && int < (self.end as isize) {
                            let global_idx = (int as u32) + self.start;
                            Ok($pytype {
                                store: self.store.clone(),
                                id: Id::from(global_idx),
                            }
                            .into_py(py))
                        } else {
                            Err(PyIndexError::new_err("index out of range"))
                        }
                    }
                }
            }

            fn __iter__(&self) -> $iter {
                $iter {
                    store: self.store.clone(),
                    idx: self.start,
                    end: self.end,
                }
            }

            fn __len__(&self) -> usize {
                (self.end - self.start) as usize
            }
        }

        #[pyclass]
        #[pyo3(module = "flatgfa")]
        struct $iter {
            store: Arc<Store>,
            idx: u32,
            end: u32,
        }

        #[pymethods]
        impl $iter {
            fn __iter__(self_: Py<Self>) -> Py<Self> {
                self_
            }

            fn __next__(&mut self) -> Option<$pytype> {
                if self.idx < self.end as u32 {
                    let obj = $pytype {
                        store: self.store.clone(),
                        id: Id::from(self.idx),
                    };
                    self.idx += 1;
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
struct PySegment {
    store: Arc<Store>,
    id: Id<flatgfa::Segment>,
}

#[pymethods]
impl PySegment {
    /// Get the nucleotide sequence for the segment as a byte string.
    ///
    /// This copies the underlying sequence data to contruct the Python bytes object,
    /// so it is slow to use for large sequences.
    fn sequence<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let gfa = self.store.view();
        let seg = &gfa.segs[self.id];
        let seq = gfa.get_seq(seg);
        PyBytes::new_bound(py, seq)
    }

    /// The segment's name as declared in the GFA file, an `int`.
    #[getter]
    fn name(&self) -> usize {
        let seg = self.store.view().segs[self.id];
        seg.name
    }

    /// The unique identifier for the segment, an `int`.
    #[getter]
    fn id(&self) -> u32 {
        self.id.into()
    }

    fn __repr__(&self) -> String {
        format!("<Segment {}>", u32::from(self.id))
    }

    fn __str__(&self) -> String {
        let gfa = self.store.view();
        let seg = gfa.segs[self.id];
        format!("{}", print::Display(&gfa, &seg))
    }

    fn __eq__(&self, other: &PySegment) -> bool {
        Arc::as_ptr(&self.store) == Arc::as_ptr(&other.store) && self.id == other.id
    }

    fn __hash__(&self) -> isize {
        u32::from(self.id) as isize
    }

    fn __len__(&self) -> usize {
        let gfa = self.store.view();
        let seg = gfa.segs[self.id];
        seg.len()
    }
}

#[pymethods]
impl SegmentList {
    /// Find a segment by its name (an `int`), or return `None` if not found.
    fn find(&self, name: usize) -> Option<PySegment> {
        let gfa = self.store.view();
        let id = gfa.find_seg(name)?;
        Some(PySegment {
            store: self.store.clone(),
            id,
        })
    }
}

/// A sequence of :class:`Segment` objects.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct SegmentList {
    store: Arc<Store>,
    start: u32,
    end: u32,
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
struct PyPath {
    store: Arc<Store>,
    id: Id<flatgfa::Path>,
}

#[pymethods]
impl PyPath {
    /// The unique identifier for the path, an `int`.
    #[getter]
    fn id(&self) -> u32 {
        self.id.into()
    }

    /// Get the name of this path as declared in the GFA file, as a string.
    #[getter]
    fn name(&self) -> String {
        let gfa = self.store.view();
        let path = &gfa.paths[self.id];
        let name = gfa.get_path_name(path);
        name.try_into().unwrap()
    }

    fn __repr__(&self) -> String {
        format!("<Path {}>", u32::from(self.id))
    }

    fn __str__(&self) -> String {
        let gfa = self.store.view();
        let path = gfa.paths[self.id];
        format!("{}", print::Display(&gfa, &path))
    }

    fn __eq__(&self, other: &PyPath) -> bool {
        Arc::as_ptr(&self.store) == Arc::as_ptr(&other.store) && self.id == other.id
    }

    fn __hash__(&self) -> isize {
        u32::from(self.id) as isize
    }

    fn __iter__(&self) -> StepIter {
        let path = self.store.view().paths[self.id];
        StepIter {
            store: self.store.clone(),
            span: path.steps,
            cur: path.steps.start,
        }
    }

    fn __getitem__(&self, idx: usize) -> PyHandle {
        let gfa = self.store.view();
        let path = gfa.paths[self.id];
        let handle = gfa.steps[path.steps][idx];
        PyHandle {
            store: self.store.clone(),
            handle,
        }
    }

    fn __len__(&self) -> usize {
        let path = self.store.view().paths[self.id];
        path.steps.len()
    }
}

/// A sequence of :class:`Path` objects.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct PathList {
    store: Arc<Store>,
    start: u32,
    end: u32,
}

#[pymethods]
impl PathList {
    /// Find a path by its name (a string), or return `None` if not found.
    fn find(&self, name: &str) -> Option<PyPath> {
        let gfa = self.store.view();
        let id = gfa.find_path(name.as_ref())?;
        Some(PyPath {
            store: self.store.clone(),
            id,
        })
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
        PySegment {
            store: self.store.clone(),
            id: self.handle.segment(),
        }
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

/// An iterator over the steps in a path.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct StepIter {
    store: Arc<Store>,
    span: Span<flatgfa::Handle>,
    cur: Id<flatgfa::Handle>,
}

#[pymethods]
impl StepIter {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<PyHandle> {
        let gfa = self.store.view();
        if self.span.contains(self.cur) {
            let handle = PyHandle {
                store: self.store.clone(),
                handle: gfa.steps[self.cur],
            };
            self.cur = (u32::from(self.cur) + 1).into();
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
struct PyLink {
    store: Arc<Store>,
    id: Id<flatgfa::Link>,
}

#[pymethods]
impl PyLink {
    /// The unique identifier for the link.
    #[getter]
    fn id(&self) -> u32 {
        self.id.into()
    }

    fn __repr__(&self) -> String {
        format!("<Link {}>", u32::from(self.id))
    }

    fn __str__(&self) -> String {
        let gfa = self.store.view();
        let link = gfa.links[self.id];
        format!("{}", print::Display(&gfa, &link))
    }

    fn __eq__(&self, other: &PyLink) -> bool {
        Arc::as_ptr(&self.store) == Arc::as_ptr(&other.store) && self.id == other.id
    }

    fn __hash__(&self) -> isize {
        u32::from(self.id) as isize
    }

    /// The edge's source handle.
    #[getter]
    fn from_(&self) -> PyHandle {
        PyHandle {
            store: self.store.clone(),
            handle: self.store.view().links[self.id].from,
        }
    }

    /// The edge's sink handle.
    #[getter]
    fn to(&self) -> PyHandle {
        PyHandle {
            store: self.store.clone(),
            handle: self.store.view().links[self.id].to,
        }
    }
}

/// A sequence of :class:`Link` objects.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct LinkList {
    store: Arc<Store>,
    start: u32,
    end: u32,
}

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFlatGFA>()?;
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
    Ok(())
}
