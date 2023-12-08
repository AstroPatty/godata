use crate::project::ProjectManager;
use crate::project::get_collection_names;
use std::sync::{Arc, Mutex};

pub(crate) fn list_collections() -> Vec<String> {
    let collections = get_collection_names(false);
    collections.unwrap()
}

pub(crate) fn list_projects(project_manager: Arc<Mutex<ProjectManager>>, collection: String) -> Vec<String> {
    let projects = project_manager.lock().unwrap().get_project_names(collection, false);
    println!("projects: {:?}", projects);
    projects.unwrap()
}