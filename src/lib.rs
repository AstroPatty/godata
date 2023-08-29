mod vfsio;

#[cfg(test)]
mod tests{
    use super::vfsio;
    #[test]
    fn test_create() {
        let manager = vfsio::DBManager::get();
        manager.create_project("test", None);
        manager.remove_project("test", "default")

    }
}

