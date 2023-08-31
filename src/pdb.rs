/// Databse routines for managing the top-level project and collections database
/// 

use serde::{Serialize, Deserialize};
use polodb_core::{Database, Collection, bson::doc};
use std::{path::PathBuf, fs::File};
use nanoid::nanoid;
use crate::mdb::{ProjectDocument, Result, ProjectError};
use std::fs;


#[derive(Serialize, Deserialize)]
pub(crate) struct FolderDocument {
    pub(crate) name: String,
    uuid: String,
    children: Vec<String>,
    location: Option<PathBuf>,
    parent: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct FileDocument {
    name: String,
    uuid: String,
    parent: String,
    location: PathBuf,
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
                location: None,
                parent: None,
            };
            folder_metadata.insert_one(&root_folder).unwrap();
        }
        
        ProjectFileSystemManager { project_config: config, db: db}       
    }

    pub(crate) fn folder_exists(&self, folder_path: &[&str], parent: Option<&str>) -> bool {
        match self.get_folder_at_path(folder_path, parent) {
            Some(_) => true,
            None => false
        }
    }

    pub(crate) fn get_folder_contents(&self, uuid: &str) 
        -> Result<Vec<FileSystemObject>> {


        let folder_collection = self.db.collection::<FolderDocument>("folder_metadata");
        let file_collection = self.db.collection::<FileDocument>(uuid);
        let folder_doc = folder_collection.find_one(doc!{
            "uuid": uuid
        }).unwrap();
        match folder_doc {
            Some(doc) => {
                let mut return_objects = Vec::new();
                for child in doc.children {
                    let id_split: Vec<&str> = child.split(":").collect();
                    if id_split[0] == "folder" {
                        let child_doc = folder_collection.find_one(doc!{
                            "uuid": id_split[1]
                        }).unwrap();
                        match child_doc {
                            Some(c) => return_objects.push(FileSystemObject::Folder(c)),
                            None => ()
                        }
                    }
                    else if id_split[0] == "file" {
                        let child_doc = file_collection.find_one(
                            doc!{"uuid": id_split[1]}
                        ).unwrap();
                        match child_doc {
                            Some(c) => return_objects.push(FileSystemObject::File(c)),
                            None => ()
                        }
                    }
                    else {
                        return Err(ProjectError{msg: format!("Invalid child type found {}", id_split[0]).to_string()})
                    }
                }
                Ok(return_objects)
            }
            None => Err(ProjectError{msg: "Folder not found".to_string()})
        }


    }
    
    pub(crate) fn get_folder_at_path(&self, folder_path: &[&str], parent: Option<&str>) -> Option<String> {      
        if folder_path.len() == 1 && parent.is_some() {
            let collection = self.db.collection::<FolderDocument>("folder_metadata");
            let folder_doc = collection.find_one(doc!{
                "name": folder_path[0],
                "parent": parent.unwrap()
            }).unwrap();
            match folder_doc {
                Some(doc) => {
                    return Some(doc.uuid.clone())
                }
                None => return None
            }
        }
        
        match parent {
            Some(p) =>{
                let collection = self.db.collection::<FolderDocument>("folder_metadata");
                let folder_doc = collection.find_one(doc!{
                    "name": folder_path[0],
                    "parent": p
                }).unwrap();
                match folder_doc {
                    Some(doc) => return self.get_folder_at_path(&folder_path[1..], Some(&doc.uuid)),
                    None => return None
                }
            }
            None => {
                let parent_uuid = &self.project_config.uuid;
                return self.get_folder_at_path(folder_path, Some(parent_uuid))
            }
        }


        
    }
    fn name_exists(&self, item: &FileSystemObject) -> bool {
        match item {
            FileSystemObject::File(f) => {
                let parent = f.parent.to_string();
                let collection = self.db.collection::<FileDocument>(&parent);
                let file_doc = collection.find_one(doc!{
                    "name": f.name.to_string()
                }).unwrap();
                match file_doc {
                    Some(_) => true,
                    None => false
                }
            },
            FileSystemObject::Folder(f) => {
                let parent = f.parent.clone().unwrap_or(self.project_config.uuid.clone());
                let collection = self.db.collection::<FolderDocument>("folder_metadata");
                let folder_doc = collection.find_one(doc!{
                    "name": f.name.to_string(),
                    "parent": parent
                }).unwrap();
                match folder_doc {
                    Some(_) => true,
                    None => false
                }
            }
        }
        
    }

    fn insert_and_link(&self, item: &FileSystemObject) -> Result<String> {
        if self.name_exists(item) {
            let item_type = match item {
                FileSystemObject::Folder(_) => "Folder",
                FileSystemObject::File(_) => "File"
            };
            return Err(ProjectError{msg: format!("{} `{}` already exists in this folder!", item_type, item.get_name())})
        }

        let folder_collection = self.db.collection::<FolderDocument>("folder_metadata");
        match item {
            FileSystemObject::Folder(f) => {
                folder_collection.insert_one(f).unwrap();
                self.link(item, &folder_collection);
                Ok(f.uuid.clone())
            }
            FileSystemObject::File(f) => {
                let file_collection = self.db.collection::<FileDocument>(&f.parent);
                file_collection.insert_one(f).unwrap();
                self.link(item, &folder_collection);
                Ok(f.uuid.clone())
            }
        }
    }

    pub(crate) fn attach_file(&self, file_path: &PathBuf, project_path: &str) -> Result<String> {
        let project_path_split = project_path.split(".").collect::<Vec<&str>>();
        let file_name = project_path_split[project_path_split.len()-1];
        let folder_uuid = self.get_folder_at_path(&project_path_split[0..project_path_split.len()-1], None);


        match folder_uuid {
            Some(uuid) => {
                let file_uuid = nanoid!();
                let file_doc = FileDocument {
                    name: file_name.to_string(),
                    uuid: file_uuid.clone(),
                    parent: uuid,
                    location: PathBuf::from(file_path),
                };
                self.insert_and_link(&FileSystemObject::File(file_doc))
            }
            None => Err(ProjectError{msg: "Folder not found".to_string()})
        }
    }
    
 
    pub(crate) fn create_folder(&self, folder_path: &str) -> Result<String> {
        let folder_path_split = folder_path.split(".").collect::<Vec<&str>>();
        let parent_uuid: String;
        if folder_path_split.len() == 1 {
            parent_uuid = self.project_config.uuid.clone();
        } else {
            parent_uuid = self.get_folder_at_path(&folder_path_split[0..folder_path_split.len()-1], None).unwrap();
        }
        
        let uuid = nanoid!();
        let folder_collection: Collection<FolderDocument> = self.db.collection("folder_metadata");
        let folder_doc = FolderDocument {
            name: folder_path_split[folder_path_split.len()-1].to_string(),
            uuid: uuid.clone(),
            children: Vec::new(),
            location: None,
            parent: Some(parent_uuid.to_string()),
        };
        self.insert_and_link(&FileSystemObject::Folder(folder_doc))
    }

    fn link(&self, item: &FileSystemObject, folder_collection: &Collection<FolderDocument> ) {
        let child_uid = item.get_identifier();
        let child_type = item.get_type();
        let child_id = format!("{}:{}", child_type, child_uid);


        match item.get_parent() {
            Some(p) => {
                let parent_doc = folder_collection.find_one(doc!{
                    "uuid": p
                }).unwrap();
                match parent_doc {
                    Some(mut doc) => {
                        doc.children.push(child_id);
                        folder_collection.update_one(doc!{
                            "uuid": doc.uuid
                        }, doc! {
                            "$set": {
                                "children": doc.children
                            }
                        }).unwrap();
                    }
                    None => ()  //Error handling if the parent is not found
                }

            }

            None => ()
        }
    }
}