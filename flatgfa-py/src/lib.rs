use flatgfa::pool::{Id, Span};
use flatgfa::{self, FlatGFA, HeapGFAStore};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
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
    fn parse(filename: &str) -> Self {
        let file = flatgfa::file::map_file(filename);
        let store = flatgfa::parse::Parser::for_heap().parse_mem(file.as_ref());
        Self::Heap(Box::new(store))
    }

    /// Load a FlatGFA binary file.
    fn load(filename: &str) -> Self {
        let mmap = flatgfa::file::map_file(filename);
        Self::File(mmap)
    }

    /// Get the FlatGFA stored here.
    fn view(&self) -> FlatGFA {
        // TK It seems wasteful to check the type of store every time... and to construct
        // the view every time. It's probably possible to fix this with a self-reference,
        // e.g., with the `owning_ref` crate.
        match self {
            Store::Heap(ref store) => (**store).as_ref(),
            Store::File(ref mmap) => flatgfa::file::view(mmap),
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
    PyFlatGFA(Arc::new(Store::parse(filename)))
}

/// Load a binary FlatGFA file.
#[pyfunction]
fn load(filename: &str) -> PyFlatGFA {
    PyFlatGFA(Arc::new(Store::load(filename)))
}

#[pymethods]
impl PyFlatGFA {
    /// The segments (nodes) in the graph.
    #[getter]
    fn segments(&self) -> SegmentList {
        SegmentList {
            store: self.0.clone(),
        }
    }

    /// The paths in the graph.
    #[getter]
    fn paths(&self) -> PathList {
        PathList {
            store: self.0.clone(),
        }
    }

    /// The links in the graph.
    #[getter]
    fn links(&self) -> LinkList {
        LinkList {
            store: self.0.clone(),
        }
    }
}

/// Generate the Python types for an iterable container of GFA objects.
macro_rules! gen_container {
    ($type: ident, $field: ident, $pytype: ident, $list: ident, $iter: ident) => {
        /// A sequence container for `$type`s.
        #[pyclass]
        #[pyo3(module = "flatgfa")]
        struct $list {
            store: Arc<Store>,
        }

        #[pymethods]
        impl $list {
            fn __getitem__(&self, idx: u32) -> $pytype {
                $pytype {
                    store: self.store.clone(),
                    id: Id::from(idx),
                }
            }

            fn __iter__(&self) -> $iter {
                $iter {
                    store: self.store.clone(),
                    idx: 0,
                }
            }

            fn __len__(&self) -> usize {
                self.store.view().$field.len()
            }
        }

        #[pyclass]
        #[pyo3(module = "flatgfa")]
        struct $iter {
            store: Arc<Store>,
            idx: u32,
        }

        #[pymethods]
        impl $iter {
            fn __iter__(self_: Py<Self>) -> Py<Self> {
                self_
            }

            fn __next__(&mut self) -> Option<$pytype> {
                let gfa = self.store.view();
                if self.idx < gfa.$field.len() as u32 {
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

    /// The segment's name as declared in the GFA file.
    #[getter]
    fn name(&self) -> usize {
        let seg = self.store.view().segs[self.id];
        seg.name
    }

    /// The unique identifier for the segment.
    #[getter]
    fn id(&self) -> u32 {
        self.id.into()
    }

    fn __repr__(&self) -> String {
        format!("<Segment {}>", u32::from(self.id))
    }
}

/// A path in a GFA graph.
///
/// Paths are walks through the GFA graph, where each step is an oriented segment.
#[pyclass(frozen)]
#[pyo3(name = "Segment", module = "flatgfa")]
struct PyPath {
    store: Arc<Store>,
    id: Id<flatgfa::Path>,
}

#[pymethods]
impl PyPath {
    /// The unique identifier for the path.
    #[getter]
    fn id(&self) -> u32 {
        self.id.into()
    }

    /// Get the name of this path as declared in the GFA file.
    #[getter]
    fn name<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let gfa = self.store.view();
        let path = &gfa.paths[self.id];
        let name = gfa.get_path_name(path);
        PyBytes::new_bound(py, name)
    }

    fn __repr__(&self) -> String {
        format!("<Path {}>", u32::from(self.id))
    }

    fn __iter__(&self) -> StepIter {
        let path = self.store.view().paths[self.id];
        StepIter {
            store: self.store.clone(),
            span: path.steps,
            cur: path.steps.start,
        }
    }

    fn __len__(&self) -> usize {
        let path = self.store.view().paths[self.id];
        path.steps.len()
    }
}

/// An oriented segment reference.
#[pyclass(frozen)]
#[pyo3(name = "Handle", module = "flatgfa")]
struct PyHandle {
    store: Arc<Store>,
    handle: flatgfa::Handle,
}

#[pymethods]
impl PyHandle {
    /// The segment ID.
    #[getter]
    fn seg_id(&self) -> u32 {
        self.handle.segment().into()
    }

    /// The orientation.
    #[getter]
    fn is_forward(&self) -> bool {
        self.handle.orient() == flatgfa::Orientation::Forward
    }

    /// The segment.
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
/// `Handle`s, i.e., the "forward" or "backward" direction of a given segment.
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

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFlatGFA>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_class::<PySegment>()?;
    m.add_class::<PyPath>()?;
    m.add_class::<PyHandle>()?;
    m.add_class::<PyLink>()?;
    Ok(())
}
