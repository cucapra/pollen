use flatgfa::flatgfa::{GFABuilder, HeapStore, Segment};
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
    fn get_a_seg(self_: Py<Self>) -> PySegment {
        let s = self_.get().0.segs[0];
        PySegment { gfa: self_, seg: s }
    }
}

#[pyclass(frozen)]
#[pyo3(name = "Segment")]
struct PySegment {
    gfa: Py<PyFlatGFA>,
    seg: Segment,
}

#[pymethods]
impl PySegment {
    fn get_seq<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let view = self.gfa.get().0.view();
        let seq = view.get_seq(&self.seg);
        PyBytes::new_bound(py, seq) // TK Can we avoid this copy?
    }
}

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
