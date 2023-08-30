mod pdb;
mod project;
mod mdb;

#[cfg(test)]
mod tests{
    use super::{pdb, project};
    #[test]
    fn test_create() {
        let project_mgr = project::ProjectManager::new();
        let project = project_mgr.create_project("test2", None).unwrap();
        project.mkdir("test");
        project.mkdir("test.test2");
        project.ls(None);
        project.ls(Some("test"));
        ()
    }
}

