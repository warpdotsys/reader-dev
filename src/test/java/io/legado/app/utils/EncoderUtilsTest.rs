// package io.legado.app.utils;

// import java.security.KeyPair;
// import org.junit.Test;

// import static org.junit.Assert.assertEquals;

// public class EncoderUtilsTest {
pub struct EncoderUtilsTest;

impl EncoderUtilsTest {
    // @Test
    // public void rsaMethodsRoundTripShortAndSegmentedText() throws Exception {
    pub fn rsa_methods_round_trip_short_and_segmented_text() {
        let keys = EncoderUtils::INSTANCE.generate_keys();
        let short_text = "license payload";
        let mut long_text_builder = StringBuilder::new();
        for index in 0..80 {
            long_text_builder.append(short_text);
        }
        let long_text = long_text_builder.to_string();

        assert_eq!(short_text, EncoderUtils::INSTANCE.decrypt_by_private_key(
            EncoderUtils::INSTANCE.encrypt_by_public_key(short_text, keys.get_public()), keys.get_private()));
        assert_eq!(short_text, EncoderUtils::INSTANCE.decrypt_by_public_key(
            EncoderUtils::INSTANCE.encrypt_by_private_key(short_text, keys.get_private()), keys.get_public()));
        assert_eq!(long_text, EncoderUtils::INSTANCE.decrypt_segment_by_private_key(
            EncoderUtils::INSTANCE.encrypt_segment_by_public_key(long_text, keys.get_public(), 2048), keys.get_private(), 2048));
        assert_eq!(long_text, EncoderUtils::INSTANCE.decrypt_segment_by_public_key(
            EncoderUtils::INSTANCE.encrypt_segment_by_private_key(long_text, keys.get_private(), 2048), keys.get_public(), 2048));
    }
}
