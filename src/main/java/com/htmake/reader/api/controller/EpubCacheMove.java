package com.htmake.reader.api.controller;

import java.io.IOException;
import java.io.InterruptedIOException;
import java.nio.file.AccessDeniedException;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;

/** Retry only a denied rename of an owned EPUB cache; never copy, replace or relax ACLs. */
public final class EpubCacheMove {
    private EpubCacheMove() { }

    @FunctionalInterface interface Move { void run(Path source, Path target) throws IOException; }
    @FunctionalInterface interface Pause { void run(long milliseconds) throws InterruptedException; }

    public static void move(Path source, Path target) throws IOException {
        move(source, target, (from, to) -> Files.move(from, to), Thread::sleep);
    }

    static void move(Path source, Path target, Move operation, Pause pause) throws IOException {
        for (int attempt = 0; ; attempt++) {
            try {
                operation.run(source, target);
                return;
            } catch (AccessDeniedException denied) {
                // Windows can briefly deny a just-written directory rename. The cause may also
                // be a persistent ACL denial: retry at most 5 times / 1500 ms, never bypass it.
                if (attempt >= 4 || !Files.exists(source, LinkOption.NOFOLLOW_LINKS)
                        || Files.exists(target, LinkOption.NOFOLLOW_LINKS)) throw denied;
                try {
                    pause.run(100L << attempt);
                } catch (InterruptedException interrupted) {
                    Thread.currentThread().interrupt();
                    InterruptedIOException failure = new InterruptedIOException("EPUB cache rename retry interrupted");
                    failure.initCause(interrupted);
                    failure.addSuppressed(denied);
                    throw failure;
                }
            }
        }
    }
}
