use flatgfa::flatgfa::{GFABuilder, HeapStore};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyfunction]
fn parse(filename: &str) -> PyFlatGFA {
    let file = flatgfa::file::map_file(filename);
    let store = flatgfa::parse::Parser::for_heap().parse_mem(file.as_ref());
    PyFlatGFA(store)
}

#[pyclass(frozen)]
#[pyo3(name = "FlatGFA")]
struct PyFlatGFA(HeapStore);

#[pymethods]
impl PyFlatGFA {
    #[getter]
    fn segments(self_: Py<Self>) -> SegmentIter {
        SegmentIter { gfa: self_, idx: 0 }
    }
}

#[pyclass]
struct SegmentIter {
    gfa: Py<PyFlatGFA>,
    idx: u32,
}

#[pymethods]
impl SegmentIter {
    fn __iter__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__<'py>(self_: Bound<'py, Self>) -> Option<PySegment> {
        let mut s = self_.borrow_mut();
        let view = s.gfa.get().0.view();
        if s.idx < view.segs.len() as u32 {
            let seg = PySegment {
                gfa: s.gfa.clone(),
                seg: s.idx,
            };
            s.idx += 1;
            Some(seg)
        } else {
            None
        }
    }
}

#[pyclass(frozen)]
#[pyo3(name = "Segment")]
struct PySegment {
    gfa: Py<PyFlatGFA>,
    seg: u32,
}

#[pymethods]
impl PySegment {
    fn sequence<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let view = self.gfa.get().0.view();
        let seg = view.segs[self.seg as usize];
        let seq = view.get_seq(&seg);
        PyBytes::new_bound(py, seq) // TK Can we avoid this copy?
    }

    #[getter]
    fn name<'py>(&self) -> usize {
        let view = self.gfa.get().0.view();
        let seg = view.segs[self.seg as usize];
        seg.name
    }
}

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
