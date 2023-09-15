use crate::project::{ProjectManager, Project};

#[allow(dead_code)]
pub fn create_project(name: &str, collection: Option<&str>) -> Project {
    let mut manager = ProjectManager::new();
    let p = manager.create_project(name, collection).unwrap();
    p
}

#[allow(dead_code)]
pub fn delete_project(name: &str, collection: Option<&str>) {
    let mut manager = ProjectManager::new();
    manager.remove_project(name, collection).unwrap();
}

#[allow(dead_code)]
pub fn load_project(name: &str, collection: Option<&str>) -> Project {
    let manager = ProjectManager::new();
    let p = manager.load_project(name, collection).unwrap();
    p
}

#[allow(dead_code)]
pub fn list_projects(show_hidden: bool, collection: Option<&str>) -> Vec<String> {
    let manager = ProjectManager::new();
    let projects = manager.list_projects(show_hidden, collection).unwrap();
    projects
}

#[allow(dead_code)]
pub fn list_collections(show_hidden: bool) -> Vec<String> {
    let manager = ProjectManager::new();
    let collections = manager.list_collections(show_hidden).unwrap();
    collections
}



