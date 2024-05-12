use flatgfa::flatgfa::{FlatGFA, HeapGFAStore, Segment};
use flatgfa::pool::Id;
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
}

/// A sequence container for `Segment`s.
#[pyclass]
#[pyo3(module = "flatgfa")]
struct SegmentList {
    store: Arc<Store>,
}

#[pymethods]
impl SegmentList {
    fn __getitem__(&self, idx: u32) -> PySegment {
        PySegment {
            store: self.store.clone(),
            id: Id::from(idx),
        }
    }

    fn __iter__(&self) -> SegmentIter {
        SegmentIter {
            store: self.store.clone(),
            idx: 0,
        }
    }

    fn __len__(&self) -> usize {
        self.store.view().segs.len()
    }
}

#[pyclass]
#[pyo3(module = "flatgfa")]
struct SegmentIter {
    store: Arc<Store>,
    idx: u32,
}

#[pymethods]
impl SegmentIter {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self) -> Option<PySegment> {
        let view = self.store.view();
        if self.idx < view.segs.len() as u32 {
            let seg = PySegment {
                store: self.store.clone(),
                id: Id::from(self.idx),
            };
            self.idx += 1;
            Some(seg)
        } else {
            None
        }
    }
}

/// A segment in a GFA graph.
///
/// Segments are the nodes in the GFA graph. They have a unique ID and an associated
/// nucleotide sequence.
#[pyclass(frozen)]
#[pyo3(name = "Segment", module = "flatgfa")]
struct PySegment {
    store: Arc<Store>,
    id: Id<Segment>,
}

#[pymethods]
impl PySegment {
    /// Get the nucleotide sequence for the segment as a byte string.
    ///
    /// This copies the underlying sequence data to contruct the Python bytes object,
    /// so it is slow to use for large sequences.
    fn sequence<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let view = self.store.view();
        let seg = &view.segs[self.id];
        let seq = view.get_seq(&seg);
        PyBytes::new_bound(py, seq)
    }

    /// The segment's name as declared in the GFA file.
    #[getter]
    fn name(&self) -> usize {
        let view = self.store.view();
        let seg = view.segs[self.id];
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

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFlatGFA>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_class::<PySegment>()?;
    Ok(())
}
