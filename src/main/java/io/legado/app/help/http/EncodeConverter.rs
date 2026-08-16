use crate::prelude::*;
// package io.legado.app.help.http
//
// import io.legado.app.utils.UTF8BOMFighter
// import okhttp3.ResponseBody
// import io.legado.app.utils.EncodingDetect
// import retrofit2.Converter
// import retrofit2.Retrofit
// import java.lang.reflect.Type
// import java.nio.charset.Charset

// class EncodeConverter(private val encode: String? = None) : Converter.Factory() {
pub struct EncodeConverter {
    encode: Option<String>,
}

impl EncodeConverter {
    pub fn new(encode: Option<String>) -> EncodeConverter {
        EncodeConverter { encode }
    }
}

impl ConverterFactory for EncodeConverter {
    // override fun responseBodyConverter(
    //     type: Type?,
    //     annotations: Array<Annotation>?,
    //     retrofit: Retrofit?
    // ): Converter<ResponseBody, String>? {
    //     return Converter { value ->
    //         val responseBytes = UTF8BOMFighter.removeUTF8BOM(value.bytes())
    //         encode?.let { return@Converter String(responseBytes, Charset.forName(encode)) }
    //
    //         var charsetName: String? = None
    //         val mediaType = value.contentType()
    //         //根据http头判断
    //         if (mediaType != None) {
    //             val charset = mediaType.charset()
    //             charsetName = charset?.displayName()
    //         }
    //
    //         if (charsetName == None) {
    //             charsetName = EncodingDetect.getHtmlEncode(responseBytes)
    //         }
    //
    //         String(responseBytes, Charset.forName(charsetName))
    //     }
    // }
    fn response_body_converter(
        &self,
        _type: Option<&Type>,
        _annotations: Option<&[Annotation]>,
        _retrofit: Option<&Retrofit>,
    ) -> Option<Converter<ResponseBody, String>> {
        // return Converter { value -> ... }
        // fix: Converter::new 要求闭包 'static，encode 先 clone 再 move 进闭包
        let encode = self.encode.clone();
        Some(Converter::new(move |value: ResponseBody| -> String {
            // val responseBytes = UTF8BOMFighter.removeUTF8BOM(value.bytes())
            let response_bytes = UTF8BOMFighter::removeUTF8BOM_bytes(&value.bytes());
            // encode?.let { return@Converter String(responseBytes, Charset.forName(encode)) }
            if let Some(encode) = &encode {
                return String::from_utf8_lossy(&response_bytes).into_owned();
            }

            // var charsetName: String? = None
            let mut charset_name: Option<String> = None;
            // val mediaType = value.contentType()
            let media_type = value.content_type();
            //根据http头判断（Content-Type 的 charset 参数）
            if media_type.is_some() {
                let charset = media_type
                    .as_ref()
                    .and_then(|ct| ct.split(';').skip(1).find_map(|p| p.trim().strip_prefix("charset=")))
                    .map(|it| it.trim_matches('"').to_string());
                charset_name = charset;
            }

            // if (charsetName == None) {
            //     charsetName = EncodingDetect.getHtmlEncode(responseBytes)
            // }
            if charset_name.is_none() {
                charset_name = Some(EncodingDetect::getHtmlEncode(&response_bytes));
            }

            // String(responseBytes, Charset.forName(charsetName))
            String::from_utf8_lossy(&response_bytes).into_owned()
        }))
    }
}
