use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use std::path::{PathBuf, Path};
use directories::BaseDirs;
use nanoid::nanoid;
use std::result;
use rusqlite::Connection;
use crate::db;
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ProjectDocument {
    pub(crate) name: String,
    pub(crate) uuid: String,
    pub(crate) root: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ProjectError {
    pub(crate) msg: String
}


pub(crate) type Result<T> = result::Result<T, ProjectError>;



pub(crate) struct MainDBManager{
    #[allow(unused)]
    loc: PathBuf,
    data: Connection,
    _modified: bool,
}


impl MainDBManager {
    pub(crate) fn get() -> Self {
        let loc = get_database_path();
        let data = Connection::open(&loc).unwrap();
        MainDBManager {
            loc: loc,
            data: data,
            _modified: false,
        }
    }

    pub(crate) fn create_project(&mut self, name: &str, collection: Option<&str>) -> Result<ProjectDocument> {
        if self.has_project(name, collection) {
            return Err(ProjectError{msg: "Project already exists".to_string()})
        }


        let uuid = nanoid!();
        let path = get_dirs()
                    .get("data_dir")
                    .unwrap()
                    .join(&uuid);
        
        let project = ProjectDocument {
            name: name.to_string(),
            uuid: uuid,
            root: path
        };
        let _collection = collection.unwrap_or("default");
        if !db::table_exists(&self.data, _collection) {
            db::create_kv_table(&self.data, _collection).unwrap();
        }
        db::add_to_table(&self.data, _collection, name, &project).unwrap();
        Ok(project)
    }
    
    pub(crate) fn remove_project(&mut self, name: &str, colname: Option<&str>) -> Result<()> {
        let colname_ = colname.unwrap_or("default");
        let _result = db::remove(&self.data, colname_, name).unwrap();
        if db::n_records(&self.data, colname_).unwrap_or(1) == 0 {
            db::delete_kv_table(&self.data, colname_).unwrap();
        }
        Ok(())
    }
    
    pub (crate) fn list_collections(&self, _show_hidden: bool) -> Option<Vec<String>> {
        let _names = db::list_tables(&self.data);
        if _names.len() == 0 {
            return None
        }
        Some(_names)
    }
    
    pub(crate) fn get_project(&self, name:&str, colname: Option<&str>) -> Result<ProjectDocument> {
        let colname_: &str;
        match colname {
            Some(cname) => {
                colname_ = cname;
            }
            None => {
                colname_ = "default";
            }
        }
        if !self.has_collection(colname_) {
            return Err(ProjectError{msg: format!("Collection {} does not exist", colname_)})
        }
        let p_ = db::get_record_from_table(&self.data, colname_, name);
        let p = match p_ {
            Some(p) => {
                p
            }
            None => {
                return Err(ProjectError{msg: format!("Project {} does not exist in collection {}", name, colname_)})
            }
        };
        let project: ProjectDocument = serde_json::from_str(&p).unwrap();
        Ok(project)
    }
    pub(crate) fn list_projects(&self, _show_hidden: bool, colname: Option<&str>) -> Result<Vec<String>> {
        let colname_: &str;
        match colname {
            Some(cname) => {
                colname_ = cname;
            }
            None => {
                colname_ = "default";
            }
        }
        if !self.has_collection(colname_) {
            return Err(ProjectError{msg: format!("Collection `{}` does not exist", colname_)})
        }

        let names = db::get_keys(&self.data, colname_);
        Ok(names)
    }

    pub(crate) fn has_project(&self, name: &str, colname: Option<&str>) -> bool {
        let colname_: &str;
        match colname {
            Some(cname) => {
                colname_ = cname;
            }
            None => {
                colname_ = "default";
            }
        }
        if !self.has_collection(colname_) {
            return false
        }

        let projects = db::get_keys(&self.data, colname_);
        if projects.iter().any(|k| k == name) {
            return true
        }
        false
    }

    pub(crate) fn has_collection(&self, name: &str) -> bool {
        let n_records = db::n_records(&self.data, name);
        match n_records {
            Ok(n) => {
                return n > 0
            }
            Err(_) => {
                false
            }
        }
    }
}

pub(crate) fn get_dirs() -> HashMap<String, PathBuf> {
    let mut dirs = HashMap::new();
    let base_dir: BaseDirs  = BaseDirs::new().unwrap();
    let user_data_dir: &Path = base_dir.data_dir();
    let package_root: PathBuf = user_data_dir.join("godata");
    if !package_root.exists() {
        std::fs::create_dir_all(&package_root).unwrap();
    }

    let db_path: PathBuf = package_root.join(".godata");
    let data_dir: PathBuf = package_root.join("data");
    dirs.insert("package_root".to_string(), package_root);
    dirs.insert("db_path".to_string(), db_path);
    dirs.insert("data_dir".to_string(), data_dir);
    dirs
}

fn get_database_path() -> PathBuf {
    let dirs = get_dirs();
    let db_path = dirs.get("db_path").unwrap();
    db_path.clone()
}