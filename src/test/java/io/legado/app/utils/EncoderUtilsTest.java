package io.legado.app.utils;

import java.security.KeyPair;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class EncoderUtilsTest {

    @Test
    public void rsaMethodsRoundTripShortAndSegmentedText() throws Exception {
        KeyPair keys = EncoderUtils.INSTANCE.generateKeys();
        String shortText = "license payload";
        StringBuilder longTextBuilder = new StringBuilder();
        for (int index = 0; index < 80; index++) {
            longTextBuilder.append(shortText);
        }
        String longText = longTextBuilder.toString();

        assertEquals(shortText, EncoderUtils.INSTANCE.decryptByPrivateKey(
                EncoderUtils.INSTANCE.encryptByPublicKey(shortText, keys.getPublic()), keys.getPrivate()));
        assertEquals(shortText, EncoderUtils.INSTANCE.decryptByPublicKey(
                EncoderUtils.INSTANCE.encryptByPrivateKey(shortText, keys.getPrivate()), keys.getPublic()));
        assertEquals(longText, EncoderUtils.INSTANCE.decryptSegmentByPrivateKey(
                EncoderUtils.INSTANCE.encryptSegmentByPublicKey(longText, keys.getPublic(), 2048), keys.getPrivate(), 2048));
        assertEquals(longText, EncoderUtils.INSTANCE.decryptSegmentByPublicKey(
                EncoderUtils.INSTANCE.encryptSegmentByPrivateKey(longText, keys.getPrivate(), 2048), keys.getPublic(), 2048));
    }
}
