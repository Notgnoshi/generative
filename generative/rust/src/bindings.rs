use pyo3::prelude::*;

#[pymodule]
fn rust(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    #[pyo3(name = "add")]
    fn add_py(_py: Python, a: i64, b: i64) -> PyResult<i64> {
        let out = add(a, b);
        Ok(out)
    }

    Ok(())
}

fn add(a: i64, b: i64) -> i64 {
    a + b
}
