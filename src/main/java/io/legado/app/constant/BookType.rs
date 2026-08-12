use crate::prelude::*;
pub struct BookType;

impl BookType {
    pub const default: i32 = 0;           // 0 文本
    pub const audio: i32 = 1;             // 1 音频
    pub const image: i32 = 2;            // 2 图片
    pub const file: i32 = 3;               // 3 只提供下载服务的网站
    pub const video: i32 = 4;              // 4 视频
    pub const local: &'static str = "loc_book";
}
