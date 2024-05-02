use flatgfa::flatgfa::HeapStore;
use pyo3::prelude::*;

#[pyfunction]
fn parse(filename: &str) -> Graph {
    let file = flatgfa::file::map_file(filename);
    let store = flatgfa::parse::Parser::for_heap().parse_mem(file.as_ref());
    Graph(store)
}

#[pyclass]
struct Graph(HeapStore);

#[pymodule]
#[pyo3(name = "flatgfa")]
fn pymod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
