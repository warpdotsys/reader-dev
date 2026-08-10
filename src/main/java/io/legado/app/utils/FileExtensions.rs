pub fn getFile(base: &File, subDirFiles: &[&str]) -> File {
    let path = FileUtils::getPath(base, subDirFiles);
    File::new(path)
}

pub fn exists(base: &File, subDirFiles: &[&str]) -> bool {
    getFile(base, subDirFiles).exists()
}
