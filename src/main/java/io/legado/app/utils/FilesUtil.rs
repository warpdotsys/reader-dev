use crate::prelude::*;
// fix: 显式导入以覆盖 prelude 中多个 glob 重导出导致的同名歧义
// fix: (File ← stubs 与 me_ag2s_epublib_util_resourceutil; Closeable ← stubs 与 me_ag2s_epublib_util_ioutil; 等)
use crate::stubs::{BufferedInputStream, ByteArrayOutputStream, Closeable, File, FileInputStream};
pub struct FileUtils;

impl FileUtils {
    pub const GB: i64 = 1073741824;
    pub const MB: i64 = 1048576;
    pub const KB: i64 = 1024;

    pub fn getFileExtetion(url: &str) -> String {
        Self::getFileExtetion_default(url, "")
    }

    pub fn getFileExtetion_default(url: &str, defaultExt: &str) -> String {
        let result: Result<String, ()> = (|| {
            let file = url.splitn(2, "?").nth(0).unwrap().split("/").last().unwrap();
            let dotPos = file.rfind('.').unwrap_or(0usize.wrapping_sub(1));
            if dotPos != 0usize.wrapping_sub(1) {
                Ok(file[dotPos + 1..].to_string())
            } else {
                Err(())
            }
        })();
        match result {
            Ok(s) => s,
            Err(_) => defaultExt.to_string(),
        }
    }

    pub fn exists(root: &File, subDirFiles: &[&str]) -> bool {
        Self::getFile(root, subDirFiles).exists()
    }

    pub fn createFileIfNotExist(root: &File, subDirFiles: &[&str]) -> File {
        let filePath = Self::getPath(root, subDirFiles);
        Self::createFileIfNotExist_path(&filePath)
    }

    pub fn createFolderIfNotExist(root: &File, subDirs: &[&str]) -> File {
        let filePath = Self::getPath(root, subDirs);
        Self::createFolderIfNotExist_path(&filePath)
    }

    pub fn createFolderIfNotExist_path(filePath: &str) -> File {
        let file = File::new(filePath);
        //如果文件夹不存在，就创建它
        if !file.exists() {
            file.mkdirs();
        }
        file
    }

    #[allow(non_snake_case)]
    pub fn createFileIfNotExist_path(filePath: &str) -> File {
        let file = File::new(filePath);
        if !file.exists() {
            //创建父类文件夹
            if let Some(parent) = file.parent() {
                Self::createFolderIfNotExist_path(&parent);
            }
            //创建文件
            file.createNewFile();
        }
        file
    }

    pub fn createFileWithReplace(filePath: &str) -> File {
        let file = File::new(filePath);
        if !file.exists() {
            //创建父类文件夹
            if let Some(parent) = file.parent() {
                Self::createFolderIfNotExist_path(&parent);
            }
            //创建文件
            file.createNewFile();
        } else {
            file.delete();
            file.createNewFile();
        }
        file
    }

    pub fn getFile(root: &File, subDirFiles: &[&str]) -> File {
        let filePath = Self::getPath(root, subDirFiles);
        File::new(&filePath)
    }

    pub fn getPath(root: &File, subDirFiles: &[&str]) -> String {
        let mut path = String::from(root.absolutePath());
        for it in subDirFiles {
            if !it.is_empty() {
                path.push_str(&File::separator());
                path.push_str(it);
            }
        }
        path
    }

    //递归删除文件夹下的数据
    pub fn deleteFile(filePath: &str) {
        let file = File::new(filePath);
        if !file.exists() {
            return;
        }

        if file.isDirectory() {
            let files = file.listFiles();
            if let Some(files) = files {
                for subFile in files {
                    let path = subFile.path();
                    Self::deleteFile(&path);
                }
            }
        }
        //删除文件
        file.delete();
    }

    pub fn getCachePath() -> String {
        // fix: 真实实现（原 panic!("Not implemented")，JS zip/unzip 流程中断）
        format!("{}/storage/cache", crate::stubs::get_work_dir())
    }

    pub const BY_NAME_ASC: i32 = 0;
    pub const BY_NAME_DESC: i32 = 1;
    pub const BY_TIME_ASC: i32 = 2;
    pub const BY_TIME_DESC: i32 = 3;
    pub const BY_SIZE_ASC: i32 = 4;
    pub const BY_SIZE_DESC: i32 = 5;
    pub const BY_EXTENSION_ASC: i32 = 6;
    pub const BY_EXTENSION_DESC: i32 = 7;

    /**
     * 将目录分隔符统一为平台默认的分隔符，并为目录结尾添加分隔符
     */
    pub fn separator(path: &str) -> String {
        let mut path1 = path.to_string();
        let separator = File::separator();
        path1 = path1.replace("\\", &separator);
        if !path1.ends_with(&separator) {
            path1.push_str(&separator);
        }
        path1
    }

    pub fn closeSilently(c: Option<&mut dyn Closeable>) {
        // fix: Option<&mut dyn Closeable> 不支持 PartialEq，Kotlin 的 `c == None` 改为 is_none()
        if c.is_none() {
            return;
        }
        let _ = c.unwrap().close();
    }

    /**
     * 列出指定目录下的所有子目录
     */
    pub fn listDirs(startDirPath: &str) -> Vec<File> {
        Self::listDirs_sorted(startDirPath, None, Self::BY_NAME_ASC)
    }

    pub fn listDirs_sorted(
        startDirPath: &str,
        excludeDirs: Option<&[String]>,
        sortType: i32
    ) -> Vec<File> {
        let mut excludeDirs1 = excludeDirs;
        let mut dirList = Vec::new();
        let startDir = File::new(startDirPath);
        if !startDir.isDirectory() {
            return Vec::new();
        }
        let dirs = match startDir.listFiles_dir() {
            Some(dirs) => dirs,
            None => return Vec::new(),
        };
        if excludeDirs1 == None {
            excludeDirs1 = Some(&[]);
        }
        for dir in dirs {
            let file = dir.absoluteFile();
            if !excludeDirs1.unwrap().contains(&file.name()) {
                dirList.push(file);
            }
        }
        match sortType {
            Self::BY_NAME_ASC => Self::sort(&mut dirList, &mut SortByName::new()),
            Self::BY_NAME_DESC => {
                Self::sort(&mut dirList, &mut SortByName::new());
                dirList.reverse();
            }
            Self::BY_TIME_ASC => Self::sort(&mut dirList, &mut SortByTime::new()),
            Self::BY_TIME_DESC => {
                Self::sort(&mut dirList, &mut SortByTime::new());
                dirList.reverse();
            }
            Self::BY_SIZE_ASC => Self::sort(&mut dirList, &mut SortBySize::new()),
            Self::BY_SIZE_DESC => {
                Self::sort(&mut dirList, &mut SortBySize::new());
                dirList.reverse();
            }
            Self::BY_EXTENSION_ASC => Self::sort(&mut dirList, &mut SortByExtension::new()),
            Self::BY_EXTENSION_DESC => {
                Self::sort(&mut dirList, &mut SortByExtension::new());
                dirList.reverse();
            }
            _ => {}
        }
        dirList
    }

    /**
     * 列出指定目录下的所有子目录及所有文件
     */
    pub fn listDirsAndFiles(startDirPath: &str) -> Option<Vec<File>> {
        Self::listDirsAndFiles_extensions(startDirPath, None)
    }

    pub fn listDirsAndFiles_extensions(
        startDirPath: &str,
        allowExtensions: Option<&[String]>
    ) -> Option<Vec<File>> {
        let files: Option<Vec<File>> = if allowExtensions == None {
            Some(Self::listFiles(startDirPath))
        } else {
            Self::listFiles_allow(startDirPath, allowExtensions)
        };
        let dirs = Self::listDirs(startDirPath);
        if files == None {
            return None;
        }
        let mut result = dirs;
        result.extend(files.unwrap());
        Some(result)
    }

    /**
     * 列出指定目录下的所有文件
     */
    pub fn listFiles(startDirPath: &str) -> Vec<File> {
        Self::listFiles_filtered(startDirPath, None, Self::BY_NAME_ASC)
    }

    pub fn listFiles_filtered(
        startDirPath: &str,
        filterPattern: Option<&Pattern>,
        sortType: i32
    ) -> Vec<File> {
        let mut fileList = Vec::new();
        let f = File::new(startDirPath);
        if !f.isDirectory() {
            return Vec::new();
        }
        let files = match f.listFiles_filter(|file| {
            if file.isDirectory() {
                return false;
            }
            match filterPattern {
                // fix: Pattern::matcher 接收 String（按值），去掉多余的 &
                Some(p) => p.matcher(file.name()).find(),
                None => true,
            }
        }) {
            Some(files) => files,
            None => return Vec::new(),
        };
        for file in files {
            fileList.push(file.absoluteFile());
        }
        match sortType {
            Self::BY_NAME_ASC => Self::sort(&mut fileList, &mut SortByName::new()),
            Self::BY_NAME_DESC => {
                Self::sort(&mut fileList, &mut SortByName::new());
                fileList.reverse();
            }
            Self::BY_TIME_ASC => Self::sort(&mut fileList, &mut SortByTime::new()),
            Self::BY_TIME_DESC => {
                Self::sort(&mut fileList, &mut SortByTime::new());
                fileList.reverse();
            }
            Self::BY_SIZE_ASC => Self::sort(&mut fileList, &mut SortBySize::new()),
            Self::BY_SIZE_DESC => {
                Self::sort(&mut fileList, &mut SortBySize::new());
                fileList.reverse();
            }
            Self::BY_EXTENSION_ASC => Self::sort(&mut fileList, &mut SortByExtension::new()),
            Self::BY_EXTENSION_DESC => {
                Self::sort(&mut fileList, &mut SortByExtension::new());
                fileList.reverse();
            }
            _ => {}
        }
        fileList
    }

    /**
     * 列出指定目录下的所有文件
     */
    pub fn listFiles_allow(startDirPath: &str, allowExtensions: Option<&[String]>) -> Option<Vec<File>> {
        let file = File::new(startDirPath);
        file.listFiles_name(|name| {
            //返回当前目录所有以某些扩展名结尾的文件
            let extension = Self::getExtension(name);
            match allowExtensions {
                Some(exts) => exts.contains(&extension),
                None => true,
            }
        })
    }

    /**
     * 列出指定目录下的所有文件
     */
    pub fn listFiles_allow_one(startDirPath: &str, allowExtension: Option<&str>) -> Option<Vec<File>> {
        if allowExtension == None {
            Self::listFiles_allow(startDirPath, None)
        } else {
            Self::listFiles_allow(startDirPath, Some(&[allowExtension.unwrap().to_string()]))
        }
    }

    /**
     * 判断文件或目录是否存在
     */
    pub fn exist(path: &str) -> bool {
        let file = File::new(path);
        file.exists()
    }

    /**
     * 删除文件或目录
     */
    pub fn delete(file: &File) -> bool {
        Self::delete_deleteRootDir(file, false)
    }

    pub fn delete_deleteRootDir(file: &File, deleteRootDir: bool) -> bool {
        let mut result = false;
        if file.isFile() {
            //是文件
            result = Self::deleteResolveEBUSY(file);
        } else {
            //是目录
            let files = match file.listFiles() {
                Some(files) => files,
                None => return false,
            };
            if files.is_empty() {
                result = deleteRootDir && Self::deleteResolveEBUSY(file);
            } else {
                for f in files {
                    Self::delete_deleteRootDir(&f, deleteRootDir);
                    result = Self::deleteResolveEBUSY(&f);
                }
            }
            if deleteRootDir {
                result = Self::deleteResolveEBUSY(file);
            }
        }
        result
    }

    /**
     * bug: open failed: EBUSY (Device or resource busy)
     * fix: http://stackoverflow.com/questions/11539657/open-failed-ebusy-device-or-resource-busy
     */
    fn deleteResolveEBUSY(file: &File) -> bool {
        // Before you delete a Directory or File: rename it!
        // fix: File::new 接收 &str，Kotlin 中 String 参数需借用
        let to = File::new(&(file.absolutePath() + &System::currentTimeMillis().to_string()));

        file.renameTo(&to);
        to.delete()
    }

    /**
     * 删除文件或目录
     */
    pub fn delete_path(path: &str) -> bool {
        Self::delete_path_deleteRootDir(path, false)
    }

    pub fn delete_path_deleteRootDir(path: &str, deleteRootDir: bool) -> bool {
        let file = File::new(path);

        if file.exists() {
            Self::delete_deleteRootDir(&file, deleteRootDir)
        } else {
            false
        }
    }

    /**
     * 复制文件为另一个文件，或复制某目录下的所有文件及目录到另一个目录下
     */
    pub fn copy(src: &str, tar: &str) -> bool {
        let srcFile = File::new(src);
        srcFile.exists() && Self::copy_file(&srcFile, &File::new(tar))
    }

    /**
     * 复制文件或目录
     */
    pub fn copy_file(src: &File, tar: &File) -> bool {
        let result: Result<bool, ()> = (|| {
            if src.isFile() {
                let is = FileInputStream::new(src);
                let op = FileOutputStream::new(tar);
                let mut bis = BufferedInputStream::new(is);
                let mut bos = BufferedOutputStream::new(op);
                let mut bt = vec![0u8; 1024 * 8];
                loop {
                    let len = bis.read(&mut bt);
                    if len == -1 {
                        break;
                    } else {
                        // fix: Java 的重载 write(byte[], int, int) → write_range；len 为 i32 需转 usize
                        bos.write_range(&bt, 0, len as usize);
                    }
                }
                bis.close();
                bos.close();
            } else if src.isDirectory() {
                tar.mkdirs();
                if let Some(files) = src.listFiles() {
                    for file in files {
                        Self::copy_file(&file.absoluteFile(), &File::new_path(&tar.absoluteFile(), &file.name()));
                    }
                }
            }
            Ok(true)
        })();
        match result {
            Ok(b) => b,
            Err(_) => false,
        }
    }

    /**
     * 移动文件或目录
     */
    pub fn move_str(src: &str, tar: &str) -> bool {
        Self::move_file(&File::new(src), &File::new(tar))
    }

    /**
     * 移动文件或目录
     */
    pub fn move_file(src: &File, tar: &File) -> bool {
        // fix: Kotlin 重载解析中 move(File, File) 调用的是 rename(File, File)，即 rename_file
        Self::rename_file(src, tar)
    }

    /**
     * 文件重命名
     */
    pub fn rename(oldPath: &str, newPath: &str) -> bool {
        Self::rename_file(&File::new(oldPath), &File::new(newPath))
    }

    /**
     * 文件重命名
     */
    pub fn rename_file(src: &File, tar: &File) -> bool {
        src.renameTo(tar)
    }

    /**
     * 读取文本文件, 失败将返回空串
     */
    pub fn readText(filepath: &str) -> String {
        Self::readText_charset(filepath, "utf-8")
    }

    pub fn readText_charset(filepath: &str, charset: &str) -> String {
        let data = Self::readBytes(filepath);
        if let Some(data) = data {
            return String::from_utf8_lossy(&data)
                .trim()
                .trim_matches(|c| c <= ' ')
                .to_string();
        }
        "".to_string()
    }

    /**
     * 读取文件内容, 失败将返回空串
     */
    pub fn readBytes(filepath: &str) -> Option<Vec<u8>> {
        let mut fis: Option<FileInputStream> = None;
        let result: Result<Vec<u8>, std::io::Error> = (|| {
            // fix: Java 的 FileInputStream(String) 构造 → new_path（new 仅接收 &File）
            fis = Some(FileInputStream::new_path(filepath));
            let mut baos = ByteArrayOutputStream::new();
            let mut buffer = vec![0u8; 1024];
            loop {
                // fix: &mut buffer 与 buffer.len() 不能同时借用，先取长度再调用
                let buf_len = buffer.len();
                let len = fis.as_mut().unwrap().read(&mut buffer, 0, buf_len);
                if len == -1 {
                    break;
                } else {
                    // fix: Java 的重载 write(byte[], int, int) → write_range；len 为 i32 需转 usize
                    baos.write_range(&buffer, 0, len as usize);
                }
            }
            let data = baos.toByteArray();
            baos.close();
            Ok(data)
        })();
        Self::closeSilently(fis.as_mut().map(|f| f as &mut dyn Closeable));
        match result {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    /**
     * 保存文本内容
     */
    pub fn writeText(filepath: &str, content: &str) -> bool {
        Self::writeText_charset(filepath, content, "utf-8")
    }

    pub fn writeText_charset(filepath: &str, content: &str, charset: &str) -> bool {
        Self::writeBytes(filepath, &content.as_bytes().to_vec())
    }

    /**
     * 保存文件内容
     */
    pub fn writeBytes(filepath: &str, data: &[u8]) -> bool {
        let file = File::new(filepath);
        let mut fos: Option<FileOutputStream> = None;
        let result: Result<bool, std::io::Error> = (|| {
            if !file.exists() {
                file.parentFile().map(|p| p.mkdirs());
                file.createNewFile();
            }
            // fix: Java 的 FileOutputStream(String) 构造 → new_path（new 仅接收 &File）
            fos = Some(FileOutputStream::new_path(filepath));
            fos.as_mut().unwrap().write(data);
            Ok(true)
        })();
        Self::closeSilently(fos.as_mut().map(|f| f as &mut dyn Closeable));
        match result {
            Ok(b) => b,
            Err(_) => false,
        }
    }

    /**
     * 保存文件内容
     */
    pub fn writeInputStream(filepath: &str, data: &mut dyn InputStream) -> bool {
        let file = File::new(filepath);
        Self::writeInputStream_file(&file, data)
    }

    /**
     * 保存文件内容
     */
    pub fn writeInputStream_file(file: &File, data: &mut dyn InputStream) -> bool {
        let mut fos: Option<FileOutputStream> = None;
        let result: Result<bool, std::io::Error> = (|| {
            if !file.exists() {
                file.parentFile().map(|p| p.mkdirs());
                file.createNewFile();
            }
            let mut buffer = vec![0u8; 1024 * 4];
            fos = Some(FileOutputStream::new(file));
            loop {
                // fix: &mut buffer 与 buffer.len() 不能同时借用，先取长度再调用
                let buf_len = buffer.len();
                let len = data.read(&mut buffer, 0, buf_len);
                if len == -1 {
                    break;
                } else {
                    // fix: Java 的重载 write(byte[], int, int) → write_range；len 为 i32 需转 usize
                    fos.as_mut().unwrap().write_range(&buffer, 0, len as usize);
                }
            }
            data.close();
            fos.as_mut().unwrap().flush();
            Ok(true)
        })();
        Self::closeSilently(fos.as_mut().map(|f| f as &mut dyn Closeable));
        match result {
            Ok(b) => b,
            Err(_) => false,
        }
    }

    /**
     * 追加文本内容
     */
    pub fn appendText(path: &str, content: &str) -> bool {
        let file = File::new(path);
        let mut writer: Option<FileWriter> = None;
        let result: Result<bool, std::io::Error> = (|| {
            if !file.exists() {
                file.createNewFile();
            }
            writer = Some(FileWriter::new(&file, true));
            writer.as_mut().unwrap().write(content);
            Ok(true)
        })();
        Self::closeSilently(writer.as_mut().map(|f| f as &mut dyn Closeable));
        match result {
            Ok(b) => b,
            Err(_) => false,
        }
    }

    /**
     * 获取文件大小
     */
    pub fn getLength(path: &str) -> i64 {
        let file = File::new(path);
        if !file.isFile() || !file.exists() {
            0
        } else {
            file.length()
        }
    }

    /**
     * 获取文件或网址的名称（包括后缀）
     */
    pub fn getName(pathOrUrl: Option<&str>) -> String {
        if pathOrUrl == None {
            return "".to_string();
        }
        let pathOrUrl = pathOrUrl.unwrap();
        let pos = pathOrUrl.rfind('/').unwrap_or(0usize.wrapping_sub(1));
        if 0 <= pos {
            pathOrUrl[pos + 1..].to_string()
        } else {
            System::currentTimeMillis().to_string() + "." + &Self::getExtension(pathOrUrl)
        }
    }

    /**
     * 获取文件名（不包括扩展名）
     */
    pub fn getNameExcludeExtension(path: &str) -> String {
        let result: Result<String, ()> = (|| {
            let mut fileName = File::new(path).name();
            let lastIndexOf = fileName.rfind('.');
            if let Some(i) = lastIndexOf {
                fileName = fileName[0..i].to_string();
            }
            Ok(fileName)
        })();
        match result {
            Ok(s) => s,
            Err(_) => "".to_string(),
        }
    }

    /**
     * 获取格式化后的文件大小
     */
    pub fn getSize(path: &str) -> String {
        let fileSize = Self::getLength(path);
        Self::toFileSizeString(fileSize)
    }

    pub fn toFileSizeString(fileSize: i64) -> String {
        let df = DecimalFormat::new("0.00");
        let fileSizeString: String;
        if fileSize < Self::KB {
            fileSizeString = fileSize.to_string() + "B";
        } else if fileSize < Self::MB {
            fileSizeString = df.format(fileSize as f64 / Self::KB as f64) + "K";
        } else if fileSize < Self::GB {
            fileSizeString = df.format(fileSize as f64 / Self::MB as f64) + "M";
        } else {
            fileSizeString = df.format(fileSize as f64 / Self::GB as f64) + "G";
        }
        fileSizeString
    }

    /**
     * 获取文件后缀,不包括"."
     */
    pub fn getExtension(pathOrUrl: &str) -> String {
        let dotPos = pathOrUrl.rfind('.').unwrap_or(0usize.wrapping_sub(1));
        if 0 <= dotPos {
            pathOrUrl[dotPos + 1..].to_string()
        } else {
            "ext".to_string()
        }
    }

    /**
     * 获取文件的MIME类型
     */
    pub fn getMimeType(pathOrUrl: &str) -> String {
        // val ext = getExtension(pathOrUrl)
        // val map = MimeTypeMap.getSingleton()
        // return map.getMimeTypeFromExtension(ext) ?: "*/*"
        //
        panic!("Not implemented")
    }

    /**
     * 获取格式化后的文件/目录创建或最后修改时间
     */
    pub fn getDateTime(path: &str) -> String {
        Self::getDateTime_format(path, "yyyy年MM月dd日HH:mm")
    }

    pub fn getDateTime_format(path: &str, format: &str) -> String {
        let file = File::new(path);
        Self::getDateTime_file(&file, format)
    }

    /**
     * 获取格式化后的文件/目录创建或最后修改时间
     */
    pub fn getDateTime_file(file: &File, format: &str) -> String {
        // fix: Kotlin 中 val 对象的字段可修改，Rust 中需声明 mut
        let mut cal = Calendar::getInstance();
        cal.timeInMillis = file.lastModified();
        // fix: Java 的 SimpleDateFormat(String, Locale) 构造 → new_2args（new 仅接收 pattern）
        SimpleDateFormat::new_2args(format, Locale::PRC).format(cal.time)
    }

    /**
     * 比较两个文件的最后修改时间
     */
    pub fn compareLastModified(path1: &str, path2: &str) -> i32 {
        let stamp1 = File::new(path1).lastModified();
        let stamp2 = File::new(path2).lastModified();
        if stamp1 > stamp2 {
            1
        } else if stamp1 < stamp2 {
            -1
        } else {
            0
        }
    }

    /**
     * 创建多级别的目录
     */
    pub fn makeDirs(path: &str) -> bool {
        Self::makeDirs_file(&File::new(path))
    }

    /**
     * 创建多级别的目录
     */
    pub fn makeDirs_file(file: &File) -> bool {
        file.mkdirs()
    }

    // fix: Kotlin 的 Collections.sort(list, comparator) 在 Rust 中经 FileComparator 适配 sort_by 实现。
    // fix: stable Rust 不支持手动实现 FnMut/FnOnce，原 FnMut/FnOnce impl 用本地 trait FileComparator 替代。
    fn sort(list: &mut Vec<File>, comparator: &mut dyn FileComparator) {
        list.sort_by(|a, b| {
            let c = comparator.compare(Some(a), Some(b));
            if c < 0 {
                std::cmp::Ordering::Less
            } else if c > 0 {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }
}

// fix: Kotlin 中的嵌套类 SortByExtension/SortByName/SortBySize/SortByTime（Comparator<File>）
// fix: 移到模块级——Rust 不允许在 impl 块内嵌套 struct / impl。
// fix: 用本地 trait FileComparator 替代 Java 的 Comparator<File>（stable Rust 无法手动实现 Fn 系列 trait）。
pub trait FileComparator {
    fn compare(&self, f1: Option<&File>, f2: Option<&File>) -> i32;
}

pub struct SortByExtension;

impl SortByExtension {
    pub fn new() -> SortByExtension {
        SortByExtension {}
    }
}

impl FileComparator for SortByExtension {
    fn compare(&self, f1: Option<&File>, f2: Option<&File>) -> i32 {
        if f1 == None || f2 == None {
            if f1 == None {
                -1
            } else {
                1
            }
        } else {
            let f1 = f1.unwrap();
            let f2 = f2.unwrap();
            if f1.isDirectory() && f2.isFile() {
                -1
            } else if f1.isFile() && f2.isDirectory() {
                1
            } else {
                f1.name().cmp_ignore_case(&f2.name())
            }
        }
    }
}

pub struct SortByName {
    caseSensitive: bool,
}

impl SortByName {
    pub fn new() -> SortByName {
        SortByName { caseSensitive: false }
    }

    pub fn new_caseSensitive(caseSensitive: bool) -> SortByName {
        SortByName { caseSensitive }
    }
}

impl FileComparator for SortByName {
    fn compare(&self, f1: Option<&File>, f2: Option<&File>) -> i32 {
        if f1 == None || f2 == None {
            return if f1 == None { -1 } else { 1 };
        } else {
            let f1 = f1.unwrap();
            let f2 = f2.unwrap();
            return if f1.isDirectory() && f2.isFile() {
                -1
            } else if f1.isFile() && f2.isDirectory() {
                1
            } else {
                let s1 = f1.name();
                let s2 = f2.name();
                if self.caseSensitive {
                    s1.cmp_sensitive(&s2)
                } else {
                    s1.cmp_ignore_case(&s2)
                }
            };
        }
    }
}

pub struct SortBySize;

impl SortBySize {
    pub fn new() -> SortBySize {
        SortBySize {}
    }
}

impl FileComparator for SortBySize {
    fn compare(&self, f1: Option<&File>, f2: Option<&File>) -> i32 {
        if f1 == None || f2 == None {
            if f1 == None {
                -1
            } else {
                1
            }
        } else {
            let f1 = f1.unwrap();
            let f2 = f2.unwrap();
            if f1.isDirectory() && f2.isFile() {
                -1
            } else if f1.isFile() && f2.isDirectory() {
                1
            } else {
                if f1.length() < f2.length() {
                    -1
                } else {
                    1
                }
            }
        }
    }
}

pub struct SortByTime;

impl SortByTime {
    pub fn new() -> SortByTime {
        SortByTime {}
    }
}

impl FileComparator for SortByTime {
    fn compare(&self, f1: Option<&File>, f2: Option<&File>) -> i32 {
        if f1 == None || f2 == None {
            if f1 == None {
                -1
            } else {
                1
            }
        } else {
            let f1 = f1.unwrap();
            let f2 = f2.unwrap();
            if f1.isDirectory() && f2.isFile() {
                -1
            } else if f1.isFile() && f2.isDirectory() {
                1
            } else {
                if f1.lastModified() > f2.lastModified() {
                    -1
                } else {
                    1
                }
            }
        }
    }
}
