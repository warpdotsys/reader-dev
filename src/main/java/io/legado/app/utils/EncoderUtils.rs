use crate::prelude::*;
use crate::io_legado_app_utils_base64::Base64;
use crate::stubs::ByteArrayOutputStream;

#[allow(unused)]
pub struct EncoderUtils;

impl EncoderUtils {
    pub fn escape(src: &str) -> String {
        let mut tmp = String::new();
        for char in src.chars() {
            let charCode = char as u32;
            if charCode >= 48 && charCode <= 57 || charCode >= 65 && charCode <= 90 || charCode >= 97 && charCode <= 122 {
                tmp.push(char);
                continue;
            }

            let prefix = if charCode < 16 {
                "%0"
            } else if charCode < 256 {
                "%"
            } else {
                "%u"
            };
            tmp.push_str(prefix);
            tmp.push_str(&format!("{:x}", charCode));
        }
        tmp
    }

    pub fn base64Decode(str: &str) -> String {
        Self::base64Decode_flags(str, Base64::DEFAULT)
    }

    pub fn base64Decode_flags(str: &str, flags: i32) -> String {
        let bytes = Base64::decode_str(str, flags);
        String::from_utf8(bytes).unwrap()
    }

    pub fn base64Encode(str: &str) -> Option<String> {
        Self::base64Encode_flags(str, Base64::NO_WRAP)
    }

    pub fn base64Encode_flags(str: &str, flags: i32) -> Option<String> {
        Some(Base64::encodeToString(str.as_bytes(), flags))
    }

    //////////AES Start

    /**
     * Return the Base64-encode bytes of AES encryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the Base64-encode bytes of AES encryption
     */
    pub fn encryptAES2Base64(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: Option<&str>,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        let transformation = transformation.unwrap_or("DES/ECB/PKCS5Padding");
        Some(Base64::encode(&Self::encryptAES(data, key, Some(transformation), iv)?, Base64::NO_WRAP))
    }

    /**
     * Return the bytes of AES encryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES encryption
     */
    pub fn encryptAES(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: Option<&str>,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::symmetricTemplate(data, key, "AES", transformation.unwrap(), iv, true)
    }


    /**
     * Return the bytes of AES decryption for Base64-encode bytes.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES decryption for Base64-encode bytes
     */
    pub fn decryptBase64AES(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: &str,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::decryptAES(Some(&Base64::decode(data?, Base64::NO_WRAP)), key, transformation, iv)
    }

    /**
     * Return the bytes of AES decryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES decryption
     */
    pub fn decryptAES(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: &str,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::symmetricTemplate(data, key, "AES", transformation, iv, false)
    }


    /**
     * Return the bytes of symmetric encryption or decryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param algorithm      The name of algorithm.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, <i>DES/CBC/PKCS5Padding</i>.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @param isEncrypt      True to encrypt, false otherwise.
     * @return the bytes of symmetric encryption or decryption
     */
    pub fn symmetricTemplate(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        algorithm: &str,
        transformation: &str,
        iv: Option<&[u8]>,
        isEncrypt: bool
    ) -> Option<Vec<u8>> {
        let data = data?;
        let key = key?;
        if data.is_empty() || key.is_empty() {
            return None;
        }
        let keySpec = SecretKeySpec::new(key, algorithm);
        let mut cipher = Cipher::getInstance(transformation);
        let mode = if isEncrypt { Cipher::ENCRYPT_MODE } else { Cipher::DECRYPT_MODE };
        if iv == None || iv.unwrap().is_empty() {
            cipher.init_spec(mode, &keySpec);
        } else {
            let params = IvParameterSpec::new(iv.unwrap());
            cipher.init_spec_iv(mode, &keySpec, &params);
        }
        Some(cipher.do_final_data(data))
    }

    //////////DES Start

    /**
     * Return the Base64-encode bytes of DES encryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the Base64-encode bytes of AES encryption
     */
    pub fn encryptDES2Base64(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: Option<&str>,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        let transformation = transformation.unwrap_or("DES/ECB/PKCS5Padding");
        Some(Base64::encode(&Self::encryptDES(data, key, Some(transformation), iv)?, Base64::NO_WRAP))
    }

    /**
     * Return the bytes of DES encryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES encryption
     */
    pub fn encryptDES(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: Option<&str>,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::symmetricTemplate(data, key, "DES", transformation.unwrap(), iv, true)
    }


    /**
     * Return the bytes of DES decryption for Base64-encode bytes.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES decryption for Base64-encode bytes
     */
    pub fn decryptBase64DES(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: &str,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::decryptDES(Some(&Base64::decode(data?, Base64::NO_WRAP)), key, transformation, iv)
    }

    /**
     * Return the bytes of DES decryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DES/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES decryption
     */
    pub fn decryptDES(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: &str,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::symmetricTemplate(data, key, "DES", transformation, iv, false)
    }

    //////////DESede Start

    /**
     * Return the Base64-encode bytes of DESede encryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DESede/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the Base64-encode bytes of AES encryption
     */
    pub fn encryptDESede2Base64(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: Option<&str>,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        let transformation = transformation.unwrap_or("DESede/ECB/PKCS5Padding");
        Some(Base64::encode(&Self::encryptDESede(data, key, Some(transformation), iv)?, Base64::NO_WRAP))
    }

    /**
     * Return the bytes of DESede encryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DESede/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES encryption
     */
    pub fn encryptDESede(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: Option<&str>,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::symmetricTemplate(data, key, "DESede", transformation.unwrap(), iv, true)
    }


    /**
     * Return the bytes of DESede decryption for Base64-encode bytes.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DESede/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES decryption for Base64-encode bytes
     */
    pub fn decryptBase64DESede(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: &str,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::decryptDESede(Some(&Base64::decode(data?, Base64::NO_WRAP)), key, transformation, iv)
    }

    /**
     * Return the bytes of DESede decryption.
     *
     * @param data           The data.
     * @param key            The key.
     * @param transformation The name of the transformation,
     * 加密算法/加密模式/填充类型, *DESede/CBC/PKCS5Padding*.
     * @param iv             The buffer with the IV. The contents of the
     * buffer are copied to protect against subsequent modification.
     * @return the bytes of AES decryption
     */
    pub fn decryptDESede(
        data: Option<&[u8]>,
        key: Option<&[u8]>,
        transformation: &str,
        iv: Option<&[u8]>
    ) -> Option<Vec<u8>> {
        Self::symmetricTemplate(data, key, "DESede", transformation, iv, false)
    }

    pub fn encryptByPrivateKey(input: &str, privateKey: &PrivateKey) -> String {
        Self::rsaBase64(input, privateKey, Cipher::ENCRYPT_MODE)
    }

    pub fn decryptByPublicKey(input: &str, publicKey: &PublicKey) -> String {
        Self::rsaString(input, publicKey, Cipher::DECRYPT_MODE)
    }

    pub fn encryptByPublicKey(input: &str, publicKey: &PublicKey) -> String {
        Self::rsaBase64(input, publicKey, Cipher::ENCRYPT_MODE)
    }

    pub fn decryptByPrivateKey(input: &str, privateKey: &PrivateKey) -> String {
        Self::rsaString(input, privateKey, Cipher::DECRYPT_MODE)
    }

    pub fn encryptSegmentByPrivateKey(input: &str, privateKey: &PrivateKey) -> String {
        Self::encryptSegmentByPrivateKey_keySize(input, privateKey, 2048)
    }

    pub fn encryptSegmentByPrivateKey_keySize(input: &str, privateKey: &PrivateKey, keySize: i32) -> String {
        Self::rsaSegmentBase64(input.as_bytes(), privateKey, Cipher::ENCRYPT_MODE, keySize / 8 - 11)
    }

    pub fn decryptSegmentByPublicKey(input: &str, publicKey: &PublicKey) -> Option<String> {
        Self::decryptSegmentByPublicKey_keySize(input, publicKey, 2048)
    }

    pub fn decryptSegmentByPublicKey_keySize(input: &str, publicKey: &PublicKey, keySize: i32) -> Option<String> {
        Some(String::from_utf8(Self::rsaSegmentBytes(&Base64::decode_str(input, Base64::NO_WRAP), publicKey, Cipher::DECRYPT_MODE, keySize / 8)).ok()?)
    }

    pub fn encryptSegmentByPublicKey(input: &str, publicKey: &PublicKey) -> String {
        Self::encryptSegmentByPublicKey_keySize(input, publicKey, 2048)
    }

    pub fn encryptSegmentByPublicKey_keySize(input: &str, publicKey: &PublicKey, keySize: i32) -> String {
        Self::rsaSegmentBase64(input.as_bytes(), publicKey, Cipher::ENCRYPT_MODE, keySize / 8 - 11)
    }

    pub fn decryptSegmentByPrivateKey(input: &str, privateKey: &PrivateKey) -> Option<String> {
        Self::decryptSegmentByPrivateKey_keySize(input, privateKey, 2048)
    }

    pub fn decryptSegmentByPrivateKey_keySize(input: &str, privateKey: &PrivateKey, keySize: i32) -> Option<String> {
        Some(String::from_utf8(Self::rsaSegmentBytes(&Base64::decode_str(input, Base64::NO_WRAP), privateKey, Cipher::DECRYPT_MODE, keySize / 8)).ok()?)
    }

    pub fn generateKeys() -> KeyPair {
        let generator = KeyPairGenerator::getInstance("RSA");
        generator.genKeyPair()
    }

    fn rsaBase64(input: &str, key: &dyn java_security_Key, mode: i32) -> String {
        let mut cipher = Cipher::getInstance("RSA");
        cipher.init_key(mode, key);
        Base64::encodeToString(&cipher.do_final_data(input.as_bytes()), Base64::NO_WRAP)
    }

    fn rsaString(input: &str, key: &dyn java_security_Key, mode: i32) -> String {
        let mut cipher = Cipher::getInstance("RSA");
        cipher.init_key(mode, key);
        String::from_utf8(cipher.do_final_data(&Base64::decode_str(input, Base64::NO_WRAP))).unwrap()
    }

    fn rsaSegmentBase64(input: &[u8], key: &dyn java_security_Key, mode: i32, blockSize: i32) -> String {
        let mut output = ByteArrayOutputStream::new();
        let mut cipher = Cipher::getInstance("RSA");
        cipher.init_key(mode, key);
        let mut offset = 0usize;
        while input.len() - offset > 0 {
            let block: Vec<u8>;
            if input.len() - offset >= blockSize as usize {
                block = cipher.do_final_range(input, offset, blockSize as usize);
                offset += blockSize as usize;
            } else {
                block = cipher.do_final_range(input, offset, input.len() - offset);
                offset = input.len();
            }
            output.write(&block);
        }
        output.close();
        Base64::encodeToString(&output.toByteArray(), Base64::NO_WRAP)
    }

    fn rsaSegmentBytes(input: &[u8], key: &dyn java_security_Key, mode: i32, blockSize: i32) -> Vec<u8> {
        let mut output = ByteArrayOutputStream::new();
        let mut cipher = Cipher::getInstance("RSA");
        cipher.init_key(mode, key);
        let mut offset = 0usize;
        while input.len() - offset > 0 {
            let block: Vec<u8>;
            if input.len() - offset >= blockSize as usize {
                block = cipher.do_final_range(input, offset, blockSize as usize);
                offset += blockSize as usize;
            } else {
                block = cipher.do_final_range(input, offset, input.len() - offset);
                offset = input.len();
            }
            output.write(&block);
        }
        output.close();
        output.toByteArray()
    }
}
