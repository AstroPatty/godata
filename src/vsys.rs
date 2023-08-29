use std::path::PathBuf;

struct VFileSystem{
    name: String,
    root: VFolder,
}

struct VFolder {
    name: String,
    path: PathBuf,
    items: Vec<VFileSystemItem>,
}

struct VFile {
    name: String,
    path: PathBuf,
}

enum VFileSystemItem {
    File(VFile),
    Folder(VFolder),
}
