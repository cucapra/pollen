use flatgfa::flatgfa::{FlatGFA, HeapGFAStore};
use flatgfa::pool::Id;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::sync::Arc;

#[pyfunction]
fn parse(filename: &str) -> PyFlatGFA {
    let file = flatgfa::file::map_file(filename);
    let store = flatgfa::parse::Parser::for_heap().parse_mem(file.as_ref());
    PyFlatGFA(Arc::new(InternalStore::Heap(Box::new(store))))
}

#[pyfunction]
fn load(filename: &str) -> PyFlatGFA {
    let mmap = flatgfa::file::map_file(filename);
    PyFlatGFA(Arc::new(InternalStore::File(mmap)))
}

enum InternalStore {
    Heap(Box<HeapGFAStore>),
    File(memmap::Mmap),
}

impl InternalStore {
    fn view(&self) -> FlatGFA {
        // TK It seems wasteful to check the type of store every time... and to construct
        // the view every time. It would be great if we could somehow construct the view
        // once up front and hand it out to the various ancillary objects, but they need
        // to be assured that the store will survive long enough.
        match self {
            InternalStore::Heap(ref store) => (**store).as_ref(),
            InternalStore::File(ref mmap) => flatgfa::file::view(mmap),
        }
    }
}

#[pyclass(frozen)]
#[pyo3(name = "FlatGFA")]
struct PyFlatGFA(Arc<InternalStore>);

#[pymethods]
impl PyFlatGFA {
    #[getter]
    fn segments(&self) -> SegmentList {
        SegmentList {
            store: self.0.clone(),
        }
    }
}

#[pyclass]
struct SegmentList {
    store: Arc<InternalStore>,
}

#[pymethods]
impl SegmentList {
    fn __getitem__(&self, idx: u32) -> PySegment {
        PySegment {
            store: self.store.clone(),
            id: idx,
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
struct SegmentIter {
    store: Arc<InternalStore>,
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
                id: self.idx,
            };
            self.idx += 1;
            Some(seg)
        } else {
            None
        }
    }
}

#[pyclass(frozen)]
#[pyo3(name = "Segment")]
struct PySegment {
    store: Arc<InternalStore>,
    #[pyo3(get)]
    id: u32,
}

#[pymethods]
impl PySegment {
    /// Get the nucleotide sequence for the segment as a byte string.
    ///
    /// This copies the underlying sequence data to contruct the Python bytes object,
    /// so it is slow to use for large sequences.
    fn sequence<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let view = self.store.view();
        let seg = &view.segs[Id::from(self.id)];
        let seq = view.get_seq(&seg);
        PyBytes::new_bound(py, seq)
    }

    #[getter]
    fn name(&self) -> usize {
        let view = self.store.view();
        let seg = view.segs[Id::from(self.id)];
        seg.name
    }

    fn __repr__(&self) -> String {
        format!("<Segment {}>", self.id)
    }
}

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    Ok(())
}
