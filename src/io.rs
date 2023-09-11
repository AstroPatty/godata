use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use std::path::PathBuf;
use std::fs;
use crate::mdb::get_dirs;

pub(crate) fn store(object: &PyAny, output_function:  &PyAny, path: &str) -> PyResult<()> {
    // Accepts a  function that stores an object to a file that is passed to it.
    if !output_function.is_callable() {
        return Err(PyErr::new::<PyTypeError, _>("output_function must be callable"));
    }
    Python::with_gil(|_| -> PyResult<()> {
        output_function.call((object, path), None)?;
        Ok(())
    })?;
    Ok(())
}

pub(crate) fn remove_if_internal(path: &PathBuf) {
    // Remove a file or folder if it is stored in the internal storage.
    let dirs = get_dirs();
    let data_dir = dirs.get("data_dir").unwrap();
    if path.starts_with(data_dir) {
        if path.is_file() {
            fs::remove_file(&path).unwrap();
            if path.parent().unwrap().read_dir().unwrap().count() == 0 {
                fs::remove_dir_all(path.parent().unwrap()).unwrap();
            }
        }
        else {
            fs::remove_dir_all(path).unwrap();
        }
    }

}