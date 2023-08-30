use polodb_core::{Database, Collection};
use polodb_core::bson::doc;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use directories::BaseDirs;
use nanoid::nanoid;
use std::result;
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
    db: Database,
}

impl MainDBManager {
    pub(crate) fn get() -> Self {
        let db = get_database();
        MainDBManager {
            db,
        }
    }

    pub(crate) fn create_project(&self, name: &str, collection: Option<&str>) -> Result<ProjectDocument> {
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

        match collection {
            Some(colname) => {
                let collection = self.db.collection::<ProjectDocument>(colname);
                collection.insert_one(&project);
                Ok(project)
            }
            None => {
                let collection = self.db.collection::<ProjectDocument>("default");
                collection.insert_one(&project);
                Ok(project)
            }
        }
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

        let projects: Collection<ProjectDocument> = self.db.collection(colname_);
        let project = projects.find_one(
            doc! {
                "name": name
            }
        ).unwrap();
        match project {
            Some(p) => {
                Ok(p)
            }
            None => {
                Err(ProjectError{msg: format!("Project {} does not exist in collection {}", name, colname_)})
            }
        }
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

        let projects: Collection<ProjectDocument> = self.db.collection(colname_);
        let project = projects.find_one(
            doc! {
                "name": name
            }
        ).unwrap();
        if project.is_some() {
            return true
        }
        false
    }
    pub(crate) fn has_collection(&self, name: &str) -> bool {
        let collections = self.db.list_collection_names();
        match collections {
            Ok(colls) => {
                for coll in colls {
                    if coll == name {
                        return true
                    }
                }
                false
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

fn get_database() -> Database {
    let dirs = get_dirs();
    let db_path = dirs.get("db_path").unwrap();
    let db = Database::open_file(&db_path).unwrap();
    db
}