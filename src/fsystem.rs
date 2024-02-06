// Definition of the virtual file system. Folders in the file system may be backed
// by real folders, or may be entirely virtual. Files in the file system are always
// backed by real files.

// As far as the rest of the library is concrened,

use sled::{Batch, Db};
use std::collections::HashMap;
use std::io::Result;
use uuid::Uuid;

use ciborium::{from_reader, into_writer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
            FSObject::Folder(mut f) => {
                let mut child_files = drain(f);
                files.append(&mut child_files);
            }
        }
    }
    files
}

impl FileSystem {
    pub(crate) fn new(name: String, root_path: PathBuf) -> Result<FileSystem> {
        let db = sled::open(root_path)?;
        let root_folder = db.get("root".as_bytes()).unwrap();
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
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "File system already exists",
                ))
            }
        };

        Ok(FileSystem {
            root,
            _name: name,
            _modified: true,
            db,
        })
    }
    pub(crate) fn export(
        &mut self,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>, impl Iterator<Item = Vec<Vec<u8>>>)>> {
        // Copy the database to the specified path
        self.save();
        self.db.flush()?;
        let res = self.db.export();
        Ok(res)
    }

    pub(crate) fn load(name: &str, root_dir: PathBuf) -> Result<FileSystem> {
        let db = sled::open(root_dir)?;
        let root_folder = db.get("root".as_bytes()).unwrap();
        // If there is no root folder, fail

        let root = match root_folder {
            None => {
                // get a list of the found folders
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File system was opened, but no root folder was found".to_string(),
                ));
            }
            Some(_) => Folder::from_tree(&db, "root".to_string()),
        };

        Ok(FileSystem {
            root,
            _modified: false,
            _name: name.to_string(),
            db,
        })
    }

    pub(crate) fn list(
        &self,
        virtual_path: Option<String>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let folder = match virtual_path {
            Some(path) => {
                let f_ = self.root.get(&path)?;
                match f_ {
                    FSObject::File(_) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Path is a file",
                        ))
                    }
                    FSObject::Folder(f) => f,
                }

            },
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

    pub(crate) fn get(
        &self,
        virtual_path: &str,
    ) -> Result<&File> {
        let file = self.root.get(virtual_path)?;
        match file {
            FSObject::Folder(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path is a folder",
                ))
            }
            FSObject::File(f) => {
                return Ok(f);
            },
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

    pub(crate) fn insert_many<I>(
        &mut self,
        files: I,
        virtual_path: &str,
    ) -> Result<()>
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

    pub(crate) fn remove(&mut self, virtual_path: &str) -> Result<Vec<File>> {
        let result = self.root.delete(virtual_path)?;
        let mut batch = Batch::default();
        let output = match result {
            RemoveResult::IsEmpty => {
                self.root.drop_from_tree(&mut batch); 
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
            RemoveResult::Item(f) => {
                match f {
                    FSObject::File(f) => {
                        vec![f]
                    },
                    FSObject::Folder(mut f) => {
                        f.drop_from_tree(&mut batch);
                        drain(f)
                    }
                }
            }
        };
        self.db.apply_batch(batch).unwrap();
        self._modified = true;

        Ok(output)
    }

    pub(crate) fn move_(&mut self, source_path: &str, dest_path: &str, overwrite: bool) -> Result<Option<Vec<File>>> {
        if !self.root.exists(source_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Source path does not exist",
            ));
        }
        if self.root.exists(dest_path) && !overwrite {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Destination path already exists",
            ));
        }
        let item = self.root.get(source_path)?;
        // make a copy of the item
        let item = (*item).clone();
        let result = self.root.insert(item, dest_path, overwrite)?;
        self.remove(source_path)?;
        self._modified = true;
        self.save();
        Ok(result)

    }

    pub(crate) fn exists(&self, virtual_path: &str) -> bool {
        self.root.exists(virtual_path)
    }

    fn save(&mut self) {
        // Write the root folder to the database
        let mut batch = Batch::default();
        self.root.write_to_tree(&mut batch);
        self.db.apply_batch(batch).unwrap();
        self.root.reset();
        self._modified = false;
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

    fn from_tree(db: &Db, uuid: String) -> Folder {
        let folder_info = db.get(uuid.as_bytes()).unwrap();

        let db_folder: DbFolder = from_reader(folder_info.unwrap().as_ref()).unwrap();
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
            metadata: db_folder.metadata,
            _uuid: uuid,
            _modified: false,
        }
    }

    fn write_to_tree(&mut self, batch: &mut Batch) {
        // Write the folder and all of its children to the database
        if self._modified {
            self.write_to_db(batch).unwrap();
        }
        for (_, child) in self.children.iter_mut() {
            match child {
                FSObject::File(_) => (),
                FSObject::Folder(f) => f.write_to_tree(batch),
            }
        }
    }

    fn drop_from_tree(&mut self, batch: &mut Batch) {
        // Remove the folder and all of its children from the database
        batch.remove(self._uuid.as_bytes());
        for (_, child) in self.children.iter_mut() {
            if let FSObject::Folder(f) = child {
                f.drop_from_tree(batch);
            }   
        }    
    }


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
                    FSObject::File(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "Path is a file",
                    )), // We have a file with this name, and nothing is left in the path
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
        into_writer(&db_folder, &mut bytes).unwrap();
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

    fn get(&self, virtual_path: &str) -> Result<&FSObject> {
        // Get a file from the folder. Will fail if there is no file located
        // at the virtual path.

        // split up the path
        let path_parts = virtual_path.split('/');
        let path: Vec<&str> = path_parts.collect();
        self._get(&path)
    }

    fn _get(&self, path_parts: &[&str]) -> Result<&FSObject> {
        // Get a file or folder from the folder.
        // If path is this folder's name, return it
        // If path is a subfolder, return it from the subfolder

        let path_part = path_parts.first();
        let child = match path_part {
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ))
            }
            Some(&part) => self.children.get(part),
        };

        if child.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ));
        }

        else if path_parts.len() == 1 {
            return Ok(child.unwrap());
        }

        else {
            match child.unwrap() {
                FSObject::File(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    ))
                }
                FSObject::Folder(f) => {
                    return f._get(&path_parts[1..]);
                }
            }

        }

    }

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
        let path_parts = virtual_path.split('/');
        self._insert(fs_object, path_parts, overwrite)
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
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            "Something already exists at that path!",
                        ));
                    } else {
                       let previous = self.children.remove(fs_object.get_name()).unwrap();
                        self.children
                            .insert(fs_object.get_name().to_string(), fs_object);
                        self._modified = true;
                        let output = match previous {
                            FSObject::File(f) => Some(vec![f]),
                            FSObject::Folder(f) => {
                                let mut files = drain(f);
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
                let mut folder = Folder::new(path_part.unwrap().to_string());
                folder._insert(fs_object, path_parts, overwrite).unwrap();
                self.children
                    .insert(folder.name.clone(), FSObject::Folder(folder));
                self._modified = true;
                Ok(None)
            }
            Some(f) => {
                match f {
                    FSObject::File(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "Invalid path",
                    )), // We have a file with this name, and nothing is left in the path
                    FSObject::Folder(f) => f._insert(fs_object, path_parts, overwrite), // We have a folder with this name, and we need to check the rest of the path
                }
            }
        }
    }

    fn delete(&mut self, virtual_path: &str) -> Result<RemoveResult> {
        // Delete a file or folder from the folder.
        // If path is this folder's name, delete it here
        // If path is a subfolder, delete it from the subfolder

        // split up the path
        let path: Vec<&str> = virtual_path.split('/').collect();
        self._delete(&path)
    }

    fn _delete(&mut self, path: &[&str]) -> Result<RemoveResult> {
        // Delete a file or folder from the folder.
        // If path is this folder's name, delete it here
        // If path is a subfolder, delete it from the subfolder

        // split up the path
        let path_part = path.first();
        if path_part.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid path",
            ));
        }
        let path_part = path_part.unwrap();
        if !self.children.contains_key(*path_part) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ));
        }
        if path.len() == 1 {
            self._modified = true;
            if self.children.len() == 1 {
                return Ok(RemoveResult::IsEmpty);
            }
            return Ok(RemoveResult::Item(self.children.remove(*path_part).unwrap()));
        }
        match self.children.get_mut(*path_part).unwrap() {
            FSObject::File(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid path",
                ))
            }
            FSObject::Folder(f) => {
                let rm_result = f._delete(&path[1..])?;
                match rm_result {
                    RemoveResult::IsEmpty => {
                        if self.children.len() == 1 {
                            return Ok(RemoveResult::IsEmpty);
                        }
                        return Ok(RemoveResult::Item(self.children.remove(*path_part).unwrap()));
                    }
                    RemoveResult::Item(_) => {
                        return Ok(rm_result);
                    }
                    
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
