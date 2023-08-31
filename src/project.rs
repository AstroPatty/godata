use crate::pdb::{ProjectFileSystemManager, FileSystemObject};
use crate::mdb::{MainDBManager, Result, ProjectDocument};
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
    fs: ProjectFileSystemManager,
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
                Ok(Project {
                    cfg: config,
                    fs: fs,
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
                Ok(Project {
                    cfg: config,
                    fs: fs,
                })
            }
            Err(e) => Err(GodataProjectError::new_err(e.msg))

        }
    }
}

#[pymethods]
impl Project {
    
    pub fn mkdir(&self, folder_path: &str) -> PyResult<()>{
        let result = self.fs.create_folder(folder_path);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }
    }

    pub fn add_file(&self, file_path: &str, project_path: &str) -> PyResult<()> {
        let path = PathBuf::from_str(file_path).unwrap();
        if !path.exists() || !path.is_file() {
            return Err(GodataProjectError::new_err(format!("No file found at `{file_path}`")))
        }
        let result = self.fs.attach_file(&path, project_path);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(GodataProjectError::new_err(e.msg))
        }
    }

    pub fn ls(&self, folder_path: Option<&str>) -> PyResult<()>{
        let folder_uuid: String;
        match folder_path {
            None => {
                folder_uuid = self.cfg.uuid.clone();
            }
            Some(path) => {
                let folder_uuid_ = self.fs.get_folder_at_path(&path.split(".").collect::<Vec<&str>>(), None);
                match folder_uuid_ {
                    Some(uuid) => {
                        folder_uuid = uuid;
                    }
                    None => {
                        return Err(GodataProjectError::new_err("Folder not found"))
                    }
                }
            }

        }
        let contents = self.fs.get_folder_contents(&folder_uuid).unwrap();
        let mut files = Vec::new();
        let mut folders = Vec::new();
        for item in contents {
            match item {
                FileSystemObject::Folder(_) => folders.push(item),
                FileSystemObject::File(_) => files.push(item)
            }
        }


        for folder in folders {
            println!("{}/", folder.get_name())
        }

        for file in files {
            println!("{}", file.get_name());
        }
        Ok(())
        

    }

}