use crate::prelude::*;
use crate::stubs::File;
use crate::stubs::FileInputStream;
// package me.ag2s.epublib.domain;

// import java.io.File;
// import java.io.FileInputStream;
// import java.io.IOException;
// import java.io.InputStream;

/**
 * 用于创建epub，添加大文件（如大量图片）时容易OOM，使用LazyResource，避免OOM.
 *
 */

pub struct FileResourceProvider {
    //需要导入资源的父目录
    dir: String,
}

impl FileResourceProvider {
    /**
     * 创建一个文件夹里面文件夹的LazyResourceProvider，用于LazyResource。
     * @param parentDir 文件的目录
     */
    pub fn new(parent_dir: String) -> FileResourceProvider {
        FileResourceProvider {
            dir: parent_dir,
        }
    }

    /**
     * 创建一个文件夹里面文件夹的LazyResourceProvider，用于LazyResource。
     * @param parentFile 文件夹
     */
    // @SuppressWarnings("unused")
    pub fn with_parent_file(parent_file: &File) -> FileResourceProvider {
        FileResourceProvider {
            dir: parent_file.get_path(),
        }
    }

    /**
     * 根据子文件名href,再父目录下读取文件获取FileInputStream
     * @param href 子文件名href
     * @return 对应href的FileInputStream
     * @throws IOException 抛出IOException
     */
    // @Override
    pub fn get_resource_stream(&self, href: &String) -> Result<FileInputStream, IOException> {
        return Ok(FileInputStream::new(&File::new_path(&File::new(&self.dir), href)));
    }
}
