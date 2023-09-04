/// Databse routines for managing the top-level project and collections database
/// 

use serde::{Serialize, Deserialize};
use polodb_core::{Database, Collection, bson::doc};
use std::{path::PathBuf, fs::File};
use nanoid::nanoid;
use crate::mdb::{ProjectDocument, Result, ProjectError};
use crate::ftree::FileTreeObject;
use std::fs;


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


pub(crate) enum FileSystemObject {
    Folder(FolderDocument),
    File(FileDocument)
}

impl FileSystemObject {
    
    fn get_identifier(&self) -> String {
        match self {
            FileSystemObject::Folder(f) =>  f.uuid.clone(),
            FileSystemObject::File(f) => f.uuid.clone()
        }
    }

    fn get_parent(&self) -> Option<String> {
        match self {
            FileSystemObject::Folder(f) => f.parent.clone(),
            FileSystemObject::File(f) => Some(f.parent.clone())
        }
    }
    pub(crate) fn get_name(&self) -> String {
        match self {
            FileSystemObject::Folder(f) => f.name.clone(),
            FileSystemObject::File(f) => f.name.clone()
        }
    }
    fn get_type(&self) -> String {
        match self {
            FileSystemObject::Folder(_) => "folder".to_string(),
            FileSystemObject::File(_) => "file".to_string()
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
    db: Database
}


impl ProjectFileSystemManager {
    pub(crate) fn open(config: ProjectDocument) -> ProjectFileSystemManager {
        if !config.root.exists() {
            fs::create_dir_all(&config.root).unwrap();
        }
        let db_path = config.root.join(".godata");
        let db = Database::open_file(&db_path).unwrap();
        let folder_metadata = db.collection::<FolderDocument>("folder_metadata");
        if folder_metadata.count_documents().unwrap_or(0) == 0 {
            let root_folder = FolderDocument {
                name: config.name.clone(),
                uuid: config.uuid.clone(),
                children: Vec::new(),
                location: config.root.clone(),
                parent: None,
            };
            folder_metadata.insert_one(&root_folder).unwrap();
        }
        
        ProjectFileSystemManager { project_config: config, db: db}       
    }
    pub(crate) fn get_child_records(&self, parent: &FolderDocument) -> Result<Vec<FileSystemObject>> {
        let mut children = Vec::new();
        for child in &parent.children {
            let collection = self.db.collection::<FolderDocument>("folder_metadata");
            let child_record = collection.find_one(doc!{
                "uuid": child
            }).unwrap().unwrap();
            children.push(FileSystemObject::Folder(child_record));
        }
        let collection = self.db.collection::<FileDocument>(&parent.uuid);
        let file_records = collection.find(doc!{
            "parent": parent.uuid.clone()
        }).unwrap();
        for file in file_records {
            children.push(FileSystemObject::File(file.unwrap()));
        }
        Ok(children)
    }
    pub(crate) fn get_root(&self) -> FolderDocument {
        let collection = self.db.collection::<FolderDocument>("folder_metadata");
        collection.find_one(doc!{
            "uuid": self.project_config.uuid.clone()
        }).unwrap().unwrap() //This should never fail, since the root is always created
    }
    pub(crate) fn add(&self, record: &FileSystemObject) -> Result<()> {
        let parent = record.get_parent().unwrap();

        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                let collection = self.db.collection::<FolderDocument>("folder_metadata");
                collection.insert_one(f).unwrap();
                let parent_children_list = &collection.find_one(doc!{
                    "uuid": &parent
                }).unwrap().unwrap();
                let mut children = parent_children_list.children.clone();
                children.push(uuid.clone());
                collection.update_one(doc!{
                    "uuid": &parent
                }, doc!{ 
                    "$set": {
                        "children": children
                    }
                }).unwrap();
                
                Ok(())
            }

            FileSystemObject::File(f) => {
                let parent = f.parent.clone();
                let parent_collection = self.db.collection::<FileDocument>(&parent);
                parent_collection.insert_one(f).unwrap();
                Ok(())
            }
        }

    }
    pub(crate) fn update(&self, record: &FileSystemObject) -> Result<()> {
        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                let collection = self.db.collection::<FolderDocument>("folder_metadata");
                let current = collection.delete_one(doc!{
                    "uuid": uuid
                }).unwrap();
                collection.insert_one(f).unwrap();                
                Ok(())
            }

            FileSystemObject::File(f) => {
                let parent = f.parent.clone();
                let parent_collection = self.db.collection::<FileDocument>(&parent);
                let current = parent_collection.delete_one(doc!{
                    "uuid": f.uuid.clone()
                }).unwrap();
                parent_collection.insert_one(f).unwrap();
                Ok(())
            }
        }

    }
    pub(crate) fn remove(&self, record: &FileSystemObject) -> Result<()> {
        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                let children = self.get_child_records(&f)?;
                for child in children {
                    self.remove(&child)?;
                }
                let collection = self.db.collection::<FolderDocument>("folder_metadata");
                collection.delete_one(doc!{
                    "uuid": uuid
                }).unwrap();
                Ok(())
            }

            FileSystemObject::File(f) => {
                let parent = &f.parent;
                let parent_collection = self.db.collection::<FileDocument>(&parent);
                parent_collection.delete_one(doc!{
                    "uuid": f.uuid.clone()
                }).unwrap();
                Ok(())
            }
        }
    }
}