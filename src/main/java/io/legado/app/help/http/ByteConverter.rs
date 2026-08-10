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
    ) -> Option<Converter<ResponseBody, Vec<u8>>> {
        Some(Converter::new(|value: ResponseBody| {
            value.bytes()
        }))
    }
}
