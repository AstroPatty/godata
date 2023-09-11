use serde::{Serialize, Deserialize};
use serde_json::{Value, Map};
use std::collections::HashMap;
use std::hash::Hash;
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
    loc: PathBuf,
    data: HashMap<String, HashMap<String, ProjectDocument>>,
    _modified: bool,
}

impl Drop for MainDBManager {
    fn drop(&mut self) {
        if self._modified {
            let data = serde_json::to_string(&self.data).unwrap();
            std::fs::write(&self.loc, data).unwrap();
            println!("{:?}", &self.loc);
        }
    }
}

impl MainDBManager {
    pub(crate) fn get() -> Self {
        let loc = get_database();
        let data;
        if !loc.exists() {
            data = HashMap::new();
        }
        else {
            let contents = std::fs::read_to_string(&loc).unwrap();
            data = serde_json::from_str(&contents).unwrap();
        }
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
        let mut coldata;
        if !self.data.contains_key(_collection) {
            let c_ = HashMap::new();
            self.data.insert(_collection.to_string(),  c_);
        }
        coldata = self.data.get_mut(_collection).unwrap();
        coldata.insert(name.to_string(), project.clone());
        self._modified = true;
        Ok(project)
    }
    
    pub(crate) fn remove_project(&mut self, name: &str, colname: Option<&str>) -> Result<()> {
        let colname_ = colname.unwrap_or("default");
        let main_collection = self.data.get_mut(colname_).unwrap();
        let result = main_collection.remove(name).unwrap();
        self._modified = true;
        Ok(())
    }
    
    pub (crate) fn list_collections(&self, show_hidden: bool) -> Option<Vec<String>> {
        let _names = self.data.keys().map(|x| x.to_string()).collect::<Vec<String>>();
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

        let projects = self.data.get(colname_).unwrap();
        let project = projects.get(name);
        match project {
            Some(p) => {
                Ok(p.clone())
            }
            None => {
                Err(ProjectError{msg: format!("Project {} does not exist in collection {}", name, colname_)})
            }
        }
    }
    pub(crate) fn list_projects(&self, show_hidden: bool, colname: Option<&str>) -> Result<Vec<String>> {
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

        let projects = self.data.get(colname_).unwrap();
        let names = projects.keys().map(|x| x.to_string()).collect::<Vec<String>>();
        if names.len() == 0 {
            return Err(ProjectError{msg: format!("Collection {} does not exist, or only contains hidden projects", colname_)})
        }
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

        let projects = self.data.get(colname_).unwrap();
        if projects.contains_key(name) {
            return true
        }
        false
    }

    pub(crate) fn has_collection(&self, name: &str) -> bool {
        let collections = self.list_collections(true);
        match collections {
            Some(colls) => {
                if colls.contains(&name.to_string()) {
                    return true
                }
                false
            }
            None => {
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

fn get_database() -> PathBuf {
    let dirs = get_dirs();
    let db_path = dirs.get("db_path").unwrap();
    db_path.clone()
}