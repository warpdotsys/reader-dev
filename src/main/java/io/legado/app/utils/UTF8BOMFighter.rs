use crate::prelude::*;
pub struct UTF8BOMFighter;

impl UTF8BOMFighter {
    fn UTF8_BOM_BYTES() -> Vec<u8> {
        vec![0xEF, 0xBB, 0xBF]
    }

    pub fn removeUTF8BOM(xmlText: &str) -> String {
        let bytes = xmlText.as_bytes();
        let containsBOM = (bytes.len() > 3
            && bytes[0] == Self::UTF8_BOM_BYTES()[0]
            && bytes[1] == Self::UTF8_BOM_BYTES()[1]
            && bytes[2] == Self::UTF8_BOM_BYTES()[2]);
        if containsBOM {
            return String::from_utf8_lossy(&bytes[3..bytes.len()]).to_string();
        }
        xmlText.to_string()
    }

    pub fn removeUTF8BOM_bytes(bytes: &[u8]) -> Vec<u8> {
        let containsBOM = (bytes.len() > 3
            && bytes[0] == Self::UTF8_BOM_BYTES()[0]
            && bytes[1] == Self::UTF8_BOM_BYTES()[1]
            && bytes[2] == Self::UTF8_BOM_BYTES()[2]);
        if containsBOM {
            let mut copy = vec![0u8; bytes.len() - 3];
            System::arraycopy(bytes, 3, &mut copy, 0, bytes.len() - 3);
            return copy;
        }
        bytes.to_vec()
    }
}
