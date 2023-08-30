use crate::pdb::ProjectFileSystemManager;
use crate::mdb::{MainDBManager, Result, ProjectDocument};
use std::clone::Clone;


pub(crate) struct ProjectManager {
    db: MainDBManager,
}



pub(crate) struct Project {
    cfg: ProjectDocument,
    fs: ProjectFileSystemManager,
}

impl ProjectManager {

    pub(crate) fn new() -> ProjectManager {
        let db = MainDBManager::get();
        ProjectManager {
            db: db,
        }        
    }

    pub(crate) fn load_project(&self, name: &str, colname: Option<&str>) -> Result<Project> {
        let pconfig = self.db.get_project(name, colname);
        match pconfig {
            Ok(config) => {
                let fs = ProjectFileSystemManager::open(config.clone());
                Ok(Project {
                    cfg: config,
                    fs: fs,
                })
            }
            Err(e) => Err(e)
        }

    }

    pub(crate) fn create_project(&self, name: &str, colname: Option<&str>) -> Result<Project> {
        let pconfig = self.db.create_project(name, colname);
        match pconfig {
            Ok(config) => {
                let fs = ProjectFileSystemManager::open(config.clone());
                Ok(Project {
                    cfg: config,
                    fs: fs,
                })
            }
            Err(e) => Err(e)

        }
    }
}


impl Project {
    
    pub fn mkdir(&self, folder_path: &str) {
        self.fs.create_folder(folder_path);
    }

    pub fn ls(&self, folder_path: Option<&str>) {
        let folder_uuid: String;
        match folder_path {
            None => {
                folder_uuid = self.cfg.uuid.clone();
            }
            Some(path) => {
                let folder_uuid_ = self.fs.get_folder_at_path(&path.split(".").collect::<Vec<&str>>(), None);
                match folder_uuid_ {
                    Some(uuid) => {
                        folder_uuid = uuid;
                    }
                    None => {
                        println!("Folder not found");
                        return;
                    }
                }
            }

        }
        let contents = self.fs.get_folder_contents(&folder_uuid).unwrap();
        for content in contents {
            println!("{}", content.name);
        }
        

    }

}