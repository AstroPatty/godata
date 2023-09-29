use crate::pdb::{FolderDocument, FileDocument, ProjectFileSystemManager, FileSystemObject};
use crate::mdb::{ProjectError, Result, get_dirs};
use std::borrow::BorrowMut;
use std::sync::{Arc};
use std::cell::{RefCell, Ref};
use std::path::PathBuf;
use std::fs;

#[derive(Clone)]
pub(crate) enum FileTreeObject {
    Folder(FileTreeFolder),
    File(FileTreeFile),
}

impl FileTreeObject {
    pub(crate) fn get_name(&self) -> &str {
        match self {
            FileTreeObject::Folder(f) => &f.cfg.name,
            FileTreeObject::File(f) => &f.cfg.name,
        }
    }
    pub(crate) fn get_config(&self) -> FileSystemObject {
        match self {
            FileTreeObject::Folder(f) => FileSystemObject::Folder(f.cfg.clone()),
            FileTreeObject::File(f) => FileSystemObject::File(f.cfg.clone()),
        }
    }
    
    pub(crate) fn get_path(&self) -> PathBuf {
        match self {
            FileTreeObject::Folder(f) => f.cfg.location.clone(),
            FileTreeObject::File(f) => f.cfg.location.clone(),
        }
    }
}
pub(crate) struct FileTree {
    root: FileTreeFolder,
    mgr: ProjectFileSystemManager,
}

#[derive(Clone)]
pub(crate) struct FileTreeFolder {
    pub(super) cfg: FolderDocument,
    _children: RefCell<Vec<FileTreeObject>>,
    _child_records: Vec<FileSystemObject>,
}

#[derive(Clone)]
pub(crate) struct FileTreeFile {
    pub(super) cfg: FileDocument
}

impl FileTree {
    pub(crate) fn new_from_db(mgr: ProjectFileSystemManager) -> FileTree {
        let root = mgr.get_root();
        let root_node = FileTreeFolder::new_from_record(root, &mgr);
        FileTree {
            mgr: mgr,
            root: root_node,
        }
    }

    pub(crate) fn query(&mut self, path: &str) -> Result<FileTreeObject> {
        let split = path.split("/").collect::<Vec<&str>>();
        self.root.query(&split, &self.mgr)
    } 

    pub(crate) fn exists(&mut self, project_path: &str) -> bool {
        return self.query(project_path).is_ok();
    } 

    /// This function is used to store a python object in the project's internal storage. 
    /// It takes in a project path and a suffix, and returns a path to the location the object
    /// should be stored. In case of failure, it is better for the tree to have a reference to a 
    /// file that does not exist, then for a file to exist that we have no reference to.
    pub(crate) fn store(&mut self, project_path: &str, recursive: bool, suffix: &str) -> Result<PathBuf> {
        let split_project_path = project_path
                                    .strip_suffix("/")
                                    .unwrap_or(project_path)
                                    .split("/")
                                    .collect::<Vec<&str>>();
        let project_storage_directory = get_dirs()
                                    .get("data_dir")
                                    .unwrap()
                                    .join(&self.root.cfg.uuid);
        let uuid = nanoid::nanoid!();
        let path: PathBuf;
        let parent_uuid: String;
        if split_project_path.len() == 1 {
            parent_uuid = self.root.cfg.uuid.to_string(); // Store in root directory
            path = PathBuf::from(project_storage_directory.join(&uuid).with_extension(suffix).to_str().unwrap());
        }
        else {
            if !self.exists(project_path) {
                if !recursive {
                    return Err(ProjectError {msg: "Path does not exist".to_string()})
                }
                self.add_folder(&split_project_path[0..&split_project_path.len() - 1], true)?;
            }
            let parent_folder_ = self.root.query(&split_project_path[0..&split_project_path.len() - 1], &self.mgr)?;
            match parent_folder_ {
                FileTreeObject::File(_) => {
                    return Err(ProjectError {msg: "Path is a file".to_string()})
                }
                FileTreeObject::Folder(f) => {
                    path = f.cfg.location.join(&uuid).with_extension(suffix);
                    parent_uuid = f.cfg.uuid.to_string();
                }
            }
        }
        if !path.parent().unwrap().exists() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
        }

        let new_file = FileTreeFile {
            cfg: FileDocument {
                name: split_project_path[split_project_path.len() - 1].to_string(),
                uuid: uuid,
                parent: parent_uuid.to_string(),
                location: path.clone(),
            }
        };
        self.mgr.add(&FileSystemObject::File(new_file.cfg.clone()))?;
        self.root.insert(&split_project_path, FileTreeObject::File(new_file), &self.mgr)?;
        Ok(path)
        
    }

    pub(crate) fn add_file(&mut self, path: PathBuf, project_path: &str, resursive: bool) -> Result<()> {
        let split_project_path = project_path
                                    .strip_suffix("/")
                                    .unwrap_or(project_path)
                                    .split("/")
                                    .collect::<Vec<&str>>();
        if split_project_path.len() == 1 {
            let uuid = nanoid::nanoid!();
            let new_file = FileTreeFile {
                cfg: FileDocument {
                    name: split_project_path[0].to_string(),
                    uuid: uuid,
                    parent: self.root.cfg.uuid.clone(),
                    location: path,
                }
            };
            self.mgr.add(&FileSystemObject::File(new_file.cfg.clone()))?;
            self.root.insert(&split_project_path, FileTreeObject::File(new_file), &self.mgr)?;
            return Ok(())
        }
        let parent_folder =  match self.root.query(&split_project_path[0..split_project_path.len() - 1], &self.mgr) {
            Ok(f) => f,
            Err(e) => {
                if resursive {
                    self.add_folder(&split_project_path[0..split_project_path.len() - 1], true)?;

                    self.root.query(&split_project_path[0..split_project_path.len() - 1], &self.mgr)?

                } else {
                    return Err(e)
                }
            }
        };


        match parent_folder {
            FileTreeObject::File(_f) => {
                return Err(ProjectError {msg: "Path is a file".to_string()})
            }
            FileTreeObject::Folder(f) => {
                let uuid = nanoid::nanoid!();
                let new_file = FileTreeFile {
                    cfg: FileDocument {
                        name: split_project_path[split_project_path.len() - 1].to_string(),
                        uuid: uuid,
                        parent: f.cfg.uuid.clone(),
                        location: path,
                    }
                };
                self.mgr.add(&FileSystemObject::File(new_file.cfg.clone()))?;
                self.root.insert(&split_project_path, FileTreeObject::File(new_file), &self.mgr)?; //TODO: This is a bit of a hack, but it works for now
                return Ok(())
            }
        }
    }
    fn add_folder(&mut self, split_project_path: &[&str], recursive: bool) -> Result<()> {
        if split_project_path.len() == 1 { //We're adding to the root folder
            let child_name = split_project_path[0];
            let children = self.root.get_child_names(&self.mgr);
            if children.contains(&child_name.to_string()) {
                return Err(ProjectError {msg: "Path already exists".to_string()})
            }
            let uuid = nanoid::nanoid!();
            let new_folder = FileTreeFolder {
                cfg: FolderDocument {
                    name: split_project_path[0].to_string(),
                    location: self.root.cfg.location.clone().join(&uuid),
                    uuid: uuid,
                    children: Vec::new(),
                    parent: Some(self.root.cfg.uuid.clone()),
                },
                _children: RefCell::new(Vec::new()),
                _child_records: Vec::new(),
            };
            self.mgr.add(&FileSystemObject::Folder(new_folder.cfg.clone()))?;
            self.root.insert(split_project_path, FileTreeObject::Folder(new_folder), &self.mgr)?;
            return Ok(());
        }

        let parent_path = &split_project_path[0..split_project_path.len()-1];
        let parent = self.root.query(&parent_path, &self.mgr);

        match parent {
            Ok(p) => { //The parent already exists!
                match p {
                    FileTreeObject::File(_f) => {
                        return Err(ProjectError {msg: format!("Path {} is a file", &parent_path.join("/"))})
                    }
                    FileTreeObject::Folder(f) => {
                        let children = f.get_child_names(&self.mgr);
                        let child_name = split_project_path[split_project_path.len() - 1].to_string();
                        if children.contains(&child_name) {
                            return Err(ProjectError {msg: format!("Path {} already exists", &split_project_path.join("/"))})
                        }
                        let uuid = nanoid::nanoid!();
                        let new_folder = FileTreeFolder {
                            cfg: FolderDocument {
                                name: split_project_path[split_project_path.len() - 1].to_string(),
                                location: self.root.cfg.location.join(&uuid),
                                uuid: uuid,
                                children: Vec::new(),
                                parent: Some(f.cfg.uuid.clone()),
                            },
                            _children: RefCell::new(Vec::new()),
                            _child_records: Vec::new(),
                        };
                        self.mgr.add(&FileSystemObject::Folder(new_folder.cfg.clone()))?;
                        self.root.insert(split_project_path, FileTreeObject::Folder(new_folder), &self.mgr)?;
                        return Ok(())
                    }
                }
            }
            Err(_) => { //The parent folder does NOT exist
                if !recursive {
                    return Err(ProjectError {msg: format!("Path {} does not exist", &parent_path.join("/"))})
                }
                self.add_folder(parent_path, true)?;
                self.add_folder(split_project_path, true)?;
                return Ok(())
            }
        }
    }
    


    pub(crate) fn remove(&mut self, project_path: &str, recursive: bool) -> Result<FileSystemObject> {
        let split_project_path = project_path
                                    .strip_suffix("/")
                                    .unwrap_or(project_path)
                                    .split("/")
                                    .collect::<Vec<&str>>();
        let doc = self.root.remove(&split_project_path, recursive, &self.mgr);
        match doc {
            Ok(d) => {
                let cfg = d.get_config();
                self.mgr.remove(&cfg)?;
                Ok(cfg)
            },
            Err(e) => Err(ProjectError{msg: e.msg})
        }
    }
    pub(crate) fn get_contents(&self, path: Option<&str>) -> Result<Vec<FileSystemObject>> {
        let split = match path {
            Some(p) => p.split("/").collect::<Vec<&str>>(),
            None => vec![]
        };
        return self.root.get_contents(&split, &self.mgr)
    }
}


impl FileTreeFolder {
    fn new_from_record(cfg: FolderDocument, mgr: &ProjectFileSystemManager) -> FileTreeFolder {
        let children_records = mgr.get_child_records(&cfg).unwrap();
        let children_nodes = Vec::new();
        FileTreeFolder {
            cfg: cfg,
            _children: RefCell::new(children_nodes),
            _child_records: children_records,
        }
    }
    fn is_empty(&self) -> bool {
        self._child_records.len() == 0 && self._children.borrow().len() == 0
    }
    fn get_contents(&self, project_path: &[&str], mgr: &ProjectFileSystemManager) -> Result<Vec<FileSystemObject>> {
        self.load_children(mgr);
        if project_path.len() == 0 {
            let children = self._children.borrow();
            let mut children_records = Vec::new();
            for child in children.iter() {
                children_records.push(child.get_config());
            }
            return Ok(children_records)
        }
        let child_index = self._children.borrow().iter().position(|x| x.get_name() == project_path[0]);
        if child_index.is_none() {
            return Err(ProjectError {msg: "Path not found".to_string()})
        }
        let child_index = child_index.unwrap().clone();
        let children = self._children.borrow();
        let c = children.get(child_index).unwrap();
        match c {
            FileTreeObject::File(_f) => {
                return Err(ProjectError {msg: "Path not found".to_string()})
            }
            FileTreeObject::Folder(f) => {
                let sub_path = &project_path[1..project_path.len()];
                return f.get_contents(sub_path, mgr)
            }
        };        

    }
    
    fn load_children(&self, mgr: &ProjectFileSystemManager) {
        if self._child_records.len() == 0 || self._children.borrow().len() > 0 {
            return
        }

        for child in &self._child_records {
            match child {
                FileSystemObject::File(f) => {
                    let file_obj = FileTreeFile {
                        cfg: f.clone()
                    };
                    self._children.borrow_mut().push(FileTreeObject::File(file_obj));
                }
                FileSystemObject::Folder(f) => {
                    let folder_obj = FileTreeFolder::new_from_record(f.clone(), mgr);
                    self._children.borrow_mut().push(FileTreeObject::Folder(folder_obj));
                }
            }
        }



    }
    
    pub(crate) fn get_children(&self, mgr: &ProjectFileSystemManager) -> Vec<FileSystemObject> {
        self.load_children(mgr);
        let mut children = Vec::new();
        for child in self._children.borrow().iter() {
            children.push(child.get_config());
        }
        children
    }

    pub(crate) fn get_child_names(&self, mgr: &ProjectFileSystemManager) -> Vec<String> {
        self.load_children(mgr);
        let mut names = Vec::new();
        for child in self._children.borrow().iter() {
            names.push(child.get_name().to_string());
        }
        names
    }

    pub(crate) fn query(&self, query_path: &[&str], mgr: &ProjectFileSystemManager) -> Result<FileTreeObject> {
        self.load_children(mgr);
        let child_index = self._children.borrow().iter().position(|x| x.get_name() == query_path[0]);
        if child_index.is_none() {
            return Err(ProjectError {msg: "Path not found".to_string()})
        }
        let child_index = child_index.unwrap().clone();
        let children = self._children.borrow();
        let c = children.get(child_index).unwrap();
        if query_path.len() == 1 {
            // We're at the end of the query path and we have a match
            return Ok(c.clone())
        }
        match c {
            FileTreeObject::File(_f) => {
                // We're not at the end of the query path, but we've found
                // a file
                return Err(ProjectError { msg: "Path not found".to_string()})
            }
            FileTreeObject::Folder(ref f) => {
                // We're not at the end of the query path, but we cound a folder
                let sub_path = &query_path[1..query_path.len()];
                return f.query(sub_path, mgr)
            }
        };
    }

    pub(crate) fn insert(&mut self, path: &[&str], obj: FileTreeObject, mgr: &ProjectFileSystemManager) -> Result<()> {
        self.load_children(mgr);
        let child_names = self.get_child_names(mgr);
        let child_index = child_names.iter().position(|x| x == path[0]);

        if path.len() == 1 && child_index.is_none() {
            // Base case, we have the last already-extant folder
            self._children.borrow_mut().push(obj);
            return Ok(())
        }
        // Recursive case, we need to find the next folder
        else if path.len() > 1 && child_index.is_some() {
            let mut children = self._children.borrow_mut();
            let child = children.get_mut(child_index.unwrap()).unwrap();
            match child {
                FileTreeObject::File(_f) => {
                    return Err(ProjectError {msg: "Path not found".to_string()})
                }
                FileTreeObject::Folder(f) => {
                    return f.insert(&path[1..path.len()], obj, mgr)
                }
            }
        }
        else {
            return Err(ProjectError {msg: "Path not found".to_string()})
        }

    }
    pub(crate) fn remove(&mut self, path: &[&str], recursive: bool, mgr: &ProjectFileSystemManager) -> Result<FileTreeObject> {
        let children = self._children.get_mut();
        if path.len() == 1 {
            //check children for path[0]
            for (i, child) in children.iter().enumerate() {
                if child.get_name() == path[0] {
                    match child {
                        FileTreeObject::File(_f) => {}
                        FileTreeObject::Folder(f) => {
                            if !recursive && !f.is_empty() {
                                return Err(ProjectError {msg: "Path is a folder which contains items".to_string()})
                            }
                        }
                    }
                    return Ok(children.remove(i))
                }
            }
            return Err(ProjectError {msg: "Path not found".to_string()})


        }

        else {
            for child in children.iter_mut() {

                if child.get_name() == path[0] {
                    match child {
                        FileTreeObject::File(_f) => {
                            return Err(ProjectError {msg: format!("Path {} is a file", path[0])})
                        }
                        FileTreeObject::Folder(f) => {
                            return f.remove(&path[1..path.len()], recursive, mgr)
                        }
                    }
                }
            }
            Err(ProjectError {msg: "Path not found".to_string()})
        }
    }
}

