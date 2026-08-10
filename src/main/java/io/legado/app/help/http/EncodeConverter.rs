// package io.legado.app.help.http
//
// import io.legado.app.utils.UTF8BOMFighter
// import okhttp3.ResponseBody
// import io.legado.app.utils.EncodingDetect
// import retrofit2.Converter
// import retrofit2.Retrofit
// import java.lang.reflect.Type
// import java.nio.charset.Charset

// class EncodeConverter(private val encode: String? = null) : Converter.Factory() {
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
    //         var charsetName: String? = null
    //         val mediaType = value.contentType()
    //         //根据http头判断
    //         if (mediaType != null) {
    //             val charset = mediaType.charset()
    //             charsetName = charset?.displayName()
    //         }
    //
    //         if (charsetName == null) {
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
        Some(Converter::new(move |value: ResponseBody| {
            // val responseBytes = UTF8BOMFighter.removeUTF8BOM(value.bytes())
            let response_bytes = UTF8BOMFighter::remove_utf8_bom(value.bytes());
            // encode?.let { return@Converter String(responseBytes, Charset.forName(encode)) }
            if let Some(encode) = &self.encode {
                return String::from_utf8_lossy(&response_bytes).into_owned();
            }

            // var charsetName: String? = null
            let mut charset_name: Option<String> = None;
            // val mediaType = value.contentType()
            let media_type = value.content_type();
            //根据http头判断
            if media_type.is_some() {
                // val charset = mediaType.charset()
                let charset = media_type.as_ref().unwrap().charset();
                // charsetName = charset?.displayName()
                charset_name = charset.map(|it| it.display_name());
            }

            // if (charsetName == null) {
            //     charsetName = EncodingDetect.getHtmlEncode(responseBytes)
            // }
            if charset_name.is_none() {
                charset_name = Some(EncodingDetect::get_html_encode(&response_bytes));
            }

            // String(responseBytes, Charset.forName(charsetName))
            String::from_utf8_lossy(&response_bytes).into_owned()
        }))
    }
}
