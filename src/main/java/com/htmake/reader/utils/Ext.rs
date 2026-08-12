use crate::prelude::*;
// 显式导入消解跨模块 glob 导入歧义（优先于 prelude 的 glob 导入）
use crate::stubs::{
    ByteArray, File, FileInputStream, FileOutputStream, ZipEntry, ZipFile,
    ZipOutputStream,
};
// @file:JvmName("ExtKt")
// @file:JvmMultifileClass

// package com.htmake.reader.utils

// import java.io.File
// import java.io.OutputStream
// import java.io.InputStream
// import java.io.FileOutputStream
// import java.io.FileInputStream
// import java.util.zip.ZipFile
// import java.util.zip.ZipEntry
// import java.util.zip.ZipOutputStream
// import okhttp3.HttpUrl.Companion.toHttpUrl

/**
 * @Date: 2019-07-19 23:43
 * @Description:
 */

// fun String.url(): String {
pub fn url(this: &str) -> String {
    if this.starts_with("//") {
        return ("http:".to_owned() + this).to_http_url().to_string();
    } else if this.starts_with("http") {
        return this.to_http_url().to_string();
    }
    return this.to_string();
}

// fun File.deleteRecursively() {
pub fn delete_recursively(this: &File) {
    if this.exists() {
        if this.is_file() {
            this.delete();
        } else {
            this.list_files().into_iter().for_each(|it| {
                delete_recursively(&it);
            });
            this.delete();
        }
    }
}

// fun File.unzip(descDir: String): Boolean {
pub fn unzip(this: &File, desc_dir: &str) -> bool {
    if !this.exists() {
        return false;
    }
    let mut buffer = ByteArray::new(1024);
    let mut output_stream: Option<FileOutputStream> = None;
    let mut input_stream: Option<FileInputStream> = None;
    // fix: try/catch/finally → 闭包 + match（finally 中关闭流的逻辑移到 match 之后执行）
    let try_result: Result<(), StubError> = (|| {
        let zf = ZipFile::new(this);
        let mut entries = zf.entries();
        while entries.has_more_elements() {
            let zip_entry: ZipEntry = entries.next_element();
            // fix: E0382——name 字段后续还需借用 zip_entry，先 clone
            let zip_entry_name: String = zip_entry.name.clone();

            let desc_file_path: String = desc_dir.to_string() + &File::SEPARATOR.to_string() + &zip_entry_name;
            if zip_entry.is_directory {
                create_dir(&desc_file_path);
            } else {
                input_stream = Some(zf.get_input_stream(&zip_entry));
                let desc_file: File = create_file(&desc_file_path);
                output_stream = Some(FileOutputStream::new(&desc_file));

                let mut len: i32 = 0;
                // fix: E0502/E0381——buffer.len() 与 &mut buffer 借用冲突、len 未初始化；先取值并赋初值
                let buf_len = buffer.len();
                while input_stream.read(&mut buffer, 0, buf_len).also(&mut len) > 0 {
                    output_stream.write(&buffer, 0, len as usize);
                }
                input_stream.close();
                output_stream.close();
            }
        }
        Ok(())
    })();
    let result = match try_result {
        Ok(_) => true,
        Err(e) => {
            e.printStackTrace();
            false
        }
    };
    // fix: Kotlin `inputStream?.close()`（finally）→ if-let 显式解包
    if let Some(mut stream) = input_stream {
        stream.close();
    }
    if let Some(mut stream) = output_stream {
        stream.close();
    }
    return result;
}

// fun File.zip(zipFilePath: String): Boolean {
pub fn zip_file(this: &File, zip_file_path: &str) -> bool {
    if !this.exists() {
        return false;
    }
    if this.is_directory() {
        let files = this.list_files();
        let files_list: Vec<File> = files;
        return zip(files_list, zip_file_path);
    } else {
        return zip(vec![this.clone()], zip_file_path);
    }
}

// fun zip(files: List<File>, zipFilePath: String): Boolean {
pub fn zip(files: Vec<File>, zip_file_path: &str) -> bool {
    if files.is_empty() {
        return false;
    }

    let zip_file = create_file(zip_file_path);
    let mut buffer = ByteArray::new(1024);
    let mut zip_output_stream: Option<ZipOutputStream> = None;
    let mut input_stream: Option<FileInputStream> = None;
    // fix: try/catch/finally → 闭包 + match（finally 中关闭流的逻辑移到 match 之后执行）
    let try_result: Result<(), StubError> = (|| {
        zip_output_stream = Some(ZipOutputStream::new(FileOutputStream::new(&zip_file)));
        for file in files {
            if !file.exists() { continue; }
            // fix: Option<ZipOutputStream> 方法调用 → if-let 显式解包
            if let Some(zip_output_stream) = zip_output_stream.as_mut() {
                // fix: E0382——file.name 移动进 ZipEntry 后 file 又被借用，先 clone
                zip_output_stream.put_next_entry(&ZipEntry::new(file.name.clone()));
                input_stream = Some(FileInputStream::new(&file));
                let mut len: i32 = 0;
                // fix: E0502/E0381——buffer.len() 与 &mut buffer 借用冲突、len 未初始化；先取值并赋初值
                let buf_len = buffer.len();
                while input_stream.read(&mut buffer, 0, buf_len).also(&mut len) > 0 {
                    zip_output_stream.write_range(&buffer, 0, len as usize);
                }
                zip_output_stream.close_entry();
            }
        }
        Ok(())
    })();
    let result = match try_result {
        Ok(_) => true,
        Err(e) => {
            e.printStackTrace();
            false
        }
    };
    // fix: Kotlin `inputStream?.close()`（finally）→ if-let 显式解包
    if let Some(mut stream) = input_stream {
        stream.close();
    }
    if let Some(mut stream) = zip_output_stream {
        stream.close();
    }
    return result;
}

// fun createDir(filePath: String): File {
pub fn create_dir(file_path: &str) -> File {
    let file = File::new(file_path);
    logger().debug(format!("createDir filePath {}", file_path));
    if !file.exists() {
        file.mkdirs();
    }
    return file;
}

// fun createFile(filePath: String): File {
pub fn create_file(file_path: &str) -> File {
    let file = File::new(file_path);
    // fix: E0382——file.parent_file 部分移动，先 clone 再 unwrap
    let parent_file = file.parent_file.clone().unwrap();
    logger().debug(format!("createFile filePath {}", file_path));
    if !parent_file.exists() {
        parent_file.mkdirs();
    }
    if !file.exists() {
        file.create_new_file();
    }
    return file;
}
