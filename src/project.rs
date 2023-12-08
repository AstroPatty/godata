use crate::fsystem::{FileSystem, is_empty};
use crate::locations::{create_project_dir, load_project_dir, load_collection_dir, delete_project_dir};
use crate::storage::{StorageEndpoint, LocalEndpoint, StorageManager};
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Result;
use std::sync::{Arc, Mutex};


pub struct Project {
    pub(crate) tree: FileSystem,
    name: String,
    collection: String,
    _endpoint: Box<dyn StorageEndpoint + Send>,
}

impl Project {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_collection(&self) -> &str {
        &self.collection
    }

    pub(crate) fn add_file(&mut self, name: String, real_path: String, project_path: String) -> Result<()> {
        self.tree.insert(name, real_path, &project_path)?;
        Ok(())
    }

    pub(crate) fn add_folder(&mut self, real_path: String, project_path: String, recursive: bool) -> Result<()> {
        let mut folders: Vec<PathBuf> = Vec::new();
        let files = std::fs::read_dir(&real_path)?
                                    .filter(|x| x.is_ok())
                                    .filter_map(|x| {
                                        let path = x.unwrap().path();
                                        if path.is_file() {
                                            Some(path)
                                        } else {
                                            if recursive {
                                                folders.push(path);
                                            }
                                            None
                                        }
                                    });
        
        self.tree.insert_many(files, &project_path)?;
        if recursive {
            for folder in folders {
                let folder_name = folder.file_name().unwrap().to_str().unwrap().to_string();
                let folder_path = folder.to_str().unwrap().to_string();
                let folder_project_path = format!("{}/{}", project_path, folder_name);
                self.add_folder(folder_path, folder_project_path, recursive)?;
            }
        }


        Ok(())
    }

    pub(crate) fn get_file(&self, project_path: String) -> Result<String> {
        let file = self.tree.get(&project_path)?;
        Ok(file)
    }

    pub(crate) fn list(&self, project_path: Option<String>) -> Result<HashMap<String, Vec<String>>> {
        let list = self.tree.list(project_path)?;
        Ok(list)
    }

    pub(crate) fn remove_file(&mut self, project_path: String) -> Result<()> {
        self.tree.remove(&project_path)?;
        Ok(())
    }

    pub(crate) fn exists(&self, project_path: String) -> Result<bool> {
        let exists = self.tree.exists(&project_path);
        Ok(exists)
    }

    pub(crate) fn generate_path(&self, project_path: String) -> Result<String> {
        let path = self._endpoint.generate_path(&project_path)?;
        Ok(path.to_str().unwrap().to_owned())
    }
}

pub fn get_project_manager() -> ProjectManager {
    let storage_manager = StorageManager::get_manager();
    ProjectManager {
        storage_manager,
        projects: HashMap::new()
    }
}


pub struct ProjectManager {
    storage_manager: StorageManager,
    projects: HashMap<String, Arc<Mutex<Project>>>
}

impl ProjectManager {
    pub fn create_project(&mut self, name: String, collection: String, force: bool, storage_location: Option<String>) -> Result<Arc<Mutex<Project>>> {
        let key = format!("{}/{}", name, collection);
        let project_dir = create_project_dir(&name, &collection, force)?;
        let tree = FileSystem::new(name.clone(), project_dir)?;
        let base_path = match storage_location {
            Some(path) => PathBuf::from(path),
            None => crate::locations::get_default_project_storage_dir(&name, &collection)?,
        };
        self.storage_manager.add(&name, &collection, "local", base_path.clone())?;
        let endpoint = LocalEndpoint::new(base_path);
        let p = Project {
            tree,
            name: name, 
            collection: collection.to_string(),
            _endpoint: Box::new(endpoint),
        };
        let project = Arc::new(Mutex::new(p));
        self.projects.insert(key, project.clone());
        return Ok(project);
    }

    pub fn load_project(&mut self, name: &str, collection: &str) -> Result<Arc<Mutex<Project>>> {
        let key = format!("{}/{}", name, collection);
        if self.projects.contains_key(&key) {
            return Ok(self.projects.get(&key).unwrap().clone());
        }
        let project_dir = load_project_dir(name, collection)?;
        let storage_dir = self.storage_manager.get(&name, &collection)?;
        let tree = FileSystem::load(name, project_dir)?;
        let endpoint = LocalEndpoint::new(storage_dir.1);

        let project = Project {
            tree,
            name: name.to_string(), 
            collection: collection.to_string(),
            _endpoint: Box::new(endpoint),
        };
        let project = Arc::new(Mutex::new(project));
        self.projects.insert(key, project.clone());
        return Ok(project);
    }

    pub fn delete_project(&self, name: String, collection: String, force: bool) -> Result<()> {
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
                 _ = self.storage_manager.delete(&name, &collection)?;
            }
            return Ok(())
        } 
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Project is not empty"));
    }

    pub fn get_project_names(&self, collection: String, show_hidden: bool) -> Result<Vec<String>> {
        let collection_dir = load_collection_dir(&collection);
        if collection_dir.is_err() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Collection `{}` does not exist", collection)));
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
     
}



pub fn get_collection_names(show_hidden: bool) -> Result<Vec<String>> {
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