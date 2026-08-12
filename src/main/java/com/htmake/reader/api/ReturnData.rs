use crate::prelude::*;
// package com.htmake.reader.api

// class ReturnData {
pub struct ReturnData {
    is_success: bool,
    error_msg: String,
    data: Option<Box<dyn std::any::Any>>,
}

impl ReturnData {
    // var isSuccess: Boolean = false
    //     private set
    pub fn is_success(&self) -> bool {
        self.is_success
    }

    // var errorMsg: String = "未知错误,请联系开发者!"
    //     private set
    pub fn error_msg(&self) -> &String {
        &self.error_msg
    }

    // var data: Any? = None
    //     private set
    pub fn data(&self) -> &Option<Box<dyn std::any::Any>> {
        &self.data
    }

    pub fn new() -> ReturnData {
        ReturnData {
            is_success: false,
            error_msg: String::from("未知错误,请联系开发者!"),
            data: None,
        }
    }

    // fun setErrorMsg(errorMsg: String): ReturnData {
    //     this.isSuccess = false
    //     this.errorMsg = errorMsg
    //     return this
    // }
    pub fn set_error_msg(&mut self, error_msg: String) -> &mut ReturnData {
        self.is_success = false;
        self.error_msg = error_msg;
        self
    }

    // fun setData(data: Any, msg: String = ""): ReturnData {
    //     this.isSuccess = true
    //     this.errorMsg = msg
    //     this.data = data
    //     return this
    // }
    pub fn set_data(&mut self, data: Box<dyn std::any::Any>, msg: String) -> &mut ReturnData {
        self.is_success = true;
        self.error_msg = msg;
        self.data = Some(data);
        self
    }

    // fun setData(data: Any, msg: String = ""): ReturnData {
    //     this.isSuccess = true
    //     this.errorMsg = msg
    //     this.data = data
    //     return this
    // }
    pub fn set_data_default(&mut self, data: Box<dyn std::any::Any>) -> &mut ReturnData {
        self.set_data(data, String::from(""))
    }
}
