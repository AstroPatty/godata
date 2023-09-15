use crate::pdb::{ProjectFileSystemManager};
use crate::mdb::{MainDBManager, ProjectDocument};
use crate::io::{store, remove_if_internal};
use crate::ftree::{FileTree, FileTreeObject};
use std::path::PathBuf;
use std::clone::Clone;
use std::str::FromStr;
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::create_exception;




// Define the project exception
create_exception!(project, GodataProjectError, pyo3::exceptions::PyException);


#[pyclass]
pub(crate) struct ProjectManager {
    db: MainDBManager,
}




#[pyclass]
pub struct Project {
    cfg: ProjectDocument,
    tree: FileTree,
}

#[pymethods]
impl ProjectManager {
    #[new]
    pub(crate) fn new() -> ProjectManager {
        let db = MainDBManager::get();
        ProjectManager {
            db: db,
        }        
    }

    pub(crate) fn list_collections(&self, show_hidden: bool) -> PyResult<Vec<String>> {
        let collections = self.db.list_collections(show_hidden);
        match collections {
            Some(collections) => {Ok(collections)},
            None => {Err(GodataProjectError::new_err("No collections found, or collections are hidden"))}
        }
    }

    pub(crate) fn list_projects(&self, show_hidden: bool, colname: Option<&str>) -> PyResult<Vec<String>> {
        let projects = self.db.list_projects(show_hidden, colname);
        match projects {
            Ok(projects) => {Ok(projects)},
            Err(e) => {Err(GodataProjectError::new_err(e.msg))}
        }
    }

    pub(crate) fn load_project(&self, name: &str, colname: Option<&str>) -> PyResult<Project> {
        let pconfig = self.db.get_project(name, colname);
        match pconfig {
            Ok(config) => {
                let fs = ProjectFileSystemManager::open(config.clone());
                let tree = FileTree::new_from_db(fs);
                Ok(Project {
                    cfg: config,
                    tree: tree
                })
            }
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }

    }

    pub(crate) fn create_project(&mut self, name: &str, colname: Option<&str>) -> PyResult<Project> {
        let pconfig = self.db.create_project(name, colname);
        match pconfig {
            Ok(config) => {
                let fs = ProjectFileSystemManager::open(config.clone());
                let tree = FileTree::new_from_db(fs);
                Ok(Project {
                    cfg: config,
                    tree: tree
                })
            }
            Err(e) => Err(GodataProjectError::new_err(e.msg))

        }
    }
    pub(crate) fn remove_project(&mut self, name: &str, colname: Option<&str>) -> PyResult<()> {
        let project = self.load_project(name, colname);
        // Make sure the project exists
        let project_ = match project {
            Ok(p_) => {
                p_
            }
            Err(_e) => {
                return Err(GodataProjectError::new_err(format!("Project {} does not exist", name)))
            }
        };
        let all_children = project_.tree.get_contents(true, None).unwrap();
        for child in all_children {
            match child {
                FileTreeObject::File(f) => {
                    let path = &f.cfg.location;
                    remove_if_internal(path);
                }
                FileTreeObject::Folder(_) => ()
            }
        }
        let root = project_.cfg.root; // Clean folder tree
        remove_if_internal(&root);
        let _ = match self.db.remove_project(name, colname) {
            Err(e) => Err(GodataProjectError::new_err(e.msg)),
            Ok(_) => Ok(())            
        };

        Ok(())
        // Check for data stored internally

    }
}

#[pymethods]
impl Project {
    /// Remove a file from the project. This will not delete the file
    /// unless the file has been stored in godata's internal storage.
    pub fn remove(&mut self, project_path: &str, recurisive: Option<bool>) -> PyResult<()> {
        
        let result = self.tree.remove(project_path, recurisive.unwrap_or(false));
        match result {
            Ok(fso) => {
                let path = fso.get_location();
                remove_if_internal(&path);
                Ok(())
            }
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }

    }
    /// Get a file from the project.
    pub fn get(&self, project_path: &str) -> PyResult<String> {
        let result = self.tree.query(project_path);
        match result {
            Ok(item) => {
                let path = item.get_path();
                match item {
                    FileTreeObject::File(_) => {
                        let path_str = &path.to_str().unwrap();
                        Ok(path_str.to_string())
                    }
                    FileTreeObject::Folder(_) => {
                        Err(GodataProjectError::new_err(format!("`{}` is a folder", project_path)))
                    }
                }
            }
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }
    }

    /// Store an object in the project.
    pub fn store(&mut self, object: &PyAny, project_path: &str, output_function: Option<&PyAny>, suffix: Option<&str>) -> PyResult<()> {
        match (output_function, suffix) {
            (Some(func), Some(suff)) => {
                let result = self.tree.store(project_path, true, suff);
                match result {
                    Ok(path) => {
                        let path_str = path.to_str().unwrap();
                        store(object, func, path_str)?;
                        Ok(())
                    }
                    Err(e) => Err(GodataProjectError::new_err(e.msg))
                }
            }
            _ => {
                Err(GodataProjectError::new_err("Rust io for internally stored files is not yet implemented"))
            }
        }
    }
    /// Add a file that already exists to the project. If the folder does not exist, it will
    /// be created recursively. 
    pub fn add_file(&mut self, file_path: &str, project_path: &str) -> PyResult<()> {
        let path = PathBuf::from_str(file_path).unwrap().canonicalize().unwrap();
        if !path.exists() || !path.is_file() {
            return Err(GodataProjectError::new_err(format!("No file found at `{file_path}`")))
        }
        let result = self.tree.add_file(path, project_path, true);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }
    }
    pub fn list(&self, folder_path: Option<&str>) -> PyResult<HashMap<String, Vec<String>>> {
        let contents = self.tree.get_contents(false, folder_path);
        let mut files: Vec<String> = Vec::new();
        let mut folders: Vec<String> = Vec::new();

        match contents {
            Err(e) => return Err(GodataProjectError::new_err(e.msg)),
            Ok(contents) => {
                for item in contents {
                    match item {
                        FileTreeObject::Folder(_f) => {
                            folders.push(item.get_name().to_string());
                        }
                        FileTreeObject::File(_) => {
                            files.push(item.get_name().to_string());
                        }
                    }
                }
            }
        }
        let mut output = HashMap::new();
        output.insert(String::from("files"), files);
        output.insert(String::from("folders"), folders);
        Ok(output)


    }
}