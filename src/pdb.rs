/// Databse routines for managing the top-level project and collections database
/// 

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::mdb::{ProjectDocument, Result};
use std::collections::HashMap;
use std::fs;
use serde_json;
use rusqlite::Connection;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

use crate::db;


#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FolderDocument {
    pub(crate) uuid: String,
    pub(crate) name: String,
    pub(crate) children: Vec<String>,
    pub(crate) location: PathBuf,
    pub(crate) parent: Option<String>,
}
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct FileDocument {
    pub(crate) uuid: String,
    pub(crate) name: String,
    pub(crate) parent: String,
    pub(crate) location: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum FileSystemObject {
    Folder(FolderDocument),
    File(FileDocument)
}

impl FileSystemObject {
    

    fn get_parent(&self) -> Option<String> {
        match self {
            FileSystemObject::Folder(f) => f.parent.clone(),
            FileSystemObject::File(f) => Some(f.parent.clone())
        }
    }
    pub(crate) fn get_location(&self) -> PathBuf {
        match self {
            FileSystemObject::Folder(f) => f.location.clone(),
            FileSystemObject::File(f) => f.location.clone()
        }
    }

    pub(crate) fn get_name(&self) -> String {
        match self {
            FileSystemObject::Folder(f) => f.name.clone(),
            FileSystemObject::File(f) => f.name.clone()
        }
    }
}


pub(crate) struct ProjectFileSystemManager {
    project_config: ProjectDocument,
    pool: r2d2::Pool<SqliteConnectionManager>,
}
impl ProjectFileSystemManager {
    pub(crate) fn open(config: ProjectDocument) -> ProjectFileSystemManager {
        if !config.root.exists() {
            fs::create_dir_all(&config.root).unwrap();
        }
        let data_db_path = config.root.join(".godata");
        let manager = SqliteConnectionManager::file(&data_db_path);
        let pool = r2d2::Pool::new(manager).unwrap();
        let folder_metadata_count;
        match db::n_records(pool.clone(), "folder_metadata") {
            Ok(n) => folder_metadata_count = n,
            Err(_) => {
                db::create_kv_table(pool.clone(), "folder_metadata").unwrap();
                folder_metadata_count = 0;
            }
        }
        if folder_metadata_count == 0 {
            let root_folder = FolderDocument {
                name: config.name.clone(),
                uuid: config.uuid.clone(),
                children: Vec::new(),
                location: config.root.clone(),
                parent: None,
            };
            let _ = db::add_to_table(pool.clone(), "folder_metadata", &root_folder.uuid, &root_folder);
            // The above should never fail
        }

        
        ProjectFileSystemManager { project_config: config, pool: pool }       
    }
    pub(crate) fn get_child_records(&self, parent: &FolderDocument) -> Result<Vec<FileSystemObject>> {
        let mut children = Vec::new();
        for child in &parent.children {
            let child_record = db::get_record_from_table(self.pool.clone(), "folder_metadata", &child);
            if child_record.is_none() {
                continue; // THIS NEEDS TO BE DIFFERENT          
            }
            let child_record: FolderDocument = serde_json::from_str(&child_record.unwrap()).unwrap();
            children.push(FileSystemObject::Folder(child_record.clone()));
        }
        let files = db::get_all_records(self.pool.clone(), &parent.uuid).unwrap_or(HashMap::new());
        if files.len() == 0 {
            return Ok(children)
        }
        for file in files.values() {
            let file_obj = serde_json::from_str::<FileDocument>(&file).unwrap();
            children.push(FileSystemObject::File(file_obj));
        }
        Ok(children)
    }
    pub(crate) fn get_root(&self) -> FolderDocument {
        let root_record = db::get_record_from_table(self.pool.clone(), "folder_metadata", &self.project_config.uuid).unwrap();
        let root: FolderDocument = serde_json::from_str(&root_record).unwrap();
        root
    }
    pub(crate) fn add(&mut self, record: &FileSystemObject) -> Result<()> {
        let parent = record.get_parent().unwrap();
        if !db::table_exists(self.pool.clone(), &parent) {
            db::create_kv_table(self.pool.clone(), &parent).unwrap();
        }
        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                db::add_to_table(self.pool.clone(), "folder_metadata", uuid, f).unwrap();
                let parent_record = db::get_record_from_table(self.pool.clone(), "folder_metadata", &parent).unwrap();
                let mut parent_record: FolderDocument = serde_json::from_str(&parent_record).unwrap();
                parent_record.children.push(uuid.clone());
                db::update_record(self.pool.clone(), "folder_metadata", &parent, &parent_record).unwrap();
            }

            FileSystemObject::File(f) => {
                let parent = &f.parent;
                db::add_to_table(self.pool.clone(), parent, &f.uuid, f).unwrap();
            }
        }
        Ok(())

    }
    pub(crate) fn remove(&mut self, record: &FileSystemObject) -> Result<()> {
        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                let children = self.get_child_records(&f)?;
                for child in children {
                    self.remove(&child)?;
                }
                match db::remove(self.pool.clone(), "folder_metadata", uuid) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(crate::mdb::ProjectError{msg: format!("Could not remove folder {} from folder metadata", uuid)})
                    }
                }
                db::delete_kv_table(self.pool.clone(), uuid).unwrap_or({});
            }

            FileSystemObject::File(f) => {
                db::remove(self.pool.clone(), &f.parent, &f.uuid).unwrap_or({});
            }
        }
        Ok(())
    
    }
}