use crate::prelude::*;
// package com.htmake.reader.entity

// data class Size(
//         val width: Double,
//         val height: Double
// )
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Size {
        Size { width, height }
    }
}
