use crate::prelude::*;
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
        // fix: EncoderUtils 为无状态 unit struct，方法均为关联函数（无 INSTANCE 常量），直接 EncoderUtils::xxx 调用；
        // fix: 方法名以 EncoderUtils.rs 实际 API 为准（camelCase 转录名），带 keySize 的分段方法为 *_keySize 变体（返回 Option<String> → unwrap）
        let keys = EncoderUtils::generateKeys();
        let short_text = "license payload";
        let mut long_text_builder = StringBuilder::new();
        for index in 0..80 {
            long_text_builder.append(short_text);
        }
        let long_text = long_text_builder.to_string();

        assert_eq!(short_text, EncoderUtils::decryptByPrivateKey(
            &EncoderUtils::encryptByPublicKey(short_text, keys.get_public()), keys.get_private()));
        assert_eq!(short_text, EncoderUtils::decryptByPublicKey(
            &EncoderUtils::encryptByPrivateKey(short_text, keys.get_private()), keys.get_public()));
        assert_eq!(long_text, EncoderUtils::decryptSegmentByPrivateKey_keySize(
            &EncoderUtils::encryptSegmentByPublicKey_keySize(&long_text, keys.get_public(), 2048), keys.get_private(), 2048).unwrap());
        assert_eq!(long_text, EncoderUtils::decryptSegmentByPublicKey_keySize(
            &EncoderUtils::encryptSegmentByPrivateKey_keySize(&long_text, keys.get_private(), 2048), keys.get_public(), 2048).unwrap());
    }
}
