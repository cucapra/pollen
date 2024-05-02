use flatgfa::flatgfa::{HeapStore, Segment};
use pyo3::prelude::*;

#[pyfunction]
fn parse(filename: &str) -> PyFlatGFA {
    let file = flatgfa::file::map_file(filename);
    let store = flatgfa::parse::Parser::for_heap().parse_mem(file.as_ref());
    PyFlatGFA(store)
}

#[pyclass]
#[pyo3(name = "FlatGFA")]
struct PyFlatGFA(HeapStore);

#[pymethods]
impl PyFlatGFA {
    fn get_a_seg(&self) -> PySegment {
        PySegment(self.0.segs[0])
    }
}

#[pyclass]
#[pyo3(name = "Segment")]
struct PySegment(Segment);

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
