package com.htmake.reader.entity;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class LicenseTest {

    @Test
    public void wildcardHostMatchesOneLabelAndIgnoresPort() {
        License license = new License();
        license.setHost("*.example.com,reader.internal");

        assertTrue(license.validHost("api.example.com:8080"));
        assertTrue(license.validHost("reader.internal"));
        assertFalse(license.validHost("example.com"));
        assertFalse(license.validHost("deep.api.example.com"));
    }

    @Test
    public void expiredLicenseDoesNotValidateAnyHost() {
        License license = new License();
        license.setHost("*");
        license.setExpiredAt(System.currentTimeMillis() - 1);

        assertFalse(license.validHost("reader.example.com"));
    }
}
