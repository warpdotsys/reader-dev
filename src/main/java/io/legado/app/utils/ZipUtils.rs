use crate::prelude::*;
use std::sync::Arc;

// 显式导入消解跨模块 glob 导入歧义（优先于 prelude 的 glob 导入）
use crate::stubs::{
    BufferedInputStream, BufferedOutputStream, File, FileInputStream, FileOutputStream, ZipEntry,
    ZipFile, ZipOutputStream,
};

pub struct ZipUtils;

impl ZipUtils {
    /**
     * Zip the files.
     *
     * @param srcFiles    The source of files.
     * @param zipFilePath The path of ZIP file.
     * @return `true`: success<br></br>`false`: fail
     * @throws IOException if an I/O error has occurred
     */
    pub fn zipFiles(srcFiles: &Vec<String>, zipFilePath: &str) -> bool {
        Self::zipFiles_comment(Some(srcFiles), Some(zipFilePath), None)
    }

    /**
     * Zip the files.
     *
     * @param srcFilePaths The paths of source files.
     * @param zipFilePath  The path of ZIP file.
     * @param comment      The comment.
     * @return `true`: success<br></br>`false`: fail
     * @throws IOException if an I/O error has occurred
     */
    pub fn zipFiles_comment(
        srcFilePaths: Option<&Vec<String>>,
        zipFilePath: Option<&str>,
        comment: Option<&str>
    ) -> bool {
        if srcFilePaths == None || zipFilePath == None {
            return false;
        }
        let mut zos = ZipOutputStream::new(FileOutputStream::new_path(zipFilePath.unwrap()));
        let result = (|| {
            for srcFile in srcFilePaths.unwrap() {
                if !Self::zipFile_inner(&Self::getFileByPath(srcFile).unwrap(), "", &mut zos, comment) {
                    return false;
                }
            }
            true
        })();
        drop(zos);
        result
    }

    /**
     * Zip the files.
     *
     * @param srcFiles The source of files.
     * @param zipFile  The ZIP file.
     * @param comment  The comment.
     * @return `true`: success<br></br>`false`: fail
     * @throws IOException if an I/O error has occurred
     */
    pub fn zipFiles_file(srcFiles: Option<&Vec<File>>, zipFile: Option<&File>) -> bool {
        Self::zipFiles_file_comment(srcFiles, zipFile, None)
    }

    pub fn zipFiles_file_comment(
        srcFiles: Option<&Vec<File>>,
        zipFile: Option<&File>,
        comment: Option<&str>
    ) -> bool {
        if srcFiles == None || zipFile == None {
            return false;
        }
        let mut zos = ZipOutputStream::new(FileOutputStream::new(zipFile.unwrap()));
        let result = (|| {
            for srcFile in srcFiles.unwrap() {
                if !Self::zipFile_inner(srcFile, "", &mut zos, comment) {
                    return false;
                }
            }
            true
        })();
        drop(zos);
        result
    }

    /**
     * Zip the file.
     *
     * @param srcFilePath The path of source file.
     * @param zipFilePath The path of ZIP file.
     * @return `true`: success<br></br>`false`: fail
     * @throws IOException if an I/O error has occurred
     */
    pub fn zipFile(srcFilePath: &str, zipFilePath: &str) -> bool {
        Self::zipFile_comment(srcFilePath, zipFilePath, None)
    }

    /**
     * Zip the file.
     *
     * @param srcFilePath The path of source file.
     * @param zipFilePath The path of ZIP file.
     * @param comment     The comment.
     * @return `true`: success<br></br>`false`: fail
     * @throws IOException if an I/O error has occurred
     */
    pub fn zipFile_comment(
        srcFilePath: &str,
        zipFilePath: &str,
        comment: Option<&str>
    ) -> bool {
        Self::zipFile_file(
            Self::getFileByPath(srcFilePath).as_ref(),
            Self::getFileByPath(zipFilePath).as_ref(),
            comment
        )
    }

    /**
     * Zip the file.
     *
     * @param srcFile The source of file.
     * @param zipFile The ZIP file.
     * @param comment The comment.
     * @return `true`: success<br></br>`false`: fail
     * @throws IOException if an I/O error has occurred
     */
    pub fn zipFile_file(srcFile: Option<&File>, zipFile: Option<&File>, comment: Option<&str>) -> bool {
        if srcFile == None || zipFile == None {
            return false;
        }
        let mut zos = ZipOutputStream::new(FileOutputStream::new(zipFile.unwrap()));
        return Self::zipFile_inner(srcFile.unwrap(), "", &mut zos, comment);
    }

    fn zipFile_inner(
        srcFile: &File,
        rootPath: &str,
        zos: &mut ZipOutputStream,
        comment: Option<&str>
    ) -> bool {
        let mut rootPath1 = rootPath.to_string();
        if !srcFile.exists() {
            return true;
        }
        let sep = if Self::isSpace(Some(&rootPath1)) { String::new() } else { File::separator() };
        rootPath1 = rootPath1 + &sep + &srcFile.name();
        if srcFile.isDirectory() {
            let fileList = srcFile.listFiles();
            if fileList == None || fileList.as_ref().unwrap().is_empty() {
                let mut entry = ZipEntry::new(format!("{rootPath1}/"));
                entry.comment = comment.map(|s| s.to_string());
                zos.putNextEntry(&entry);
                zos.closeEntry();
            } else {
                for file in fileList.unwrap() {
                    if !Self::zipFile_inner(&file, &rootPath1, zos, comment) {
                        return false;
                    }
                }
            }
        } else {
            let mut is = BufferedInputStream::new(FileInputStream::new(srcFile));
            let mut entry = ZipEntry::new(rootPath1);
            entry.comment = comment.map(|s| s.to_string());
            zos.putNextEntry(&entry);
            zos.write(&is.readBytes());
            zos.closeEntry();
        }
        true
    }

    /**
     * Unzip the file.
     *
     * @param zipFilePath The path of ZIP file.
     * @param destDirPath The path of destination directory.
     * @return the unzipped files
     * @throws IOException if unzip unsuccessfully
     */
    pub fn unzipFile(zipFilePath: &str, destDirPath: &str) -> Option<Vec<File>> {
        Self::unzipFileByKeyword(zipFilePath, destDirPath, None)
    }

    /**
     * Unzip the file.
     *
     * @param zipFile The ZIP file.
     * @param destDir The destination directory.
     * @return the unzipped files
     * @throws IOException if unzip unsuccessfully
     */
    pub fn unzipFile_file(zipFile: &File, destDir: &File) -> Option<Vec<File>> {
        Self::unzipFileByKeyword_file(Some(zipFile), Some(destDir), None)
    }

    /**
     * Unzip the file by keyword.
     *
     * @param zipFilePath The path of ZIP file.
     * @param destDirPath The path of destination directory.
     * @param keyword     The keyboard.
     * @return the unzipped files
     * @throws IOException if unzip unsuccessfully
     */
    pub fn unzipFileByKeyword(
        zipFilePath: &str,
        destDirPath: &str,
        keyword: Option<&str>
    ) -> Option<Vec<File>> {
        Self::unzipFileByKeyword_file(
            Self::getFileByPath(zipFilePath).as_ref(),
            Self::getFileByPath(destDirPath).as_ref(),
            keyword
        )
    }

    /**
     * Unzip the file by keyword.
     *
     * @param zipFile The ZIP file.
     * @param destDir The destination directory.
     * @param keyword The keyboard.
     * @return the unzipped files
     * @throws IOException if unzip unsuccessfully
     */
    pub fn unzipFileByKeyword_file(
        zipFile: Option<&File>,
        destDir: Option<&File>,
        keyword: Option<&str>
    ) -> Option<Vec<File>> {
        if zipFile == None || destDir == None {
            return None;
        }
        let mut files = Vec::<File>::new();
        let zip = ZipFile::new(zipFile.unwrap());
        let mut entries = zip.entries();
        let result = (|| {
            if Self::isSpace(keyword) {
                while entries.hasMoreElements() {
                    let entry = entries.nextElement();
                    let entryName = entry.name();
                    if entryName.contains("../") {
                        logger().error(format!("ZipUtils entryName: {entryName} is dangerous!"));
                        continue;
                    }
                    if !Self::unzipChildFile(destDir.unwrap(), &mut files, &zip, &entry, &entryName) {
                        return files;
                    }
                }
            } else {
                while entries.hasMoreElements() {
                    let entry = entries.nextElement();
                    let entryName = entry.name();
                    if entryName.contains("../") {
                        logger().error(format!("ZipUtils entryName: {entryName} is dangerous!"));
                        continue;
                    }
                    if entryName.contains(keyword.unwrap()) {
                        if !Self::unzipChildFile(destDir.unwrap(), &mut files, &zip, &entry, &entryName) {
                            return files;
                        }
                    }
                }
            }
            files
        })();
        drop(zip);
        Some(result)
    }

    fn unzipChildFile(
        destDir: &File,
        files: &mut Vec<File>,
        zip: &ZipFile,
        entry: &ZipEntry,
        name: &str
    ) -> bool {
        let file = File::new_path(destDir, name);
        files.push(file.clone());
        if entry.isDirectory() {
            return Self::createOrExistsDir(Some(&file));
        } else {
            if !Self::createOrExistsFile(Some(&file)) {
                return false;
            }
            let mut is = BufferedInputStream::new(zip.getInputStream(entry));
            let bytes = is.readBytes();
            let mut out = BufferedOutputStream::new(FileOutputStream::new(&file));
            out.write(&bytes);
        }
        true
    }

    /**
     * Return the files' path in ZIP file.
     *
     * @param zipFilePath The path of ZIP file.
     * @return the files' path in ZIP file
     * @throws IOException if an I/O error has occurred
     */
    pub fn getFilesPath(zipFilePath: &str) -> Option<Vec<String>> {
        Self::getFilesPath_file(Self::getFileByPath(zipFilePath).as_ref())
    }

    /**
     * Return the files' path in ZIP file.
     *
     * @param zipFile The ZIP file.
     * @return the files' path in ZIP file
     * @throws IOException if an I/O error has occurred
     */
    pub fn getFilesPath_file(zipFile: Option<&File>) -> Option<Vec<String>> {
        if zipFile == None {
            return None;
        }
        let mut paths = Vec::<String>::new();
        let zip = ZipFile::new(zipFile.unwrap());
        let mut entries = zip.entries();
        while entries.hasMoreElements() {
            let entryName = entries.nextElement().name();
            if entryName.contains("../") {
                logger().error(format!("ZipUtils entryName: {entryName} is dangerous!"));
                paths.push(entryName);
            } else {
                paths.push(entryName);
            }
        }
        zip.close();
        Some(paths)
    }

    /**
     * Return the files' comment in ZIP file.
     *
     * @param zipFilePath The path of ZIP file.
     * @return the files' comment in ZIP file
     * @throws IOException if an I/O error has occurred
     */
    pub fn getComments(zipFilePath: &str) -> Option<Vec<String>> {
        Self::getComments_file(Self::getFileByPath(zipFilePath).as_ref())
    }

    /**
     * Return the files' comment in ZIP file.
     *
     * @param zipFile The ZIP file.
     * @return the files' comment in ZIP file
     * @throws IOException if an I/O error has occurred
     */
    pub fn getComments_file(zipFile: Option<&File>) -> Option<Vec<String>> {
        if zipFile == None {
            return None;
        }
        let mut comments = Vec::<String>::new();
        let zip = ZipFile::new(zipFile.unwrap());
        let mut entries = zip.entries();
        while entries.hasMoreElements() {
            let entry = entries.nextElement();
            comments.push(entry.comment());
        }
        zip.close();
        Some(comments)
    }

    fn createOrExistsDir(file: Option<&File>) -> bool {
        match file {
            Some(f) => if f.exists() { f.isDirectory() } else { f.mkdirs() },
            None => false,
        }
    }

    fn createOrExistsFile(file: Option<&File>) -> bool {
        let file = match file {
            Some(f) => f,
            None => return false,
        };
        if file.exists() {
            return file.isFile();
        }
        if !Self::createOrExistsDir(file.parentFile().as_ref()) {
            return false;
        }
        let result: Result<bool, std::io::Error> = (|| {
            file.createNewFile();
            Ok(true)
        })();
        match result {
            Ok(b) => b,
            Err(e) => {
                e.print_stack_trace();
                false
            }
        }
    }

    fn getFileByPath(filePath: &str) -> Option<File> {
        if Self::isSpace(Some(filePath)) {
            None
        } else {
            Some(File::new(filePath))
        }
    }

    fn isSpace(s: Option<&str>) -> bool {
        let s = match s {
            Some(s) => s,
            None => return true,
        };
        let mut i = 0;
        let len = s.len();
        while i < len {
            if !Character::isWhitespace(s.as_bytes()[i] as char) {
                return false;
            }
            i += 1;
        }
        true
    }
}
