use crate::pdb::{ProjectFileSystemManager};
use crate::mdb::{MainDBManager, Result, ProjectDocument};
use crate::io::{store, remove_if_internal};
use crate::ftree::{FileTree, FileTreeObject};
use std::path::PathBuf;
use std::clone::Clone;
use std::str::FromStr;
use pyo3::prelude::*;
use pyo3::create_exception;



// Define the project exception
create_exception!(project, GodataProjectError, pyo3::exceptions::PyException);


#[pyclass]
pub(crate) struct ProjectManager {
    db: MainDBManager,
}




#[pyclass]
pub(crate) struct Project {
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

    pub(crate) fn list_projects(&self, colname: Option<&str>) -> PyResult<Vec<String>> {
        let projects = self.db.list_projects(colname);
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

    pub(crate) fn create_project(&self, name: &str, colname: Option<&str>) -> PyResult<Project> {
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
}

#[pymethods]
impl Project {
    pub fn remove(&mut self, project_path: &str, recurisive: Option<bool>) -> PyResult<()> {
        /// Remove a file from the project. This will not delete the file
        /// unless the file has been stored in godata's internal storage.
        
        let result = self.tree.remove(project_path, recurisive.unwrap_or(false));
        match result {
            Ok(fso) => {
                let path = fso.get_location();
                remove_if_internal(path);
                Ok(())
            }
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }

    }

    pub fn get(&self, project_path: &str) -> PyResult<String> {
        /// Get a file from the.
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

    pub fn store(&mut self, object: &PyAny, project_path: &str, output_function: &PyAny, suffix: &str) -> PyResult<()> {
        /// Store an object in the project. The object must be serializable to JSON.
        let result = self.tree.store(project_path, true, suffix);
        match result {
            Ok(path) => {
                let path_str = path.to_str().unwrap();
                store(object, output_function, path_str)?;
                Ok(())
            }
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }

    }


    pub fn add_file(&mut self, file_path: &str, project_path: &str) -> PyResult<()> {
        /// Add a file that already exists to the project. If the folder does not exist, it will
        /// be created recursively. 
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

    pub fn ls(&self, folder_path: Option<&str>) -> PyResult<()>{
        let contents = self.tree.get_contents(folder_path);
        match contents {
            Err(e) => Err(GodataProjectError::new_err(e.msg)),
            Ok(contents) => {
                let header_string: String;
                match folder_path {
                    None => header_string = format!("Project: {}", self.cfg.name),
                    Some(path) => header_string = format!("Folder: {}", path)
                }
                println!("{}", header_string);
                println!("{}","-".repeat(header_string.len()));

                let mut files = Vec::new();
                let mut folders = Vec::new();
                for item in contents {
                    match item {
                        FileTreeObject::Folder(_) => folders.push(item),
                        FileTreeObject::File(_) => files.push(item)
                    }
                }
                if folders.len() == 0 && files.len() == 0 {
                    println!("This folder is empty");
                    return Ok(())
                }
                
                for folder in folders {
                    println!("{}/", folder.get_name())
                }
        
                for file in files {
                    println!("{}", file.get_name());
                }
                println!("{}","-".repeat(header_string.len()));
                Ok(())
            }
        }
    }
}