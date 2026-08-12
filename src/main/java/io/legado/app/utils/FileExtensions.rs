use crate::prelude::*;
// fix: prelude 中多个 glob 重导出同名歧义（File ← stubs/me_ag2s_epublib_util_resourceutil; FileUtils ← stubs/io_legado_app_utils_filesutil）
use crate::stubs::File;
use crate::io_legado_app_utils_filesutil::FileUtils;
pub fn getFile(base: &File, subDirFiles: &[&str]) -> File {
    let path = FileUtils::getPath(base, subDirFiles);
    File::new(&path)
}

pub fn exists(base: &File, subDirFiles: &[&str]) -> bool {
    getFile(base, subDirFiles).exists()
}
