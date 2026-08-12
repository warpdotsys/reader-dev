use crate::prelude::*;
// package io.legado.app.help.http
//
// import okhttp3.ResponseBody
// import retrofit2.Converter
// import retrofit2.Retrofit
// import java.lang.reflect.Type

// class ByteConverter : Converter.Factory() {
pub struct ByteConverter;

impl ConverterFactory for ByteConverter {
    // override fun responseBodyConverter(
    //     type: Type?,
    //     annotations: Array<Annotation>?,
    //     retrofit: Retrofit?
    // ): Converter<ResponseBody, ByteArray>? {
    //     return Converter { value ->
    //         value.bytes()
    //     }
    // }
    fn response_body_converter(
        &self,
        _type: Option<&Type>,
        _annotations: Option<&[Annotation]>,
        _retrofit: Option<&Retrofit>,
    ) -> Option<Converter<ResponseBody, String>> {
        // fix: 与 stubs ConverterFactory 固定签名一致（同 EncodeConverter）；字节流按 UTF-8 转为 String
        Some(Converter::new(|value: ResponseBody| -> String {
            String::from_utf8_lossy(&value.bytes()).into_owned()
        }))
    }
}
