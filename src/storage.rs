use std::path::Path;
use std::{io::Result, path::PathBuf};
use std::fs;
use sled::Db;
use crate::locations::get_default_storage_dir;

pub(crate) struct StorageManager {
    _root_path: PathBuf,
    storage_db: Db,
}

impl StorageManager {
    pub(crate) fn get_manager() -> StorageManager {
        let default_storage_dir = get_default_storage_dir().unwrap();
        let db_location = default_storage_dir.join(".db");
        let db = sled::open(db_location).unwrap();
        StorageManager {
            _root_path: default_storage_dir,
            storage_db: db,
        }
    }

    pub(crate) fn add(&self, name: &str, collection: &str, endpoint: &str, path: PathBuf) -> Result<()> {
        let key = format!("{}/{}", name, collection);
        let value = format!("{}:{}", endpoint, path.to_str().unwrap());
        if  !path.exists() {
            fs::create_dir_all(&path)?;
        }
        if self.storage_db.contains_key(&key).unwrap() {
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Project already exists"));
        }
        self.storage_db.insert(key, value.as_bytes())?;
        Ok(())
    }

    pub(crate) fn get(&self, name: &str, collection: &str) -> Result<(String, PathBuf)> {
        let key = format!("{}/{}", name, collection);
        let value = self.storage_db.get(key).unwrap();
        let value = match value {
            None => {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Storage information not found for project {}/{}", collection, name)));
            },
            Some(value) => value,

        };

        let value = String::from_utf8(value.to_vec()).unwrap();
        let mut split = value.split(':');
        let endpoint = split.next().unwrap();
        let path = split.next().unwrap();
        let path = Path::new(path);
        Ok((endpoint.to_string(), path.to_path_buf()))
    }

    pub(crate) fn delete(&self, name: &str, collection: &str) -> Result<()> {
        let key = format!("{}/{}", name, collection);
        let path = self.get(name, collection)?;
        self.storage_db.remove(key)?;
        fs::remove_dir_all(&path.1)?;
        if path.1.parent().unwrap().read_dir()?.count() == 0 {
            fs::remove_dir(path.1.parent().unwrap())?;
        }
        Ok(())
    }
}

pub(crate) trait StorageEndpoint {
    // Represents a type of location data can be stored. For example, local disk,
    // a remote serve, etc...
    // Responsible for producing fully qualified paths to data, and checking that the
    // endpoint is actually available, providing sensible errors if not.

    // It is not actually responsible for reading or writing data. Since this is a 
    // library designed for loading and storing data in python, we leave the actual
    // reading and writing to python.



    fn generate_path(&self, project_path: &str) -> Result<PathBuf>;
    fn is_available(&self) -> Result<()>;
    fn discover_file(&self, project_path: &str, file_extension: String) -> Result<PathBuf>;
    fn move_file(&self, from: &str, to: &str) -> Result<()>;
    fn copy_file(&self, from: &str, to: &str) -> Result<()>;
    fn delete_file(&self, path: &str) -> Result<()>;
    fn is_internal(&self, path: &Path) -> bool;
    fn get_relative_path(&self, path: &Path) -> Result<PathBuf>;
    fn make_full_path(&self, relpath: &Path) -> PathBuf;

}

pub(crate) struct LocalEndpoint {
    // Represents a local disk location. 
    root_path: PathBuf,
}

impl LocalEndpoint {
    pub(crate) fn new(root_path: PathBuf) -> LocalEndpoint {
        LocalEndpoint {
            root_path,
        }
    }
}

impl StorageEndpoint for LocalEndpoint {
    fn generate_path(&self, project_path: &str) -> Result<PathBuf> {
        // Generate a path to a project. This is the path to the root of the project
        // on the local disk.
        let path = self.root_path.join(project_path);
        Ok(path)
    }

    fn is_internal(&self, path: &Path) -> bool {
        // Check if a path is internal to the project. This means that it is a path
        // that is not a symlink to a file outside the project.
        path.starts_with(&self.root_path)
    }

    fn is_available(&self) -> Result<()> {
        // Check that the local disk is available.
        Ok(())
    }

    fn discover_file(&self, project_path: &str, file_extension: String) -> Result<PathBuf> {
        let real_path = self.generate_path(project_path)?;
        let expected_file_path = real_path.with_extension(file_extension);
        if expected_file_path.exists() {
            return Ok(expected_file_path);
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
    }

    fn move_file(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.generate_path(from)?;
        let to_path = self.generate_path(to)?;
        // copy the file
        fs::rename(from_path, to_path)

    }
    fn copy_file(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.generate_path(from)?;
        let to_path = self.generate_path(to)?;
        fs::copy(from_path, to_path)?;
        Ok(())
    }
    fn delete_file(&self, path: &str) -> Result<()> {
        let real_path = self.generate_path(path)?;
        fs::remove_file(path)?;
        let parent_directory = real_path.parent().unwrap();
        if parent_directory.read_dir()?.count() == 0 {
            fs::remove_dir(parent_directory)?;
        }
        Ok(())
    }

    fn get_relative_path(&self, path: &Path) -> Result<PathBuf> {
        let result = path.strip_prefix(&self.root_path);
        
        match result {
            Ok(path) => Ok(path.to_path_buf()),
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path is not internal to project")),
        }
    }
    fn make_full_path(&self, relpath: &Path) -> PathBuf {
        
        self.root_path.join(relpath)
    }
}