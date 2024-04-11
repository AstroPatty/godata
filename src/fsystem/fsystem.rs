// Definition of the virtual file system. Folders in the file system may be backed
// by real folders, or may be entirely virtual. Files in the file system are always
// backed by real files.

// As far as the rest of the library is concrened,

use sled::{Batch, Db};
use std::collections::HashMap;
use uuid::Uuid;

use ciborium::{from_reader, into_writer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::instrument;

use crate::fsystem::errors::{GodataError, GodataErrorType, Result};

#[derive(Clone)]
enum FSObject {
    File(File),
    Folder(Folder),
}
impl FSObject {
    fn get_name(&self) -> &str {
        match self {
            FSObject::File(f) => f.get_name(),
            FSObject::Folder(f) => f.get_name(),
        }
    }

    fn rename(&mut self, new_name: String) {
        match self {
            FSObject::File(f) => f.name = new_name,
            FSObject::Folder(f) => f.name = new_name,
        }
    }
}
#[derive(Clone)]
pub(crate) struct File {
    pub(crate) real_path: PathBuf,
    pub(crate) name: String,
    pub(crate) metadata: HashMap<String, String>,
    _uuid: String,
}
#[derive(Clone)]
struct Folder {
    pub(self) name: String,
    children: HashMap<String, FSObject>,
    metadata: HashMap<String, String>,
    _uuid: String,
    _modified: bool,
}

#[derive(Serialize, Deserialize)]
struct DbFolder {
    pub(self) name: String,
    folders_uuids: Vec<String>,
    files: Vec<DbFile>,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct DbFile {
    pub(self) name: String,
    real_path: String,
    uuid: String,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

pub(crate) struct FileSystem {
    root: Folder,
    _name: String,
    _modified: bool,
    db: Db,
}

enum RemoveResult {
    Item(FSObject),
    IsEmpty,
}

pub(crate) fn is_empty(path: &PathBuf) -> bool {
    let db = sled::open(path).unwrap();
    // Count the entries in the database
    let root_folder = db.get("root".as_bytes()).unwrap();
    // Deserialize the root folder
    let db_folder = from_reader(root_folder.unwrap().as_ref());
    let db_folder: DbFolder = db_folder.unwrap();
    // If there are any files or folders in the root folder, return false
    if db_folder.folders_uuids.len() > 0 || db_folder.files.len() > 0 {
        return false;
    }
    true
}

fn drain(mut folder: Folder) -> Vec<File> {
    // Consume the folder and return a list of all the files in the folder and its children
    let mut files: Vec<File> = Vec::new();
    for (_, child) in folder.children.drain() {
        match child {
            FSObject::File(f) => {
                files.push(f);
            }
            FSObject::Folder(f) => {
                let mut child_files = drain(f);
                files.append(&mut child_files);
            }
        }
    }
    files
}

impl FileSystem {
    #[instrument]
    pub(crate) fn new(name: String, root_path: PathBuf) -> Result<FileSystem> {
        let db = sled::open(&root_path); // If we can't open the database, we just fail

        let db = match db {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Sled failed to open database: {}", e);
                return Err(GodataError::new(
                    GodataErrorType::IOError,
                    format!("Failed to open database"),
                ));
            }
        };

        let root_folder = db.get("root".as_bytes())?;
        // If there is already a root folder, fail
        let root = match root_folder {
            None => Folder {
                name: "root".to_string(),
                children: HashMap::new(),
                metadata: HashMap::new(),
                _uuid: "root".to_string(),
                _modified: true,
            },
            Some(_) => {
                tracing::error!(
                    "Was able to create a new filesystem for project {} at path {}, but somehow the root folder already exists!",
                    name,
                    root_path.display()
                );
                return Err(GodataError::new(
                    GodataErrorType::AlreadyExists,
                    "File system already exists".to_string(),
                ));
            }
        };

        Ok(FileSystem {
            root,
            _name: name,
            _modified: true,
            db,
        })
    }

    #[instrument(skip(self))]
    pub(crate) fn export(
        &mut self,
    ) -> sled::Result<Vec<(Vec<u8>, Vec<u8>, impl Iterator<Item = Vec<Vec<u8>>>)>> {
        // Copy the database to the specified path
        self.save();
        self.db.flush()?;
        let res = self.db.export();
        tracing::info!("Serialized database for project {}", self._name);
        Ok(res)
    }

    pub(crate) fn load(name: &str, root_dir: PathBuf) -> Result<FileSystem> {
        let db = sled::open(&root_dir);
        let db = match db {
            Ok(db) => db,
            Err(e) => {
                tracing::error!(
                    "Sled failed to open database for project {} at path {}: {}",
                    name,
                    root_dir.display(),
                    e
                );
                return Err(GodataError::new(
                    GodataErrorType::IOError,
                    format!("Failed to open database"),
                ));
            }
        };
        let root_folder = db.get("root".as_bytes())?;
        // If there is no root folder, fail

        let root = match root_folder {
            None => {
                // get a list of the found folders
                tracing::error!(
                    "Was able to open the database for project {} at path {}, but no root folder was found!",
                    name,
                    root_dir.display()
                );
                return Err(GodataError::new(
                    GodataErrorType::NotFound,
                    "File system was opened, but no root folder was found".to_string(),
                ));
            }
            Some(_) => Folder::from_tree(&db, "root".to_string())?,
        };

        Ok(FileSystem {
            root,
            _modified: false,
            _name: name.to_string(),
            db,
        })
    }

    #[instrument(skip(self))]
    pub(crate) fn list(
        &self,
        virtual_path: Option<String>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let folder = match virtual_path {
            Some(path) => {
                let f_ = self.root.get(&path)?;
                match f_ {
                    FSObject::File(_) => {
                        tracing::info!("Path is a file!");
                        return Err(GodataError::new(
                            GodataErrorType::InvalidPath,
                            format!("Path {} is a file", path),
                        ));
                    }
                    FSObject::Folder(f) => f,
                }
            }
            None => &self.root,
        };
        let mut files = Vec::new();
        let mut folders = Vec::new();

        for (name, child) in folder.children.iter() {
            match child {
                FSObject::File(_) => files.push(name.clone()),
                FSObject::Folder(_) => folders.push(name.clone()),
            }
        }
        let mut children = HashMap::new();
        children.insert("folders".to_string(), folders);
        children.insert("files".to_string(), files);
        Ok(children)
    }

    #[instrument(skip(self))]
    pub(crate) fn get(&self, virtual_path: &str) -> Result<&File> {
        let file = self.root.get(virtual_path)?;
        match file {
            FSObject::Folder(_) => {
                tracing::info!("Path is a folder!");
                Err(GodataError::new(
                    GodataErrorType::InvalidPath,
                    "Path is a folder".into(),
                ))
            }
            FSObject::File(f) => Ok(f),
        }
    }

    pub(crate) fn insert(
        &mut self,
        project_path: &str,
        real_path: PathBuf,
        metadata: HashMap<String, String>,
        overwrite: bool,
    ) -> Result<Option<Vec<File>>> {
        let name = project_path.split('/').last().unwrap().to_string();
        let result = if name == project_path {
            let mut file = File::new(real_path, name);
            file.metadata = metadata;
            self.root.insert(FSObject::File(file), "", overwrite)?
        } else {
            let ppath = project_path
                .strip_suffix(format!("/{}", name).as_str())
                .unwrap();
            let mut file = File::new(real_path, name);
            file.metadata = metadata;
            self.root.insert(FSObject::File(file), ppath, overwrite)?
        };
        self._modified = true;
        self.save();
        Ok(result)
    }

    pub(crate) fn insert_many<I>(&mut self, files: I, virtual_path: &str) -> Result<()>
    where
        I: Iterator<Item = PathBuf>,
    {
        let file_objects = files.map(|path| {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            File::new(path, name)
        });
        self.root.insert_many(file_objects, virtual_path)?;
        self._modified = true;
        self.save();
        Ok(())
    }

    #[instrument(skip(self))]
    pub(crate) fn remove(&mut self, virtual_path: &str) -> Result<Vec<File>> {
        let result = self.root.delete(virtual_path)?;
        tracing::info!("Removed item at path {}, dropping from tree", virtual_path);
        let mut batch = Batch::default();
        let output = match result {
            RemoveResult::IsEmpty => {
                self.root.drop_from_tree(&mut batch)?;
                let mut files: Vec<File> = Vec::new();
                for (_, child) in self.root.children.drain() {
                    match child {
                        FSObject::File(f) => {
                            files.push(f);
                        }
                        FSObject::Folder(f) => {
                            let mut child_files = drain(f);
                            files.append(&mut child_files);
                        }
                    }
                }
                self.root.children.clear();
                files
            }
            RemoveResult::Item(f) => match f {
                FSObject::File(f) => {
                    vec![f]
                }
                FSObject::Folder(mut f) => {
                    f.drop_from_tree(&mut batch)?;
                    drain(f)
                }
            },
        };
        self.db.apply_batch(batch)?;
        self._modified = true;

        Ok(output)
    }

    #[instrument(skip(self))]
    pub(crate) fn move_(
        &mut self,
        source_path: &str,
        dest_path: &str,
        overwrite: bool,
    ) -> Result<Option<Vec<File>>> {
        if !self.root.exists(source_path) {
            tracing::info!("Source path does not exist");
            return Err(GodataError::new(
                GodataErrorType::NotFound,
                format!("Source path {} does not exist", source_path),
            ));
        }
        if self.root.exists(dest_path) && !overwrite {
            tracing::info!("Destination path already exists");
            return Err(GodataError::new(
                GodataErrorType::AlreadyExists,
                format!("Destination path {} already exists", dest_path),
            ));
        }
        let item = self.root.get(source_path)?;
        // HANDLE RENAME SEMANTICS
        // make a copy of the item
        let (fpath, fname) = dest_path.rsplit_once('/').unwrap_or(("", dest_path));
        let mut item = (*item).clone();
        item.rename(fname.to_string());
        // Split the destination path into path and name

        let result = self.root.insert(item, fpath, overwrite)?;
        self.remove(source_path)?;
        self._modified = true;
        self.save();
        Ok(result)
    }

    pub(crate) fn exists(&self, virtual_path: &str) -> bool {
        self.root.exists(virtual_path)
    }

    #[instrument(skip(self))]
    fn save(&mut self) -> Result<()> {
        // Write the root folder to the database
        tracing::info!("Saving filesystem for project {}", self._name);
        let mut batch = Batch::default();
        self.root.write_to_tree(&mut batch)?;
        self.db.apply_batch(batch)?;
        self.root.reset();
        self._modified = false;
        Ok(())
        // Batching and reseting like this ensures two things
        // First, bulk changes (like adding folders) will always go through in full
        // Second, The tree will only be unmodified if its changes are saved
    }
}

impl Drop for FileSystem {
    fn drop(&mut self) {
        self.save();
    }
}

impl Folder {
    fn new(name: String) -> Folder {
        Folder {
            name,
            children: HashMap::new(),
            metadata: HashMap::new(),
            _uuid: Uuid::new_v4().to_string(),
            _modified: true,
        }
    }

    fn reset(&mut self) {
        self._modified = false;
        for (_, child) in self.children.iter_mut() {
            match child {
                FSObject::File(_) => (),
                FSObject::Folder(f) => f.reset(),
            }
        }
    }
    #[instrument]
    fn from_tree(db: &Db, uuid: String) -> Result<Folder> {
        let folder_info = db.get(uuid.as_bytes());
        if folder_info.is_err() {
            tracing::error!(
                "Failed to read folder from database: {}",
                folder_info.err().unwrap()
            );
            return Err(GodataError::new(
                GodataErrorType::IOError,
                "Failed to read folder from database".to_string(),
            ));
        }
        let folder_info = folder_info.unwrap();
        if folder_info.is_none() {
            tracing::error!("Folder not found in database");
            return Err(GodataError::new(
                GodataErrorType::NotFound,
                "Folder not found".to_string(),
            ));
        }

        let db_folder: DbFolder = from_reader(folder_info.unwrap().as_ref()).unwrap();
        let mut children = HashMap::new();
        for fuuid in db_folder.folders_uuids {
            let folder = Folder::from_tree(db, fuuid)?;
            children.insert(folder.name.clone(), FSObject::Folder(folder));
        }

        for file in db_folder.files {
            children.insert(file.name.clone(), FSObject::File(File::from_db_file(file)));
        }

        Ok(Folder {
            name: db_folder.name,
            children,
            metadata: db_folder.metadata,
            _uuid: uuid,
            _modified: false,
        })
    }

    fn write_to_tree(&mut self, batch: &mut Batch) -> Result<()> {
        // Write the folder and all of its children to the database
        if self._modified {
            self.write_to_db(batch)?;
        }
        for (_, child) in self.children.iter_mut() {
            match child {
                FSObject::File(_) => (),
                FSObject::Folder(f) => f.write_to_tree(batch)?,
            }
        }
        Ok(())
    }

    fn drop_from_tree(&mut self, batch: &mut Batch) -> Result<()> {
        // Remove the folder and all of its children from the database
        batch.remove(self._uuid.as_bytes());
        for (_, child) in self.children.iter_mut() {
            if let FSObject::Folder(f) = child {
                f.drop_from_tree(batch)?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self, files))]
    fn insert_many<I>(&mut self, files: I, virtual_path: &str) -> Result<()>
    where
        I: Iterator<Item = File>,
    {
        let path_parts = virtual_path.split('/');
        self._insert_many(files, path_parts)
    }

    fn _insert_many<I>(&mut self, files: I, mut path_parts: std::str::Split<char>) -> Result<()>
    where
        I: Iterator<Item = File>,
    {
        let path_part = path_parts.next();
        let child = match path_part {
            None => {
                //We're at the end, try to insert it here
                return self._insert_all(files);
            }
            Some(part) => self.children.get_mut(part),
        };
        match child {
            Some(item) => {
                match item {
                    FSObject::File(_) => {
                        tracing::info!("Trying to insert into a file");
                        Err(GodataError::new(
                            GodataErrorType::InvalidPath,
                            "Path is a file".into(),
                        ))
                    } // We have a file with this name, and nothing is left in the path
                    FSObject::Folder(f) => f._insert_many(files, path_parts), // We have a folder with this name, and we need to check the rest of the path
                }
            }
            None => {
                let mut folder = Folder::new(path_part.unwrap().to_string());
                folder._insert_many(files, path_parts)?;
                self.children
                    .insert(folder.name.clone(), FSObject::Folder(folder));
                self._modified = true;
                Ok(())
            }
        }
    }

    fn _insert_all<I>(&mut self, files: I) -> Result<()>
    where
        I: Iterator<Item = File>,
    {
        for file in files {
            self.children
                .insert(file.name.clone(), FSObject::File(file));
        }
        self._modified = true;
        Ok(())
    }

    fn to_db_folder(&self) -> DbFolder {
        let mut folders_uuids = Vec::new();
        let mut files = Vec::new();
        for (_, child) in self.children.iter() {
            match child {
                FSObject::File(f) => files.push(f.to_db_file()),
                FSObject::Folder(f) => folders_uuids.push(f._uuid.clone()),
            }
        }
        DbFolder {
            name: self.name.clone(),
            folders_uuids,
            files,
            metadata: self.metadata.clone(),
        }
    }

    fn write_to_db(&mut self, batch: &mut Batch) -> Result<()> {
        let db_folder = self.to_db_folder();
        let mut bytes = Vec::new();
        let write_result = into_writer(&db_folder, &mut bytes);
        if write_result.is_err() {
            tracing::error!(
                "Failed to serialize folder {} to bytes: {}",
                self.name,
                write_result.err().unwrap()
            );
            return Err(GodataError::new(
                GodataErrorType::IOError,
                "Failed to serialize folder".to_string(),
            ));
        }
        batch.insert(self._uuid.as_bytes(), bytes);
        Ok(())
    }

    fn exists(&self, virtual_path: &str) -> bool {
        // Check if a file or folder exists in the folder.
        // If path is this folder's name, return true
        // If path is a subfolder, return true if it exists in the subfolder

        // split up the path
        let path_parts = virtual_path.split('/');
        self._exists(path_parts)
    }

    fn _exists(&self, mut path_parts: std::str::Split<char>) -> bool {
        // Check if a file or folder exists in the folder.
        // If path is this folder's name, return true
        // If path is a subfolder, return true if it exists in the subfolder

        let path_part = path_parts.next();
        let child = match path_part {
            None => return true, //We're in a folder and there's nowhere left to go, it exists
            Some(part) => self.children.get(part),
        };

        match child {
            None => false, // child doesn't exist
            Some(f) => {
                match f {
                    FSObject::File(_) => path_parts.next().is_none(), // We have a file with this name, and nothing is left in the path
                    FSObject::Folder(f) => f._exists(path_parts), // We have a folder with this name, and we need to check the rest of the path
                }
            }
        }
    }

    #[instrument(skip(self))]
    fn get(&self, virtual_path: &str) -> Result<&FSObject> {
        // Get a file from the folder. Will fail if there is no file located
        // at the virtual path.

        // split up the path
        let path_parts = virtual_path.split('/');
        let path: Vec<&str> = path_parts.collect();
        let result = self._get(&path);
        if result.is_err() {
            let mut err = result.err().unwrap();
            err.message = format!("Failed to get path {}: {}", virtual_path, err.message);
            return Err(err);
        }
        result
    }

    fn _get(&self, path_parts: &[&str]) -> Result<&FSObject> {
        // Get a file or folder from the folder.
        // If path is this folder's name, return it
        // If path is a subfolder, return it from the subfolder

        let path_part = path_parts.first();
        let child = match path_part {
            None => {
                tracing::error!("Path part is none!");
                return Err(GodataError::new(
                    GodataErrorType::InternalError,
                    "Invalid path part".to_string(),
                ));
            }
            Some(&part) => self.children.get(part),
        };

        if child.is_none() {
            let msg = format!(
                "Child {} does not exist in folder {}",
                path_part.unwrap(),
                self.name
            );
            tracing::info!(msg);
            Err(GodataError::new(GodataErrorType::NotFound, msg))
        } else if path_parts.len() == 1 {
            return Ok(child.unwrap());
        } else {
            match child.unwrap() {
                FSObject::File(_) => {
                    let msg = format!(
                        "Child {} of folder {} is a file",
                        path_part.unwrap(),
                        self.name
                    );
                    tracing::info!(msg);
                    return Err(GodataError::new(GodataErrorType::NotFound, msg));
                }
                FSObject::Folder(f) => {
                    return f._get(&path_parts[1..]);
                }
            }
        }
    }

    #[instrument(skip(self, fs_object))]
    fn insert(
        &mut self,
        fs_object: FSObject,
        virtual_path: &str,
        overwrite: bool,
    ) -> Result<Option<Vec<File>>> {
        // Insert a file or folder into the folder.
        // If path is this folder's name, insert it here
        // If path is a subfolder, insert it into the subfolder

        // split up the path
        let mut path_parts = virtual_path.split('/');
        if virtual_path == "" {
            // go to the end of the iterator
            _ = path_parts.next();
        }
        let insert_result = self._insert(fs_object, path_parts, overwrite);
        if insert_result.is_err() {
            let mut err = insert_result.err().unwrap();
            err.message = format!("Failed to insert at path {}: {}", virtual_path, err.message);
            return Err(err);
        }
        insert_result
    }

    fn _insert(
        &mut self,
        fs_object: FSObject,
        mut path_parts: std::str::Split<char>,
        overwrite: bool,
    ) -> Result<Option<Vec<File>>> {
        // Insert a file or folder into the folder.
        // If path is this folder's name, insert it here
        // If path is a subfolder, insert it into the subfolder

        // split up the path
        let path_part = path_parts.next();
        let child = match path_part {
            None => {
                //We're at the end, try to insert it here
                if self.children.contains_key(fs_object.get_name()) {
                    if !overwrite {
                        tracing::info!("Path already exists");
                        return Err(GodataError::new(
                            GodataErrorType::AlreadyExists,
                            "Something already exists at that path!".to_string(),
                        ));
                    } else {
                        let previous = self.children.remove(fs_object.get_name()).unwrap();
                        self.children
                            .insert(fs_object.get_name().to_string(), fs_object);
                        self._modified = true;
                        let output = match previous {
                            FSObject::File(f) => Some(vec![f]),
                            FSObject::Folder(f) => {
                                let files = drain(f);
                                Some(files)
                            }
                        };
                        return Ok(output);
                    }
                } else {
                    self.children
                        .insert(fs_object.get_name().to_string(), fs_object);
                    self._modified = true;
                    return Ok(None);
                }
            }
            Some(part) => self.children.get_mut(part),
        };

        match child {
            None => {
                // child doesn't exist, create it
                tracing::info!("Creating new folder {}", path_part.unwrap());
                let mut folder = Folder::new(path_part.unwrap().to_string());
                folder._insert(fs_object, path_parts, overwrite).unwrap();
                self.children
                    .insert(folder.name.clone(), FSObject::Folder(folder));
                self._modified = true;
                Ok(None)
            }
            Some(f) => {
                match f {
                    FSObject::File(_) => {
                        let msg = format!(
                            "Child {} of folder {} is a file",
                            path_part.unwrap(),
                            self.name
                        );
                        tracing::info!(msg);
                        Err(GodataError::new(GodataErrorType::AlreadyExists, msg))
                    } // We have a file with this name, and nothing is left in the path
                    FSObject::Folder(f) => f._insert(fs_object, path_parts, overwrite), // We have a folder with this name, and we need to check the rest of the path
                }
            }
        }
    }

    #[instrument(skip(self))]
    fn delete(&mut self, virtual_path: &str) -> Result<RemoveResult> {
        // Delete a file or folder from the folder.
        // If path is this folder's name, delete it here
        // If path is a subfolder, delete it from the subfolder
        // This function will only every be called directly on the root folder

        // split up the path
        let path: Vec<&str> = virtual_path.split('/').collect();
        if path.len() == 0 {
            return Err(GodataError::new(
                GodataErrorType::InvalidPath,
                "Root folder cannot be removed!".to_string(),
            ));
        }
        let delete_result = self._delete(&path);
        if delete_result.is_err() {
            let mut err = delete_result.err().unwrap();
            err.message = format!("Failed to delete path {}: {}", virtual_path, err.message);
            return Err(err);
        }
        Ok(delete_result.unwrap())
    }

    fn _delete(&mut self, path: &[&str]) -> Result<RemoveResult> {
        // Delete a file or folder from the folder.
        // If path is this folder's name, delete it here
        // If path is a subfolder, delete it from the subfolder

        // split up the path
        let path_part = path.first();
        if path_part.is_none() {
            tracing::error!("Path part is none!");
            return Err(GodataError::new(
                GodataErrorType::InternalError,
                "Unable to delete path".to_string(),
            ));
        }
        let path_part = path_part.unwrap();
        if !self.children.contains_key(*path_part) {
            let msg = format!("Child {} does not exist in folder {}", path_part, self.name);
            tracing::info!(msg);
            return Err(GodataError::new(GodataErrorType::NotFound, msg));
        }
        if path.len() == 1 {
            self._modified = true;
            if self.children.len() == 1 {
                return Ok(RemoveResult::IsEmpty);
            }
            return Ok(RemoveResult::Item(
                self.children.remove(*path_part).unwrap(),
            ));
        }
        match self.children.get_mut(*path_part).unwrap() {
            FSObject::File(_) => {
                let msg = format!("Child {} of folder {} is a file", path_part, self.name);
                tracing::info!(msg);
                Err(GodataError::new(GodataErrorType::InvalidPath, msg))
            }
            FSObject::Folder(f) => {
                let rm_result = f._delete(&path[1..])?;
                match rm_result {
                    RemoveResult::IsEmpty => {
                        if self.children.len() == 1 {
                            return Ok(RemoveResult::IsEmpty);
                        }
                        Ok(RemoveResult::Item(
                            self.children.remove(*path_part).unwrap(),
                        ))
                    }
                    RemoveResult::Item(_) => Ok(rm_result),
                }
            }
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl File {
    fn new(real_path: PathBuf, name: String) -> File {
        File {
            real_path,
            name,
            metadata: HashMap::new(),
            _uuid: Uuid::new_v4().to_string(),
        }
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn to_db_file(&self) -> DbFile {
        DbFile {
            name: self.name.clone(),
            real_path: self.real_path.to_str().unwrap().to_string(),
            metadata: self.metadata.clone(),
            uuid: self._uuid.clone(),
        }
    }

    fn from_db_file(db_file: DbFile) -> File {
        File {
            name: db_file.name,
            real_path: PathBuf::from(db_file.real_path),
            metadata: db_file.metadata,
            _uuid: db_file.uuid,
        }
    }
}
