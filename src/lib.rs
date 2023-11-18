mod project;
mod fsystem;
mod storage;
mod locations;
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
    m.add("GodataProjectError", _py.get_type::<project::GodataProjectError>())?;
    m.add_function(wrap_pyfunction!(project::get_project_names, m)?)?;
    m.add_function(wrap_pyfunction!(project::get_collection_names, m)?)?;
    m.add_function(wrap_pyfunction!(project::get_project_manager, m)?)?;
    Ok(())
}

// Add some tests