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
        return ("http:" + this).to_http_url().to_string();
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
            this.list_files().for_each(|it| {
                delete_recursively(it);
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
    let buffer = ByteArray::new(1024);
    let mut output_stream: Option<OutputStream> = None;
    let mut input_stream: Option<InputStream> = None;
    try {
        let zf = ZipFile::new(this.to_string());
        let entries = zf.entries();
        while entries.has_more_elements() {
            let zip_entry: ZipEntry = entries.next_element() as ZipEntry;
            let zip_entry_name: String = zip_entry.name;

            let desc_file_path: String = desc_dir.to_string() + &File::SEPARATOR.to_string() + &zip_entry_name;
            if zip_entry.is_directory {
                create_dir(&desc_file_path);
            } else {
                input_stream = Some(zf.get_input_stream(zip_entry));
                let desc_file: File = create_file(&desc_file_path);
                output_stream = Some(FileOutputStream::new(desc_file));

                let mut len: i32;
                while input_stream.read(buffer).also(&mut len) > 0 {
                    output_stream.write(buffer, 0, len);
                }
                input_stream.close();
                output_stream.close();
            }
        }
        return true;
    } catch (e: Exception) {
        e.printStackTrace();
    } finally {
        input_stream?.close();
        output_stream?.close();
    }
    return false;
}

// fun File.zip(zipFilePath: String): Boolean {
pub fn zip_file(this: &File, zip_file_path: &str) -> bool {
    if !this.exists() {
        return false;
    }
    if this.is_directory() {
        let files = this.list_files();
        let files_list: Vec<File> = files.to_list();
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
    let buffer = ByteArray::new(1024);
    let mut zip_output_stream: Option<ZipOutputStream> = None;
    let mut input_stream: Option<FileInputStream> = None;
    try {
        zip_output_stream = Some(ZipOutputStream::new(FileOutputStream::new(zip_file)));
        for file in files {
            if !file.exists() { continue; }
            zip_output_stream.put_next_entry(ZipEntry::new(file.name));
            input_stream = Some(FileInputStream::new(file));
            let mut len: i32;
            while input_stream.read(buffer).also(&mut len) > 0 {
                zip_output_stream.write(buffer, 0, len);
            }
            zip_output_stream.close_entry();
        }
        return true;
    } catch (e: Exception) {
        e.printStackTrace();
    } finally {
        input_stream?.close();
        zip_output_stream?.close();
    }
    return false;
}

// fun createDir(filePath: String): File {
pub fn create_dir(file_path: &str) -> File {
    let file = File::new(file_path);
    logger.debug(format!("createDir filePath {}", file_path));
    if !file.exists() {
        file.mkdirs();
    }
    return file;
}

// fun createFile(filePath: String): File {
pub fn create_file(file_path: &str) -> File {
    let file = File::new(file_path);
    let parent_file = file.parent_file.unwrap();
    logger.debug(format!("createFile filePath {}", file_path));
    if !parent_file.exists() {
        parent_file.mkdirs();
    }
    if !file.exists() {
        file.create_new_file();
    }
    return file;
}
