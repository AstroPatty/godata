/// Databse routines for managing the top-level project and collections database
/// 

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::mdb::{ProjectDocument, Result};
use std::collections::HashMap;
use std::fs;
use serde_json;
use rusqlite::Connection;
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
}


pub(crate) struct ProjectFileSystemManager {
    project_config: ProjectDocument,
    db_connection: Connection,
}
impl ProjectFileSystemManager {
    pub(crate) fn open(config: ProjectDocument) -> ProjectFileSystemManager {
        if !config.root.exists() {
            fs::create_dir_all(&config.root).unwrap();
        }
        let data_db_path = config.root.join(".godata");
        let data_db = Connection::open(&data_db_path).unwrap();
        let folder_metadata_count;
        match db::n_records(&data_db, "folder_metadata") {
            Ok(n) => folder_metadata_count = n,
            Err(_) => {
                db::create_kv_table(&data_db, "folder_metadata").unwrap();
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
            let _ = db::add_to_table(&data_db, "folder_metadata", &root_folder.uuid, &root_folder);
            // The above should never fail
        }

        
        ProjectFileSystemManager { project_config: config, db_connection: data_db }       
    }
    pub(crate) fn get_child_records(&self, parent: &FolderDocument) -> Result<Vec<FileSystemObject>> {
        let mut children = Vec::new();
        for child in &parent.children {
            let child_record = db::get_record_from_table(&self.db_connection, "folder_metadata", &child).unwrap();
            let child_record: FolderDocument = serde_json::from_str(&child_record).unwrap();
            children.push(FileSystemObject::Folder(child_record.clone()));
        }
        let files = db::get_all_records(&self.db_connection, &parent.uuid).unwrap_or(HashMap::new());
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
        let root_record = db::get_record_from_table(&self.db_connection, "folder_metadata", &self.project_config.uuid).unwrap();
        let root: FolderDocument = serde_json::from_str(&root_record).unwrap();
        root
    }
    pub(crate) fn add(&mut self, record: &FileSystemObject) -> Result<()> {
        let parent = record.get_parent().unwrap();
        if !db::table_exists(&self.db_connection, &parent) {
            db::create_kv_table(&self.db_connection, &parent).unwrap();
        }
        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                db::add_to_table(&self.db_connection, "folder_metadata", uuid, f).unwrap();
                let parent_record = db::get_record_from_table(&self.db_connection, "folder_metadata", &parent).unwrap();
                let mut parent_record: FolderDocument = serde_json::from_str(&parent_record).unwrap();
                parent_record.children.push(uuid.clone());
                db::update_record(&self.db_connection, "folder_metadata", &parent, &parent_record).unwrap();
            }

            FileSystemObject::File(f) => {
                let parent = &f.parent;
                db::add_to_table(&self.db_connection, parent, &f.uuid, f).unwrap();
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
                match db::remove(&self.db_connection, "folder_metadata", uuid) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(crate::mdb::ProjectError{msg: format!("Could not remove folder {} from folder metadata", uuid)})
                    }
                }
                db::delete_kv_table(&self.db_connection, uuid).unwrap_or({});
            }

            FileSystemObject::File(f) => {
                db::remove(&self.db_connection, &f.parent, &f.uuid).unwrap_or({});
            }
        }
        Ok(())
    
    }
}