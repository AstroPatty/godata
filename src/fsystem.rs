
// Definition of the virtual file system. Folders in the file system may be backed
// by real folders, or may be entirely virtual. Files in the file system are always
// backed by real files.

// As far as the rest of the library is concrened, 

use std::io::Result;
use std::collections::HashMap;
use uuid::Uuid;
use sled::Db;
use bincode;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
enum FSObject {
    File(File),
    Folder(Folder)
}

impl FSObject {
    fn get_name(&self) -> &str {
        match self {
            FSObject::File(f) => f.get_name(),
            FSObject::Folder(f) => f.get_name()
        }
    }
}

struct File {
    real_path: String,
    pub(self) name: String,
    _uuid: String,
}

struct Folder {
    pub(self) name: String,
    children: HashMap<String, FSObject>,
    _uuid: String,
    _modified: bool
}

#[derive(Serialize, Deserialize)]
struct DbFolder {
    pub(self) name: String,
    folders_uuids: Vec<String>,
    files: Vec<DbFile>,
}

#[derive(Serialize, Deserialize)]
struct DbFile {
    pub(self) name: String,
    real_path: String,
    uuid: String
}

pub(crate) struct FileSystem {
    root: Folder,
    _name: String,
    db: Db
}

pub(crate) fn is_empty(path: &PathBuf) -> bool {
    let db = sled::open(path).unwrap();
    // Count the entires in the database
    let root_folder = db.get("root".as_bytes()).unwrap();
    // Deserialize the root folder
    let db_folder  = bincode::deserialize(root_folder.unwrap().as_ref());
    let db_folder: DbFolder = db_folder.unwrap();
    // If there are any files or folders in the root folder, return false
    if db_folder.folders_uuids.len() > 0 || db_folder.files.len() > 0 {
        return false;
    }
    true
}


impl FileSystem {
    pub(crate) fn new(name: String, root_path: PathBuf) -> Result<FileSystem> {
        let db = sled::open(root_path)?;
        let root_folder = db.get("root".as_bytes()).unwrap();
        // If there is already a root folder, fail
        let root = match root_folder {
            None => {
                let root = Folder {
                    name: "root".to_string(),
                    children: HashMap::new(),
                    _uuid: "root".to_string(),
                    _modified: true
                };
                root
        
            },
            Some(_) => {
                return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "File system already exists"))
            }
        };

        Ok(FileSystem {
            root,
            _name: name,
            db
        })
    }

    pub(crate) fn load(name: String, root_dir: PathBuf) -> Result<FileSystem> {

        let db = sled::open(root_dir)?;
        let root_folder = db.get("root".as_bytes()).unwrap();
        // If there is no root folder, fail
        let root = match root_folder {
            None => {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File system not found"))
            },
            Some(_) => {
                Folder::from_tree(&db, "root".to_string())
            }
        };
        Ok(FileSystem {
            root,
            _name: name,
            db
        })
    }

    pub(crate) fn list(&self, virtual_path: Option<String>) -> Result<HashMap<String, Vec<String>>> {
        let folder = match virtual_path {
            Some(p) => self.root.get_folder(&p)?,
            None => &self.root
        };
        let mut folders = Vec::new();
        let mut files = Vec::new();
        for (name, child) in folder.children.iter() {
            match child {
                FSObject::File(_) => files.push(name.clone()),
                FSObject::Folder(_) => folders.push(name.clone())
            }
        }
        let mut children = HashMap::new();
        children.insert("folders".to_string(), folders);
        children.insert("files".to_string(), files);
        Ok(children)
    }

    pub(crate) fn get(&self, virtual_path: &str) -> Result<String> {
        let file = self.root.get(virtual_path)?;
        Ok(file.real_path.clone())
    }

    pub(crate) fn insert(&mut self, name: String, real_path: String, virtual_path: &str) -> Result<()> {
        let file = File::new(real_path, name);
        self.root.insert(FSObject::File(file), virtual_path)?;
        Ok(())
    }

    pub(crate) fn remove(&mut self, virtual_path: &str) -> Result<()> {
        let result = self.root.delete(virtual_path)?;
        for uuid in result {
            self.db.remove(uuid.as_bytes())?;
        }
        Ok(())
    }

    pub(crate) fn exists(&self, virtual_path: &str) -> bool {
        self.root.exists(virtual_path)
    }

    
}

impl Drop for FileSystem {
    fn drop(&mut self) {
        self.root.to_tree(&self.db);
    }
}



impl Folder {
    fn new(name: String) -> Folder {
        Folder {
            name,
            children: HashMap::new(),
            _uuid: Uuid::new_v4().to_string(),
            _modified: true
        }
    }

    fn from_tree(db: &Db, uuid: String) -> Folder {
        let folder_info = db.get(uuid.as_bytes()).unwrap();
        let db_folder: DbFolder = bincode::deserialize(folder_info.unwrap().as_ref()).unwrap();
        let mut children = HashMap::new();
        for fuuid in db_folder.folders_uuids {
            let folder = Folder::from_tree(db, fuuid);
            children.insert(folder.name.clone(), FSObject::Folder(folder));
        }

        for file in db_folder.files {
            children.insert(file.name.clone(), FSObject::File(File::from_db_file(file)));
        }
        
        Folder {
            name: db_folder.name,
            children,
            _uuid: uuid,
            _modified: false
        }

    }

    fn to_tree(&self, db: &Db) {
        // Write the folder and all of its children to the database
        if self._modified {

            self.write_to_db(db).unwrap();
        }
        for (_, child) in self.children.iter() {
            match child {
                FSObject::File(_) => (),
                FSObject::Folder(f) => f.to_tree(db)
            }
        }
    }

    
    fn to_db_folder(&self) -> DbFolder {
        let mut folders_uuids = Vec::new();
        let mut files = Vec::new();
        for (_, child) in self.children.iter() {
            match child {
                FSObject::File(f) => files.push(f.to_db_file()),
                FSObject::Folder(f) => folders_uuids.push(f._uuid.clone())
            }
        }
        DbFolder {
            name: self.name.clone(),
            folders_uuids,
            files
        }

    }

    fn write_to_db(&self, db: &Db) -> Result<()> {
        let db_folder = self.to_db_folder();
        let db_folder_bytes = bincode::serialize(&db_folder).unwrap();
        db.insert(self._uuid.as_bytes(), db_folder_bytes.as_slice())?;
        Ok(())
    }

    fn exists(&self, virtual_path: &str) -> bool {
        // Check if a file or folder exists in the folder.
        // If path is this folder's name, return true
        // If path is a subfolder, return true if it exists in the subfolder

        // split up the path
        let path_parts = virtual_path.split("/");
        self._exists(path_parts)        
    }

    fn _exists(&self, mut path_parts: std::str::Split<&str>) -> bool  {
        // Check if a file or folder exists in the folder.
        // If path is this folder's name, return true
        // If path is a subfolder, return true if it exists in the subfolder

        let path_part = path_parts.next();
        let child = match path_part { 
            None => return true, //We're in a folder and there's nowhere left to go, it exists
            Some(part) => {
                self.children.get(part)
            }
        };

        match child {
            None => return false, // child doesn't exist
            Some(f) => {
                match f {
                    FSObject::File(_) => return path_parts.next() == None, // We have a file with this name, and nothing is left in the path
                    FSObject::Folder(f) => return f._exists(path_parts) // We have a folder with this name, and we need to check the rest of the path
                }
            }
        }
    }


    fn get(&self, virtual_path: &str) -> Result<&File> {
        // Get a file from the folder. Will fail if there is no file located
        // at the virtual path.

        // split up the path
        let path_parts = virtual_path.split("/");
        self._get(path_parts)
    }

    fn _get(&self, mut path_parts: std::str::Split<&str>) -> Result<&File> {
        // Get a file or folder from the folder.
        // If path is this folder's name, return it
        // If path is a subfolder, return it from the subfolder

        let path_part = path_parts.next();
        let child = match path_part { 
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")),
            Some(part) => {
                self.children.get(part)
            }
        };

        match child {
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")), // child doesn't exist
            Some(f) => {
                match f {
                    FSObject::File(f) => {
                        if path_parts.next() == None {
                            return Ok(f)
                        } else {
                            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
                        }
                    }
                    FSObject::Folder(f) => return f._get(path_parts) // We have a folder with this name, and we need to check the rest of the path
                }
            }
        }
    }

    fn get_folder(&self, virtual_path: &str) -> Result<&Folder> {
        // Get a folder from the folder. Will fail if there is no folder located
        // at the virtual path.

        // split up the path
        let path_parts = virtual_path.split("/");
        self._get_folder(path_parts)
    }

    fn _get_folder(&self, mut path_parts: std::str::Split<&str>) -> Result<&Folder> {
        // Get a file or folder from the folder.
        // If path is this folder's name, return it
        // If path is a subfolder, return it from the subfolder

        let path_part = path_parts.next();
        let child = match path_part { 
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Folder not found")),
            Some(part) => {
                self.children.get(part)
            }
        };

        match child {
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Folder not found")), // child doesn't exist
            Some(f) => {
                match f {
                    FSObject::File(_) => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Folder not found")), // We have a file with this name, and nothing is left in the path
                    FSObject::Folder(f) => {
                        if path_parts.next() == None {
                            return Ok(f)
                        } else {
                            return f._get_folder(path_parts) // We have a folder with this name, and we need to check the rest of the path
                        }
                    }
                }
            }
        }
    }


    fn insert(&mut self, fs_object: FSObject, virtual_path: &str) -> Result<()> {
        // Insert a file or folder into the folder.
        // If path is this folder's name, insert it here
        // If path is a subfolder, insert it into the subfolder

        // split up the path
        let path_parts = virtual_path.split("/");
        self._insert(fs_object, path_parts)?;
        Ok(())
    }

    fn _insert(&mut self, fs_object: FSObject, mut path_parts: std::str::Split<&str>) -> Result<()> {
        // Insert a file or folder into the folder.
        // If path is this folder's name, insert it here
        // If path is a subfolder, insert it into the subfolder

        // split up the path
        let path_part = path_parts.next();
        let child = match path_part { 
            None => { //We're at the end, try to insert it here
                if self.children.contains_key(fs_object.get_name()) {
                    return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "File already exists"))
                } else {
                    self.children.insert(fs_object.get_name().to_string(), fs_object);
                    self._modified = true;
                    return Ok(())
                }
            },
            Some(part) => {
               self.children.get_mut(part) 
            }
        };

        match child {
            None => { // child doesn't exist, create it
                let mut folder = Folder::new(path_part.unwrap().to_string());
                folder._insert(fs_object, path_parts).unwrap();
                self.children.insert(folder.name.clone(), FSObject::Folder(folder));
                self._modified = true;
                return Ok(())
            },
            Some(f) => {
                match f {
                    FSObject::File(_) => return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Invalid path")), // We have a file with this name, and nothing is left in the path
                    FSObject::Folder(f) => return f._insert(fs_object, path_parts) // We have a folder with this name, and we need to check the rest of the path
                }
            }
        }
    }
    
    fn delete(&mut self, virtual_path: &str) -> Result<Vec<String>> {
        // Delete a file or folder from the folder.
        // If path is this folder's name, delete it here
        // If path is a subfolder, delete it from the subfolder

        // split up the path
        let path_parts = virtual_path.split("/");
        let result = self._delete(path_parts)?;
        Ok(result.1)
    }

    fn _delete(&mut self, mut path_parts: std::str::Split<&str>) -> Result<(bool, Vec<String>)> {
        // Delete a file or folder from the folder.
        // If path is this folder's name, delete it here
        // If path is a subfolder, delete it from the subfolder

        // split up the path
        let path_part = path_parts.next();
        let child = match path_part { 
            None => { // This folder is getting deleted, so we need to tell the parent to remove it
                let storage = Vec::new();
                return Ok((true, storage));
            },
            Some(part) => {
               self.children.remove(part) 
            }        
        };
        let mut child = match child {
            None => { // child doesn't exist, raise an error 
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found"))
            },

            Some(child_obj) => {
                child_obj
            }
        };

        let output = match child {
            FSObject::File(_) => { // We have a file with this name...
                if path_parts.next() == None { // ...and nothing is left in the path
                    self._modified = true; // We've modified the folder, so we need to write it to the database
                    let storage = Vec::new();
                    return Ok((self.children.len() == 0, storage)) // The removal was sucessful, but the folder above us doesn't need to do anything
                } else {
                    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found"))
                }
            },

            FSObject::Folder(ref mut f) => { // We have a folder with this name, and we need to check the rest of the path
                let result = f._delete(path_parts)?; 
                if result.0 { // The child needs to be deleted, so remove it
                    let mut to_remove = result.1;
                    to_remove.push(f._uuid.clone());
                    self._modified = true; // We've modified the folder, so we need to write it to the database
                    return Ok((self.children.is_empty(), to_remove)); // If we're empty now, signal our parent to remove us
                } else {
                    Ok(result)
                }
            }
        };
        
        self.children.insert(path_part.unwrap().to_string(), child);
        return output

    }

    fn get_name (&self) -> &str {
        &self.name
    }
}

impl File {

    fn new(real_path: String, name: String) -> File {
        File {
            real_path,
            name,
            _uuid: Uuid::new_v4().to_string()
        }
    }
    fn get_name (&self) -> &str {
        &self.name
    }

    fn to_db_file(&self) -> DbFile {
        DbFile {
            name: self.name.clone(),
            real_path: self.real_path.clone(),
            uuid: self._uuid.clone()
        }
    }

    fn from_db_file(db_file: DbFile) -> File {
        File {
            name: db_file.name,
            real_path: db_file.real_path,
            _uuid: db_file.uuid
        }
    }
}

