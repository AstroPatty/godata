/// Databse routines for managing the top-level project and collections database
/// 

use serde::{Serialize, Deserialize};
use polodb_core::{Database, Collection, bson::doc};
use std::path::PathBuf;
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

struct FileDocument {
    name: String,
    uuid: String,
    parent: String,
    location: PathBuf,
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

    pub(crate) fn get_folder_contents(&self, uuid: &str) -> Result<Vec<FolderDocument>> {
        let collection = self.db.collection::<FolderDocument>("folder_metadata");
        let folder_doc = collection.find_one(doc!{
            "uuid": uuid
        }).unwrap();
        match folder_doc {
            Some(doc) => {
                let mut folder_docs = Vec::new();
                for child in doc.children {
                    let child_doc = collection.find_one(doc!{
                        "uuid": child
                    }).unwrap();
                    match child_doc {
                        Some(c) => folder_docs.push(c),
                        None => ()
                    }
                }
                Ok(folder_docs)
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
        folder_collection.insert_one(&folder_doc).unwrap();
        self.link_folder(folder_doc, &folder_collection);
        Ok(uuid)
    }

    fn link_folder(&self, folder_doc: FolderDocument, folder_collection: &Collection<FolderDocument> ) {
        match folder_doc.parent {
            Some(p) => {
                let parent_doc = folder_collection.find_one(doc!{
                    "uuid": p
                }).unwrap();
                match parent_doc {
                    Some(mut doc) => {
                        doc.children.push(folder_doc.uuid);
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