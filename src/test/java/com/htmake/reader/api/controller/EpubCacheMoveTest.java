package com.htmake.reader.api.controller;

import java.io.InterruptedIOException;
import java.nio.file.AccessDeniedException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import static org.junit.Assert.*;

public class EpubCacheMoveTest {
    @Rule public TemporaryFolder temporary = new TemporaryFolder();

    @Test public void temporaryDenialRetriesTheRenameWithoutCopying() throws Exception {
        Path source = temporary.newFolder("stage").toPath();
        Files.write(source.resolve("generated.txt"), new byte[] {1, 2, 3});
        Path target = source.resolveSibling("cache");
        AtomicInteger attempts = new AtomicInteger();
        List<Long> waits = new ArrayList<>();
        EpubCacheMove.move(source, target, (from, to) -> {
            if (attempts.incrementAndGet() < 3) throw new AccessDeniedException(from.toString());
            Files.move(from, to);
        }, waits::add);
        assertEquals(3, attempts.get());
        assertEquals(Arrays.asList(100L, 200L), waits);
        assertFalse(Files.exists(source));
        assertArrayEquals(new byte[] {1, 2, 3}, Files.readAllBytes(target.resolve("generated.txt")));
    }

    @Test public void permanentDenialHasFiniteAttemptsAndPreservesSource() throws Exception {
        Path source = temporary.newFolder("stage").toPath();
        Path target = source.resolveSibling("cache");
        AtomicInteger attempts = new AtomicInteger();
        List<Long> waits = new ArrayList<>();
        try {
            EpubCacheMove.move(source, target, (from, to) -> {
                attempts.incrementAndGet(); throw new AccessDeniedException(from.toString());
            }, waits::add);
            fail("Persistent permission denial must fail");
        } catch (AccessDeniedException expected) { }
        assertEquals(5, attempts.get());
        assertEquals(Arrays.asList(100L, 200L, 400L, 800L), waits);
        assertTrue(Files.isDirectory(source));
        assertFalse(Files.exists(target));
    }

    @Test public void targetAppearingDuringDenialIsNeverReplaced() throws Exception {
        Path source = temporary.newFolder("stage").toPath();
        Path target = source.resolveSibling("cache");
        AtomicInteger attempts = new AtomicInteger();
        try {
            EpubCacheMove.move(source, target, (from, to) -> {
                attempts.incrementAndGet(); Files.createDirectory(to);
                Files.write(to.resolve("keep.txt"), new byte[] {7});
                throw new AccessDeniedException(from.toString());
            }, milliseconds -> fail("An existing target must not be retried"));
            fail("Must retain concurrently created target");
        } catch (AccessDeniedException expected) { }
        assertEquals(1, attempts.get());
        assertTrue(Files.isDirectory(source));
        assertArrayEquals(new byte[] {7}, Files.readAllBytes(target.resolve("keep.txt")));
    }

    @Test public void unrelatedIoFailureIsNotRetried() throws Exception {
        Path source = temporary.newFolder("stage").toPath();
        try {
            EpubCacheMove.move(source, source.resolveSibling("cache"),
                    (from, to) -> { throw new FileAlreadyExistsException(to.toString()); },
                    milliseconds -> fail("Only access denial is retried"));
            fail("Must propagate unrelated failure");
        } catch (FileAlreadyExistsException expected) { }
        assertTrue(Files.isDirectory(source));
    }

    @Test public void interruptionStopsRetryAndPreservesInterruptFlag() throws Exception {
        Path source = temporary.newFolder("stage").toPath();
        try {
            EpubCacheMove.move(source, source.resolveSibling("cache"),
                    (from, to) -> { throw new AccessDeniedException(from.toString()); },
                    milliseconds -> { throw new InterruptedException("generated interruption"); });
            fail("Interrupted retry must stop");
        } catch (InterruptedIOException expected) {
            assertTrue(Thread.currentThread().isInterrupted());
            assertTrue(expected.getCause() instanceof InterruptedException);
            assertEquals(1, expected.getSuppressed().length);
        } finally { Thread.interrupted(); }
        assertTrue(Files.isDirectory(source));
    }
}
