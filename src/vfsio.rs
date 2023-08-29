/// Basic i-o routines associated with a virtual file system. In particular, responsible
/// for managing the user data directory where the virtual file system is hosted.

use directories::BaseDirs;
use serde::{Serialize, Deserialize};
use polodb_core::{Database, Collection, bson::doc};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::{fmt, result};
use nanoid::nanoid;
pub(crate) struct DBManager {
    db: Database,
}

#[derive(Debug, Clone)]
pub(crate) struct DBError {
    msg: String
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Godata database error: {}", self.msg)
    }
}

type Result<T> = result::Result<T, DBError>;


#[derive(Serialize, Deserialize)]
pub(crate) struct ProjectIOConfig {
    name: String,
    uuid: String,
}

impl DBManager {
    pub(crate) fn get() -> Self {
        let db = get_database();
        initialize_package_root();
        DBManager {
            db,
        }
    }
    fn has_collection(&self, colname: &str) -> bool {

        let known_collections = self.db.list_collection_names();
        match known_collections {
            Ok(collections) => {
                if !(collections.contains(&colname.to_string())) {
                    return false
                }
                else {
                    let res = self.db.collection::<ProjectIOConfig>(colname)
                                        .count_documents()
                                        .is_ok_and(|n| n > 0);
                    return res
                }
            }
            _ => false
        }

    }

    pub(crate) fn has_project(&self, name: &str, colname: &str) -> bool {
        if ! self.has_collection(colname) {
            return false
        }
        let projects: Collection<ProjectIOConfig> = self.db.collection(colname);
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

    pub(crate) fn create_project(&self, name: &str, collection: Option<&str>) {
        let colname: &str;
        match collection {
            Some(cname) => {
                colname = cname;
            }
            None => {
                println!("No collection name provided... Adding to default.");
                colname = "default";
            }
        }
        if self.has_project(name, colname) {
            panic!("Project already exists");
        }

        let projects: Collection<ProjectIOConfig> = self.db.collection(colname);
        let project = ProjectIOConfig {
            name: name.to_string(),
            uuid: nanoid!(),
        };
        let project_path = get_dirs()
                            .get("package_root")
                            .unwrap()
                            .join(&project.uuid);
        std::fs::create_dir_all(&project_path).unwrap();
        projects.insert_one(project);

    }

    pub(crate) fn remove_project(&self, name: &str, colname: &str) {
        if !self.has_project(name, colname) {
            panic!("Project does not exist");
        }
        let projects: Collection<ProjectIOConfig> = self.db.collection(colname);
        let project = projects.find_one(
            doc! {
                "name": name
            }
        ).unwrap()
        .unwrap();
        let project_path = get_dirs()
                            .get("package_root")
                            .unwrap()
                            .join(&project.uuid);
        std::fs::remove_dir_all(&project_path).unwrap();
        projects.delete_one(
            doc! {
                "name": name
            }
        ).unwrap();
    }
}


fn get_dirs() -> HashMap<String, PathBuf> {
    let mut dirs = HashMap::new();
    let base_dir: BaseDirs  = BaseDirs::new().unwrap();
    let data_dir: &Path = base_dir.data_dir();
    let package_root: PathBuf = data_dir.join("godata");
    let db_path: PathBuf = package_root.join(".godata");
    dirs.insert("package_root".to_string(), package_root);
    dirs.insert("db_path".to_string(), db_path);
    dirs
    }

 fn initialize_package_root() {
    let dirs = get_dirs();
    let pkg_root = dirs.get("package_root").unwrap();
    if !pkg_root.exists() {
        std::fs::create_dir_all(&pkg_root).unwrap();
    }
}

fn get_database() -> Database {
    let dirs = get_dirs();
    let db_path = dirs.get("db_path").unwrap();
    let db = Database::open_file(&db_path).unwrap();
    db
}
