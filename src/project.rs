use crate::fsystem::{is_empty, FileSystem};
use crate::locations::{
    create_project_dir, delete_project_dir, load_collection_dir, load_project_dir,
};
use crate::storage::{LocalEndpoint, StorageEndpoint, StorageManager};
use std::collections::HashMap;
use std::io::Result;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Project {
    pub(crate) tree: FileSystem,
    _name: String,
    _collection: String,
    _endpoint: Box<dyn StorageEndpoint + Send>,
}

impl Project {
    pub(crate) fn add_file(
        &mut self,
        project_path: &str,
        real_path: PathBuf,
        metadata: HashMap<String, String>,
        overwrite: bool,
    ) -> Result<Option<Vec<String>>> {
        let relpath = self._endpoint.get_relative_path(&real_path);
        let previous_entry = self
            .tree
            .insert(project_path, relpath, metadata, overwrite)?;
        if previous_entry.is_none() {
            return Ok(None);
        }
        let previous_entries = previous_entry.unwrap();
        if previous_entries.is_empty() {
            return Ok(None);
        }
        let output: Vec<String> = previous_entries
            .into_iter()
            .map(|x| self._endpoint.resolve(&x.real_path))
            .filter(|x| self._endpoint.is_internal(x))
            .map(|x| x.to_str().unwrap().to_string())
            .collect();

        Ok(Some(output))
    }

    pub(crate) fn duplicate_tree(&mut self, output_path: PathBuf) -> Result<()> {
        let export = self.tree.export()?;
        let db = sled::open(output_path)?;
        db.import(export);
        Ok(())
    }

    pub(crate) fn add_folder(
        &mut self,
        project_path: &str,
        real_path: PathBuf,
        recursive: bool,
    ) -> Result<()> {
        let mut folders: Vec<PathBuf> = Vec::new();
        let files = std::fs::read_dir(real_path)?
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
        self.tree.insert_many(files, project_path)?;
        if recursive {
            for folder in folders {
                let folder_name = folder.file_name().unwrap().to_str().unwrap().to_string();
                let folder_project_path = format!("{}/{}", project_path, folder_name);
                self.add_folder(&folder_project_path, folder, recursive)?;
            }
        }

        Ok(())
    }

    pub(crate) fn get_file(&self, project_path: &str) -> Result<HashMap<String, String>> {
        let file = self.tree.get(project_path)?;
        let fpath = self._endpoint.resolve(&file.real_path);
        let mut meta = file.metadata.clone();

        meta.insert("real_path".to_string(), fpath.to_str().unwrap().to_string());

        Ok(meta)
    }

    pub(crate) fn list(
        &self,
        project_path: Option<String>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let list = self.tree.list(project_path)?;
        Ok(list)
    }

    pub(crate) fn remove_file(&mut self, project_path: &str) -> Result<Vec<PathBuf>> {
        let removed_internal_paths = self.tree.remove(project_path)?;
        // filter out paths that are not internal
        let need_to_remove: Vec<PathBuf> = removed_internal_paths
            .into_iter()
            .map(|x| self._endpoint.resolve(&x.real_path))
            .filter(|x| self._endpoint.is_internal(x))
            .collect();
        Ok(need_to_remove)
    }

    pub(crate) fn exists(&self, project_path: String) -> bool {
        self.tree.exists(&project_path)
    }

    pub(crate) fn generate_path(&self, project_path: &str) -> Result<String> {
        let path = self._endpoint.generate_path(project_path)?;
        Ok(path.to_str().unwrap().to_owned())
    }
}

pub fn get_project_manager() -> ProjectManager {
    let storage_manager = StorageManager::get_manager();
    ProjectManager {
        storage_manager,
        projects: HashMap::new(),
        counts: HashMap::new(),
    }
}

pub struct ProjectManager {
    storage_manager: StorageManager,
    projects: HashMap<String, Arc<Mutex<Project>>>,
    counts: HashMap<String, usize>,
}

impl ProjectManager {
    pub fn create_project(
        &mut self,
        name: &str,
        collection: &str,
        force: bool,
        storage_location: Option<String>,
    ) -> Result<Arc<Mutex<Project>>> {
        let key = format!("{}/{}", collection, name);
        let project_dir = create_project_dir(name, collection, force)?;
        let tree = FileSystem::new(name.to_string(), project_dir)?;
        let base_path = match storage_location {
            Some(path) => PathBuf::from(path),
            None => crate::locations::get_default_project_storage_dir(name, collection)?,
        };
        self.storage_manager
            .add(name, collection, "local", base_path.clone())?;
        let endpoint = LocalEndpoint::new(base_path);
        let p = Project {
            tree,
            _name: name.to_string(),
            _collection: collection.to_string(),
            _endpoint: Box::new(endpoint),
        };
        let project = Arc::new(Mutex::new(p));
        self.projects.insert(key.clone(), project.clone());
        self.counts.insert(key, 1);
        Ok(project)
    }

    pub fn import_project(
        &self,
        name: &str,
        collection: &str,
        endpoint: &str,
        path: PathBuf,
    ) -> Result<PathBuf> {
        // The assumption is that the path points to a folder which contains the project data
        // Aditionally, it should contain a .tree folder which contains the tree data

        let project_dir = create_project_dir(name, collection, true)?;
        let tree_path = path.join(".tree");
        let db = sled::open(tree_path)?;
        let _root = db.get("root").unwrap().unwrap();

        let db_export = db.export();
        let final_db = sled::open(&project_dir)?;
        final_db.import(db_export);

        self.storage_manager.add(name, collection, endpoint, path)?;
        Ok(project_dir)
    }
    pub fn export_project(
        &mut self,
        name: &str,
        collection: &str,
        output_path: PathBuf,
    ) -> Result<()> {
        let output_tree_path = output_path.join(".tree");
        let project = self.load_project(name, collection);
        if project.is_err() {
            return Err(project.err().unwrap());
        }
        let project = project.unwrap();
        let mut project = project.lock().unwrap();
        project.duplicate_tree(output_tree_path)?;
        Ok(())
    }

    pub fn load_project(&mut self, name: &str, collection: &str) -> Result<Arc<Mutex<Project>>> {
        let key = format!("{}/{}", collection, name);
        if self.projects.contains_key(&key) {
            let count = self.counts.get(&key).unwrap_or(&0);
            self.counts.insert(key.clone(), count + 1);
            return Ok(self.projects.get(&key).unwrap().clone());
        }
        let project_dir = load_project_dir(name, collection)?;
        let storage_dir = self.storage_manager.get(name, collection)?;
        let tree = FileSystem::load(name, project_dir)?;
        let endpoint = LocalEndpoint::new(storage_dir.1);

        let count = self.counts.get(&key).unwrap_or(&0);
        self.counts.insert(key.clone(), count + 1);

        let project = Project {
            tree,
            _name: name.to_string(),
            _collection: collection.to_string(),
            _endpoint: Box::new(endpoint),
        };
        let project = Arc::new(Mutex::new(project));
        self.projects.insert(key, project.clone());
        Ok(project)
    }

    pub(crate) fn drop_project(&mut self, name: &str, collection: &str) -> Result<()> {
        let key = format!("{}/{}", collection, name);
        let count = self.counts.get(&key);
        if count.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Tried to drop a project {} that is not loaded!", key),
            ));
        }
        let count = count.unwrap();
        if count == &1 {
            self.projects.remove(&key);
            self.counts.remove(&key);
        } else if count < &0 {
            self.counts.remove(&key);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Tried to drop a project {} that does not exist", key),
            ));
        } else {
            self.counts.insert(key, count - 1);
        }
        Ok(())
    }

    pub fn delete_project(&mut self, name: &str, collection: &str, force: bool) -> Result<()> {
        let key = format!("{}/{}", collection, name);
        let pobj = self.projects.remove(&key);
        if let Some(obj) = pobj {
            let obj = obj.lock().unwrap();
            drop(obj);
        }

        let project_dir = load_project_dir(name, collection)?;
        let storage_dir = self.storage_manager.get(name, collection);
        let project_is_empty = is_empty(&project_dir);
        let mut storage_is_empty = storage_dir.is_err();
        if storage_dir.is_ok() {
            let storage_dir = storage_dir.unwrap();
            let storage_path = storage_dir.1;
            let mut files_in_storage = std::fs::read_dir(storage_path)?;
            storage_is_empty = files_in_storage.next().is_none();
        }

        if (project_is_empty && storage_is_empty) || force {
            delete_project_dir(name, collection)?;
            let storage_dir = self.storage_manager.get(name, collection);
            if storage_dir.is_ok() {
                self.storage_manager.delete(name, collection)?;
            }
            return Ok(());
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Project is not empty",
        ))
    }

    pub fn get_project_names(&self, collection: String, show_hidden: bool) -> Result<Vec<String>> {
        let collection_dir = load_collection_dir(&collection);
        if collection_dir.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Collection `{}` does not exist", collection),
            ));
        }
        let collection_dir = collection_dir.unwrap();

        let mut names = Vec::new();
        for entry in std::fs::read_dir(collection_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir()
                && (!path.file_name().unwrap().to_str().unwrap().starts_with('.') || show_hidden)
            {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                names.push(name);
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
        if path.is_dir()
            && (!path.file_name().unwrap().to_str().unwrap().starts_with('.') || show_hidden)
        {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            names.push(name);
        }
    }
    Ok(names)
}
