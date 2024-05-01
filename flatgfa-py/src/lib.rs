use pyo3::prelude::*;

#[pyfunction]
fn add(left: usize, right: usize) -> usize {
    println!("sup");
    left + right
}

#[pymodule]
fn flatgfa(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    Ok(())
}
