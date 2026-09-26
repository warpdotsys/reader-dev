package com.htmake.reader.api.controller;

import kotlin.coroutines.EmptyCoroutineContext;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.junit4.SpringRunner;

import java.io.File;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

/** Regression coverage for Windows storage/data user homes and link escapes. */
@RunWith(SpringRunner.class)
@SpringBootTest
public class FileControllerPathTest {
    private FileController controller;

    @Before
    public void createControllerAfterSpringContextIsReady() {
        controller = new FileController(EmptyCoroutineContext.INSTANCE);
    }

    @Test
    public void resolvesRootAndNewFileUnderAnExistingUserHome() throws Exception {
        Path home = Files.createTempDirectory("reader-file-home-");
        try {
            File root = resolve(home, "/");
            File diagnostic = resolve(home, "/diag.txt");

            assertEquals(home.toRealPath(), root.toPath());
            assertEquals(home.toRealPath().resolve("diag.txt"), diagnostic.toPath());
            assertTrue("new children must retain their verified user-home ancestor",
                    diagnostic.toPath().startsWith(home.toRealPath()));
        } finally {
            deleteTree(home);
        }
    }

    @Test
    public void rejectsExistingLinkThatLeavesUserHomeWhenSupported() throws Exception {
        Path home = Files.createTempDirectory("reader-file-home-");
        Path outside = Files.createTempDirectory("reader-file-outside-");
        try {
            Path link = home.resolve("escape");
            try {
                Files.createSymbolicLink(link, outside);
            } catch (UnsupportedOperationException | SecurityException ignored) {
                return;
            } catch (java.io.IOException ignored) {
                // Windows may deny symlink creation when Developer Mode/privileges are absent.
                return;
            }
            assertNull("a symlink or junction must not expose data outside this home",
                    resolve(home, "/escape/secret.txt"));
        } finally {
            deleteTree(home);
            deleteTree(outside);
        }
    }

    private File resolve(Path home, String requestPath) throws Exception {
        Method method = FileController.class.getDeclaredMethod("resolveSecurePath", File.class, String.class);
        method.setAccessible(true);
        return (File) method.invoke(controller, home.toFile(), requestPath);
    }

    private static void deleteTree(Path root) throws java.io.IOException {
        if (!Files.exists(root)) return;
        try (java.util.stream.Stream<Path> paths = Files.walk(root)) {
            paths.sorted(Comparator.reverseOrder()).forEach(path -> {
                try {
                    Files.deleteIfExists(path);
                } catch (java.io.IOException e) {
                    throw new java.io.UncheckedIOException(e);
                }
            });
        }
    }
}
