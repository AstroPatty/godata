use crate::pdb::{FolderDocument, FileDocument, ProjectFileSystemManager, FileSystemObject};
use crate::mdb::{ProjectError, Result};
use crate::project::Project;
use std::path::PathBuf;
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

}
pub(crate) struct FileTree {
    root: FileTreeFolder,
    mgr: ProjectFileSystemManager
}

pub(crate) struct FileTreeFolder {
    pub(super) cfg: FolderDocument,
    pub(super) children: Vec<FileTreeObject>,
}

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
    pub(crate) fn add_file(&mut self, path: PathBuf, project_path: &str, resursive: bool) -> Result<()> {
        let split_project_path = project_path.split("/").collect::<Vec<&str>>();
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
            println!("Adding file to root");
            self.mgr.add(&FileSystemObject::File(new_file.cfg.clone()))?;
            self.root.children.push(FileTreeObject::File(new_file));
            return Ok(())
        }
        let parent_folder =  match self.root.query_mutable(&split_project_path[0..split_project_path.len() - 1]) {
            Ok(f) => f,
            Err(e) => {
                if resursive {
                    self.add_folder(&split_project_path[0..split_project_path.len() - 1], true)?;

                    self.root.query_mutable(&split_project_path[0..split_project_path.len() - 1])?

                } else {
                    return Err(e)
                }
            }
        };


        match parent_folder {
            FileTreeObject::File(f) => {
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
                f.children.push(FileTreeObject::File(new_file));
                return Ok(())
            }
        }
    }
    pub(crate) fn add_folder(&mut self, split_project_path: &[&str], recursive: bool) -> Result<()> {
        println!("Adding folder {}", split_project_path.join("/"));
        if split_project_path.len() == 1 { //We're adding to the root folder
            
            let children = self.root.children.iter().map(|x| x.get_name()).collect::<Vec<&str>>();
            if children.contains(&split_project_path[0]) {
                return Err(ProjectError {msg: "Path already exists".to_string()})
            }
            let uuid = nanoid::nanoid!();
            let new_folder = FileTreeFolder {
                cfg: FolderDocument {
                    name: split_project_path[0].to_string(),
                    uuid: uuid,
                    children: Vec::new(),
                    location: None,
                    parent: Some(self.root.cfg.uuid.clone()),
                },
                children: Vec::new(),
            };
            self.mgr.add(&FileSystemObject::Folder(new_folder.cfg.clone()))?;
            self.root.insert(split_project_path, FileTreeObject::Folder(new_folder))?;
            return Ok(());
        }

        let parent_path = &split_project_path[0..split_project_path.len()-1];
        let parent = self.root.query_mutable(&parent_path);
        match parent {
            Ok(p) => { //The parent already exists!
                match p {
                    FileTreeObject::File(f) => {
                        return Err(ProjectError {msg: format!("Path {} is a file", &parent_path.join("/"))})
                    }
                    FileTreeObject::Folder(f) => {
                        let children = f.children.iter().map(|x| x.get_name()).collect::<Vec<&str>>();
                        if children.contains(&split_project_path[split_project_path.len() - 1]) {
                            return Err(ProjectError {msg: format!("Path {} already exists", &split_project_path.join("/"))})
                        }
                        let uuid = nanoid::nanoid!();
                        let new_folder = FileTreeFolder {
                            cfg: FolderDocument {
                                name: split_project_path[split_project_path.len() - 1].to_string(),
                                uuid: uuid,
                                children: Vec::new(),
                                location: None,
                                parent: Some(f.cfg.uuid.clone()),
                            },
                            children: Vec::new(),
                        };
                        self.mgr.add(&FileSystemObject::Folder(new_folder.cfg.clone()))?;
                        f.children.push(FileTreeObject::Folder(new_folder));
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
    


    pub(crate) fn remove(&mut self, path: &str, recursive: bool) -> Result<()> {
        let split = path.split("/").collect::<Vec<&str>>();
        let doc = self.root.remove(&split, recursive);
        match doc {
            Ok(d) => {
                self.mgr.remove(&d.get_config())?;
                Ok(())
            },
            Err(e) => Err(ProjectError{msg: e.msg})
        }
    }

    pub(crate) fn get_contents(&self, path: Option<&str>) -> Result<&Vec<FileTreeObject>> {
        let split = match path {
            Some(p) => p.split("/").collect::<Vec<&str>>(),
            None => return Ok(&self.root.children)
        };
        
        let node = self.root.query(&split)?;
        match node {
            FileTreeObject::File(f) => {
                return Err(ProjectError {msg: "Path is a file".to_string()})
            }
            FileTreeObject::Folder(f) => {
                return Ok(&f.children)
            }
        }
    }
}


impl FileTreeFolder {
    fn new_from_record(cfg: FolderDocument, mgr: &ProjectFileSystemManager) -> FileTreeFolder {
        let children_records = mgr.get_child_records(&cfg).unwrap();
        let mut children_nodes = Vec::new();
        for child in children_records {
            match child {
                FileSystemObject::Folder(f) => {
                    let child_node = FileTreeFolder::new_from_record(f, mgr);
                    children_nodes.push(FileTreeObject::Folder(child_node));
                }
                
                FileSystemObject::File(f) => {
                    let child_node = FileTreeFile{cfg: f};
                    children_nodes.push(FileTreeObject::File(child_node));
                }
                
            }
        }
        FileTreeFolder {
            cfg: cfg,
            children: children_nodes,
        }
    }

    pub(crate) fn query(&self, query_path: &[&str]) -> Result<&FileTreeObject> {
        for child in &self.children {
            if child.get_name() == query_path[0] {
                if query_path.len() == 1 {
                    // We're at the end of the query path and we have a match
                    return Ok(&child)
                }
                match child {
                    FileTreeObject::File(f) => {
                        // We're not at the end of the query path, but we've found
                        // a file
                        return Err(ProjectError { msg: "Path not found".to_string()})
                    }
                    FileTreeObject::Folder(f) => {
                        // We're not at the end of the query path, but we cound a folder
                        return f.query(&query_path[1..query_path.len()])
                    }
                }
            }
        }
        return Err(ProjectError {msg: "Path not found".to_string()})
    }

    fn query_mutable(&mut self, query_path: &[&str]) -> Result<&mut FileTreeObject> {
        for child in &mut self.children {
            if child.get_name() == query_path[0] {
                if query_path.len() == 1 {
                    // We're at the end of the query path and we have a match
                    return Ok(child)
                }
                match child {
                    FileTreeObject::File(f) => {
                        // We're not at the end of the query path, but we've found
                        // a file
                        return Err(ProjectError { msg: "Path not found".to_string()})
                    }
                    FileTreeObject::Folder(f) => {
                        // We're not at the end of the query path, but we cound a folder
                        return f.query_mutable(&query_path[1..query_path.len()])
                    }
                }
            }
        }
        return Err(ProjectError {msg: "Path not found".to_string()})
    }    
    pub(crate) fn insert(&mut self, path: &[&str], obj: FileTreeObject) -> Result<()> {
        if path.len() == 1 {
            // Base case, we have the last already-extant folder
            let child_names: Vec<&str> = self.children.iter().map(|x| x.get_name()).collect();
            if child_names.contains(&path[0]) {
                return Err(ProjectError {msg: "Path already exists".to_string()})
            }
            self.children.push(obj);
            return Ok(())
        }

        let parent = self.query_mutable(&path[0..path.len()-1])?;
        match parent {
            FileTreeObject::File(f) => {
                return Err(ProjectError {msg: "Path not found".to_string()})
            }
            FileTreeObject::Folder(f) => {
                return f.insert(&path[path.len() - 1..path.len()], obj)
            }
        }
    }
    pub(crate) fn remove(&mut self, path: &[&str], recursive: bool) -> Result<FileTreeObject> {
        if path.len() == 1 {
            //check children for path[0]
            for (i, child) in self.children.iter().enumerate() {
                if child.get_name() == path[0] {
                    match child {
                        FileTreeObject::File(f) => {}
                        FileTreeObject::Folder(f) => {
                            if !recursive && f.children.len() != 0{
                                return Err(ProjectError {msg: "Path is a folder which contains items".to_string()})
                            }
                        }
                    }

                    return Ok(self.children.remove(i))
                }
            }
            return Err(ProjectError {msg: "Path not found".to_string()})


        }

        else {
            for child in self.children.iter_mut() {

                if child.get_name() == path[0] {
                    match child {
                        FileTreeObject::File(f) => {
                            return Err(ProjectError {msg: format!("Path {} is a file", path[0])})
                        }
                        FileTreeObject::Folder(f) => {
                            return f.remove(&path[1..path.len()], recursive)
                        }
                    }
                }
            }
            Err(ProjectError {msg: "Path not found".to_string()})
        }
    }
}

