/// Databse routines for managing the top-level project and collections database
/// 

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::mdb::{ProjectDocument, Result};
use std::collections::HashMap;
use std::fs;
use serde_json;
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
    _modified: bool,
    folder_data: HashMap<String, HashMap<String, FileDocument>>,
    folder_metadata: HashMap<String, FolderDocument>,
}

impl Drop for ProjectFileSystemManager {
    fn drop(&mut self) {
        if self._modified {
            let folder_data_path = self.project_config.root.join(".folder_data");
            let folder_metadata_path = self.project_config.root.join(".folder_metadata");
            let folder_data = serde_json::to_string(&self.folder_data).unwrap();
            let folder_metadata = serde_json::to_string(&self.folder_metadata).unwrap();
            std::fs::write(&folder_data_path, folder_data).unwrap();
            std::fs::write(&folder_metadata_path, folder_metadata).unwrap();
        }
    }
}

impl ProjectFileSystemManager {
    pub(crate) fn open(config: ProjectDocument) -> ProjectFileSystemManager {
        if !config.root.exists() {
            fs::create_dir_all(&config.root).unwrap();
        }
        let folder_data: HashMap<String, HashMap<String, FileDocument>>;
        let folder_data_path = config.root.join(".folder_data");
        if !folder_data_path.exists() {
            folder_data = HashMap::new();
        }
        else {
            let contents = std::fs::read_to_string(&folder_data_path).unwrap();
            folder_data = serde_json::from_str(&contents).unwrap();
        }

        let mut folder_metadata: HashMap<String, FolderDocument>;
        let folder_metadata_path = config.root.join(".folder_metadata");
        if !folder_metadata_path.exists() {
            folder_metadata = HashMap::new();
        }
        else {
            let contents = std::fs::read_to_string(&folder_metadata_path).unwrap();
            folder_metadata = serde_json::from_str(&contents).unwrap();
        }

        if folder_metadata.keys().len() == 0 {
            let root_folder = FolderDocument {
                name: config.name.clone(),
                uuid: config.uuid.clone(),
                children: Vec::new(),
                location: config.root.clone(),
                parent: None,
            };
            folder_metadata.insert(config.uuid.clone(), root_folder);
        }
        
        ProjectFileSystemManager { project_config: config, folder_data: folder_data, folder_metadata: folder_metadata, _modified: false}       
    }

    pub(crate) fn remove_all(&self) {
        fs::remove_dir_all(&self.project_config.root).unwrap();
    }

    pub(crate) fn get_child_records(&self, parent: &FolderDocument) -> Result<Vec<FileSystemObject>> {
        let mut children = Vec::new();
        for child in &parent.children {
            let child_record = self.folder_metadata.get(child).unwrap();
            children.push(FileSystemObject::Folder(child_record.clone()));
        }
        let folder_info = self.folder_data.get(&parent.uuid);
        if folder_info.is_none() {
            return Ok(children)
        }
        for file in folder_info.unwrap().values() {
            children.push(FileSystemObject::File(file.clone()));
        }
        Ok(children)
    }
    pub(crate) fn get_root(&self) -> FolderDocument {
        let root = self.folder_metadata.get(&self.project_config.uuid).unwrap();
        root.clone()
    }
    pub(crate) fn add(&mut self, record: &FileSystemObject) -> Result<()> {
        let parent = record.get_parent().unwrap();

        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                self.folder_metadata.insert(uuid.clone(), f.clone());
                let parent_children_list = self.folder_metadata.get_mut(&parent).unwrap();
                parent_children_list.children.push(uuid.clone());
                self._modified = true;
                Ok(())
            }

            FileSystemObject::File(f) => {
                let parent = f.parent.clone();
                if !self.folder_data.contains_key(&parent) {
                    self.folder_data.insert(parent.clone(), HashMap::new());
                }
                let parent_collection = self.folder_data.get_mut(&parent).unwrap();
                parent_collection.insert(f.uuid.clone(), f.clone());
                self._modified = true;
                Ok(())
            }
        }

    }
    pub(crate) fn update(&mut self, record: &FileSystemObject) -> Result<()> {
        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                self.folder_metadata.remove(uuid);
                self.folder_metadata.insert(uuid.clone(), f.clone());
                self._modified = true;
                Ok(())
            }

            FileSystemObject::File(f) => {
                let parent = f.parent.clone();
                let file_list = self.folder_data.get_mut(&parent).unwrap();
                file_list.remove(&f.uuid);
                file_list.insert(f.uuid.clone(), f.clone());
                self._modified = true;
                Ok(())
            }
        }

    }
    pub(crate) fn remove(&mut self, record: &FileSystemObject) -> Result<()> {
        match record {
            FileSystemObject::Folder(f) => {
                let uuid = &f.uuid;
                let children = self.get_child_records(&f)?;
                for child in children {
                    self.remove(&child)?;
                }
                self.folder_metadata.remove(uuid);
                self._modified = true;
                Ok(())
            }

            FileSystemObject::File(f) => {
                let parent = &f.parent;
                let parent_collection = self.folder_data.get_mut(parent).unwrap();
                parent_collection.remove(&f.uuid);
                self._modified = true;
                Ok(())
            }
        }
    }
}