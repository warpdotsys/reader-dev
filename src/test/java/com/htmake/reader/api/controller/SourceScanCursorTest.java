package com.htmake.reader.api.controller;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class SourceScanCursorTest {

    @Test
    public void reportsCompleteAfterLastZeroBasedSource() {
        assertFalse(SourceScanCursor.isComplete(-1, 1));
        assertTrue(SourceScanCursor.isComplete(0, 1));
        assertFalse(SourceScanCursor.isComplete(0, 2));
        assertTrue(SourceScanCursor.isComplete(1, 2));
    }

    @Test
    public void emptySourceSetIsNotACompletedScan() {
        assertFalse(SourceScanCursor.isComplete(-1, 0));
    }
}
