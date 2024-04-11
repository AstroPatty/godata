use crate::errors::{GodataError, GodataErrorType, Result};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn get_main_dir() -> PathBuf {
    let base_dir: BaseDirs = BaseDirs::new().unwrap();
    let user_data_dir: &Path = base_dir.data_dir();
    let package_root: PathBuf = user_data_dir.join("godata");
    if !package_root.exists() {
        std::fs::create_dir_all(&package_root).unwrap();
    }
    package_root
}

fn create_collection_dir(name: &str) -> Result<PathBuf> {
    let main_directory = get_main_dir();
    let collection_path = main_directory.join(name);
    if !collection_path.exists() {
        std::fs::create_dir_all(&collection_path).unwrap();
        return Ok(collection_path);
    }

    Err(GodataError::new(
        GodataErrorType::AlreadyExists,
        format!("Collection {} already exists", name),
    ))
}

pub(crate) fn load_collection_dir(name: &str) -> Result<PathBuf> {
    let main_directory = get_main_dir();
    let collection_path = main_directory.join(name);
    if collection_path.exists() {
        return Ok(collection_path);
    }

    Err(GodataError::new(
        GodataErrorType::NotFound,
        format!("Collection {} does not exist", name),
    ))
}

fn delete_collection_dir(name: &str) -> Result<()> {
    let main_directory = get_main_dir();
    let collection_path = main_directory.join(name);
    if collection_path.exists() {
        std::fs::remove_dir_all(&collection_path)?;
        return Ok(());
    }

    Err(GodataError::new(
        GodataErrorType::NotFound,
        format!("Collection {} does not exist", name),
    ))
}

pub(crate) fn create_project_dir(
    name: &str,
    collection_name: &str,
    force: bool,
) -> Result<PathBuf> {
    let mut collection_dir = load_collection_dir(collection_name);
    if collection_dir.is_err() {
        if force {
            collection_dir = create_collection_dir(collection_name);
        } else {
            return Err(collection_dir.err().unwrap());
        }
    }
    let collection_dir = collection_dir.unwrap();

    let project_path = collection_dir.join(name);
    if !project_path.exists() {
        std::fs::create_dir_all(&project_path).unwrap();
        return Ok(project_path);
    }

    Err(GodataError::new(
        GodataErrorType::AlreadyExists,
        format!("Project {} already exists", name),
    ))
}

pub(crate) fn load_project_dir(name: &str, collection_name: &str) -> Result<PathBuf> {
    let collection_dir = load_collection_dir(collection_name)?;
    let project_path = collection_dir.join(name);
    if project_path.exists() {
        return Ok(project_path);
    }

    Err(GodataError::new(
        GodataErrorType::NotFound,
        format!("Project {} does not exist", name),
    ))
}

pub(crate) fn delete_project_dir(name: &str, collection_name: &str) -> Result<()> {
    let collection_dir = load_collection_dir(collection_name)?;
    let project_path = collection_dir.join(name);
    if project_path.exists() {
        std::fs::remove_dir_all(&project_path)?;
    } else {
        return Err(GodataError::new(
            GodataErrorType::NotFound,
            format!("Project {} does not exist", name),
        ));
    }
    // Check if this folder has any subdirectories
    for entry in fs::read_dir(&collection_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            return Ok(());
        }
    }
    // If not, delete the collection
    delete_collection_dir(collection_name)?;
    Ok(())
}

pub(crate) fn get_default_storage_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().unwrap();
    let home = base_dirs.home_dir();
    let main_dir = home.join("godata");
    if !main_dir.exists() {
        std::fs::create_dir_all(&main_dir).unwrap();
    }
    Ok(main_dir)
}

pub(crate) fn get_default_collection_storage_dir(collection_name: &str) -> Result<PathBuf> {
    let main_dir = get_default_storage_dir()?;
    let collection_dir = main_dir.join(collection_name);
    Ok(collection_dir)
}

pub(crate) fn get_default_project_storage_dir(
    name: &str,
    collection_name: &str,
) -> Result<PathBuf> {
    let collection_dir = get_default_collection_storage_dir(collection_name)?;
    let project_dir = collection_dir.join(name);
    Ok(project_dir)
}
