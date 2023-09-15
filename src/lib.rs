mod pdb;
mod project;
mod mdb;
mod ftree;
mod io;
mod db;
mod api;
use pyo3::prelude::*;
use pyo3::wrap_pymodule;

#[pymodule]
#[pyo3(name = "godata")]
fn godata_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(_project))?;
    Ok(())
}

#[pymodule]
#[pyo3(name = "project")]
fn _project(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<project::Project>()?;
    m.add_class::<project::ProjectManager>()?;
    m.add("GodataProjectError", _py.get_type::<project::GodataProjectError>())?;
    Ok(())
}

// Add some tests