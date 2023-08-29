mod pdb;

#[cfg(test)]
mod tests{
    use super::pdb;
    #[test]
    fn test_create() {
        let manager = pdb::ProjectDBManager::get();
        let ppath = manager.create_project("test", None);
        let result = manager.remove_project("test", "default");
        println!("{}", ppath.unwrap().display());
        ()
    }
}

