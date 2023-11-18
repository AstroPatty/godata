use crate::fsystem::{FileSystem, is_empty};
use crate::locations::{create_project_dir, load_project_dir, load_collection_dir, delete_project_dir};
use crate::storage::{StorageEndpoint, LocalEndpoint, StorageManager};
use pyo3::prelude::*;
use pyo3::create_exception;
use std::collections::HashMap;
use std::path::PathBuf;



// Define the project exception
create_exception!(project, GodataProjectError, pyo3::exceptions::PyException);


#[pyclass]
pub struct Project {
    pub(crate) tree: FileSystem,
    name: String,
    collection: String,
    _endpoint: Box<dyn StorageEndpoint + Send>,
}

#[pymethods]
impl Project {
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        Ok(self.name.clone())
    }

    #[getter]
    fn get_collection(&self) -> PyResult<String> {
        Ok(self.collection.clone())
    }

    fn add_file(&mut self, name: String, real_path: String, project_path: String) -> PyResult<()> {
        self.tree.insert(name, real_path, &project_path)?;
        Ok(())
    }

    fn get_file(&self, project_path: String) -> PyResult<String> {
        let file = self.tree.get(&project_path)?;
        Ok(file)
    }

    fn list(&self, project_path: Option<String>) -> PyResult<HashMap<String, Vec<String>>> {
        let list = self.tree.list(project_path)?;
        Ok(list)
    }

    fn remove_file(&mut self, project_path: String) -> PyResult<()> {
        self.tree.remove(&project_path)?;
        Ok(())
    }

    fn exists(&self, project_path: String) -> PyResult<bool> {
        let exists = self.tree.exists(&project_path);
        Ok(exists)
    }
}

#[pyfunction]
pub fn get_project_manager() -> PyResult<ProjectManager> {
    let storage_manager = StorageManager::get_manager();
    Ok(ProjectManager {
        storage_manager,
    })
}


#[pyclass]
pub struct ProjectManager {
    storage_manager: StorageManager,
}

#[pymethods]
impl ProjectManager {
    #[pyo3(signature = (name, collection, force = false, storage_location = None)) ]
    pub fn create_project(&self, name: String, collection: String, force: bool, storage_location: Option<String>) -> PyResult<Project> {
        let project_dir = create_project_dir(&name, &collection, force)?;
        let tree = FileSystem::new(name.clone(), project_dir)?;
        let base_path = match storage_location {
            Some(path) => PathBuf::from(path),
            None => crate::locations::get_default_project_storage_dir(&name, &collection)?,
        };
        self.storage_manager.add(&name, &collection, "local", base_path.clone())?;
        let endpoint = LocalEndpoint::new(base_path);
        Ok(Project {
            tree,
            name: name, 
            collection: collection.to_string(),
            _endpoint: Box::new(endpoint),
        })
    }

    pub fn load_project(&self, name: String, collection: String) -> PyResult<Project> {

        let project_dir = load_project_dir(&name, &collection)?;
        let storage_dir = self.storage_manager.get(&name, &collection)?;
        let tree = FileSystem::load(name.clone(), project_dir)?;
        let endpoint = LocalEndpoint::new(storage_dir.1);

        Ok(Project {
            tree,
            name: name, 
            collection: collection.to_string(),
            _endpoint: Box::new(endpoint),
        })
    }

    #[pyo3(signature = (name, collection, force = false)) ]
    pub fn delete_project(&self, name: String, collection: String, force: bool) -> PyResult<()> {
        let project_dir = load_project_dir(&name, &collection)?;
        let storage_dir = self.storage_manager.get(&name, &collection);
        let project_is_empty = is_empty(&project_dir);
        let mut storage_is_empty = storage_dir.is_err();
        if storage_dir.is_ok() {
            let storage_dir = storage_dir.unwrap();
            let storage_path = storage_dir.1;
            let mut files_in_storage = std::fs::read_dir(storage_path)?;
            storage_is_empty = files_in_storage.next().is_none();
        }

        if (project_is_empty && storage_is_empty) || force {
            delete_project_dir(&name, &collection)?;
            let storage_dir = self.storage_manager.get(&name, &collection);
            if storage_dir.is_ok() {
                println!("DELETE STORAGE");
                _ = self.storage_manager.delete(&name, &collection)?;
            }
            return Ok(())
        } 
        
        Err(GodataProjectError::new_err("Project is not empty"))
    }
     
}





#[pyfunction]
pub fn get_project_names(collection: String, show_hidden: bool) -> PyResult<Vec<String>> {
    let collection_dir = load_collection_dir(&collection);
    if collection_dir.is_err() {
        return Err(GodataProjectError::new_err(format!("Collection `{}` does not exist", collection)));
    }
    let collection_dir = collection_dir.unwrap();

    let mut names = Vec::new();
    for entry in std::fs::read_dir(collection_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if !path.file_name().unwrap().to_str().unwrap().starts_with(".") || show_hidden {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                names.push(name);
            }
        }
    }
    Ok(names)
}

#[pyfunction]
pub fn get_collection_names(show_hidden: bool) -> PyResult<Vec<String>> {
    let main_dir = crate::locations::get_main_dir();
    let mut names = Vec::new();
    for entry in std::fs::read_dir(main_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if !path.file_name().unwrap().to_str().unwrap().starts_with(".") || show_hidden {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                names.push(name);
            }
        }
    }
    Ok(names)
}