use std::sync::{Arc, Mutex};
use crate::project::ProjectManager;
use std::io::Result;
use std::collections::HashMap;

pub(crate) fn list(project_manager: Arc<Mutex<ProjectManager>>, project_path: Option<String>, project_name: String, collection_name: String) -> Result<HashMap<String, Vec<String>>> {
    let mut mgr = project_manager.lock().unwrap();
    let project = mgr.load_project(project_name, collection_name)?;
    let list = project.lock().unwrap().list(project_path)?;
    return Ok(list);
}

pub(crate) fn get_file(project_manager: Arc<Mutex<ProjectManager>>, project_path: String, project_name: String, collection_name: String) -> Result<String> {
    let mut mgr = project_manager.lock().unwrap();
    let project = mgr.load_project(project_name, collection_name)?;
    let file = project.lock().unwrap().get_file(project_path);
    file
}

pub(crate) fn add_file(project_manager: Arc<Mutex<ProjectManager>>, file_name: String, file_path: String, project_path: String, project_name: String, collection_name: String) -> Result<()> {
    let mut mgr = project_manager.lock().unwrap();
    let project = mgr.load_project(project_name, collection_name)?;
    project.lock().unwrap().add_file(file_name, file_path, project_path)?;
    Ok(())
}

pub(crate) fn add_folder(project_manager: Arc<Mutex<ProjectManager>>, folder_path: String, project_path: String, recursive: bool, project_name: String, collection_name: String) -> Result<()> {
    let mut mgr = project_manager.lock().unwrap();
    let project = mgr.load_project(project_name, collection_name)?;
    project.lock().unwrap().add_folder(folder_path, project_path, recursive)?;
    Ok(())
}

pub(crate) fn remove_file(project_manager: Arc<Mutex<ProjectManager>>, project_path: String, project_name: String, collection_name: String) -> Result<()> {
    let mut mgr = project_manager.lock().unwrap();
    let project = mgr.load_project(project_name, collection_name)?;
    project.lock().unwrap().remove_file(project_path)?;
    Ok(())
}