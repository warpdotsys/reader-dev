package me.ag2s.epublib.util;

import java.io.File;
import java.io.FilterInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.RandomAccessFile;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.net.URLDecoder;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayDeque;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.zip.CRC32;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

/** Finite archive budgets; errors contain no book text or resource names. */
public final class EpubArchivePolicy {
    private EpubArchivePolicy() {}
    public static final long MIB = 1024L * 1024;
    public static final Limits EPUB = new Limits(256 * MIB, 1024 * MIB, 32 * MIB, 16 * MIB, 10000);
    // Backups/CBZ share the old unzip helper; do not impose EPUB text limits on them.
    public static final Limits GENERAL_ZIP = new Limits(2048 * MIB, 4096 * MIB, 1024 * MIB, 1024 * MIB, 100000);
    public static final long MAX_EAGER_BYTES = 64 * MIB;
    public static final long MAX_CENTRAL_DIRECTORY_BYTES = 16 * MIB;

    public static final class Limits {
        public final long archiveBytes, expandedBytes, entryBytes, textBytes;
        public final int entries;
        public Limits(long archiveBytes, long expandedBytes, long entryBytes, long textBytes, int entries) {
            if (archiveBytes <= 0 || expandedBytes <= 0 || entryBytes <= 0 || textBytes <= 0 || entries <= 0) {
                throw new IllegalArgumentException("Archive limits must be positive");
            }
            this.archiveBytes = archiveBytes;
            this.expandedBytes = expandedBytes;
            this.entryBytes = entryBytes;
            this.textBytes = textBytes;
            this.entries = entries;
        }
    }

    public static final class ArchiveException extends IOException {
        public ArchiveException(String message) { super(message); }
    }

    public static final class Budget {
        private final long maximum;
        private long consumed;
        private final long started = System.nanoTime();
        public Budget(long maximum) { this.maximum = maximum; }
        public long remaining() { return maximum - consumed; }
        private void count(int bytes) throws ArchiveException {
            if (bytes > remaining()) throw new ArchiveException("EPUB/ZIP 实际展开总量超过服务端上限");
            consumed += bytes;
        }
        private void checkTime() throws ArchiveException {
            // Cooperative local-I/O deadline, not a guarantee against a blocked OS read.
            if (System.nanoTime() - started > 30_000_000_000L) {
                throw new ArchiveException("EPUB/ZIP 读取超过服务端时间预算");
            }
        }
    }

    public static ZipFile open(File source, Limits limits) throws IOException {
        if (!source.isFile() || source.length() > limits.archiveBytes) {
            throw new ArchiveException("EPUB/ZIP 压缩文件超过服务端上限或不是普通文件");
        }
        preflightDirectory(source, limits);
        return new ZipFile(source);
    }

    /** Bound the directory before ZipFile allocates its index; no large test payload is needed. */
    private static void preflightDirectory(File source, Limits limits) throws IOException {
        try (RandomAccessFile input = new RandomAccessFile(source, "r")) {
            long size = input.length();
            if (size < 22 || size > limits.archiveBytes) throw new ArchiveException("EPUB/ZIP 文件大小无效");
            byte[] tail = new byte[(int) Math.min(size, 65557)];
            input.seek(size - tail.length);
            input.readFully(tail);
            ByteBuffer view = ByteBuffer.wrap(tail).order(ByteOrder.LITTLE_ENDIAN);
            int end = -1;
            for (int i = tail.length - 22; i >= 0; i--) {
                if (view.getInt(i) == 0x06054b50 && i + 22 + Short.toUnsignedInt(view.getShort(i + 20)) == tail.length) {
                    end = i;
                    break;
                }
            }
            if (end < 0) throw new ArchiveException("EPUB/ZIP 缺少完整结束目录");
            long count = Short.toUnsignedInt(view.getShort(end + 10));
            long directoryBytes = Integer.toUnsignedLong(view.getInt(end + 12));
            long directoryOffset = Integer.toUnsignedLong(view.getInt(end + 16));
            if (Short.toUnsignedInt(view.getShort(end + 4)) != 0 || Short.toUnsignedInt(view.getShort(end + 6)) != 0) {
                throw new ArchiveException("EPUB/ZIP 不支持分卷压缩包");
            }
            if (count == 65535 || directoryBytes == 0xffffffffL || directoryOffset == 0xffffffffL) {
                long endOffset = size - tail.length + end;
                if (endOffset < 20) throw new ArchiveException("EPUB/ZIP ZIP64 目录无效");
                byte[] locator = new byte[20];
                input.seek(endOffset - 20);
                input.readFully(locator);
                ByteBuffer located = ByteBuffer.wrap(locator).order(ByteOrder.LITTLE_ENDIAN);
                long recordOffset = located.getLong(8);
                if (located.getInt(0) != 0x07064b50 || located.getInt(4) != 0 || located.getInt(16) != 1
                    || recordOffset < 0 || recordOffset > size - 56) throw new ArchiveException("EPUB/ZIP ZIP64 定位信息无效");
                byte[] record = new byte[56];
                input.seek(recordOffset);
                input.readFully(record);
                ByteBuffer data = ByteBuffer.wrap(record).order(ByteOrder.LITTLE_ENDIAN);
                if (data.getInt(0) != 0x06064b50 || data.getLong(4) < 44 || data.getLong(4) > size - recordOffset - 12
                    || data.getInt(16) != 0 || data.getInt(20) != 0 || data.getLong(24) != data.getLong(32)) {
                    throw new ArchiveException("EPUB/ZIP ZIP64 结束目录无效");
                }
                count = data.getLong(32);
                directoryBytes = data.getLong(40);
                directoryOffset = data.getLong(48);
            } else if (Short.toUnsignedInt(view.getShort(end + 8)) != count) {
                throw new ArchiveException("EPUB/ZIP 目录条目计数不一致");
            }
            if (count < 0 || count > limits.entries || directoryBytes < 0 || directoryBytes > MAX_CENTRAL_DIRECTORY_BYTES) {
                throw new ArchiveException("EPUB/ZIP 中央目录大小或条目数超过服务端上限");
            }
            if (directoryOffset < 0 || directoryOffset > size || directoryBytes > size - directoryOffset) {
                throw new ArchiveException("EPUB/ZIP 中央目录位置无效");
            }
        }
    }

    /** Decode aliases only for validation. Actual ZIP names/bytes remain unchanged. */
    public static String entryKey(String name) throws IOException {
        String path = name.replace('\\', '/');
        try { path = URLDecoder.decode(path.replace("+", "%2B"), "UTF-8"); }
        catch (IllegalArgumentException ignored) { /* literal percent in a legacy filename */ }
        path = path.replace('\\', '/');
        if (path.startsWith("/") || path.indexOf('\0') >= 0 || path.matches("^[a-zA-Z][a-zA-Z0-9+.-]*:.*")) {
            throw new ArchiveException("EPUB/ZIP 条目路径无效");
        }
        ArrayDeque<String> parts = new ArrayDeque<>();
        for (String part : path.split("/")) {
            if (part.isEmpty() || part.equals(".")) continue;
            if (part.equals("..")) {
                if (parts.isEmpty()) throw new ArchiveException("EPUB/ZIP 条目越过根目录");
                parts.removeLast();
            } else parts.addLast(part);
        }
        if (parts.isEmpty()) throw new ArchiveException("EPUB/ZIP 条目路径为空");
        return String.join("/", parts);
    }

    public static long entryLimit(String name, Limits limits) {
        String lower = name.toLowerCase(Locale.ROOT);
        return lower.matches(".*\\.(xml|opf|ncx|x?html?|css|svg)$")
            ? Math.min(limits.entryBytes, limits.textBytes) : limits.entryBytes;
    }

    public static Map<String, ZipEntry> validateMetadata(ZipFile zip, Limits limits) throws IOException {
        if (new File(zip.getName()).length() > limits.archiveBytes) {
            throw new ArchiveException("EPUB/ZIP 压缩文件超过服务端上限");
        }
        Map<String, ZipEntry> result = new LinkedHashMap<>();
        Set<String> files = new HashSet<>(), directories = new HashSet<>();
        long total = 0;
        java.util.Enumeration<? extends ZipEntry> entries = zip.entries();
        while (entries.hasMoreElements()) {
            ZipEntry entry = entries.nextElement();
            String key = entryKey(entry.getName());
            if (result.size() >= limits.entries || result.putIfAbsent(key, entry) != null) {
                throw new ArchiveException("EPUB/ZIP 条目数量超限或存在重复路径");
            }
            for (int slash = key.indexOf('/'); slash >= 0; slash = key.indexOf('/', slash + 1)) {
                String parent = key.substring(0, slash);
                if (files.contains(parent)) throw new ArchiveException("EPUB/ZIP 文件与目录路径冲突");
                directories.add(parent);
            }
            if (entry.isDirectory()) {
                if (files.contains(key)) throw new ArchiveException("EPUB/ZIP 文件与目录路径冲突");
                directories.add(key);
            } else {
                if (directories.contains(key)) throw new ArchiveException("EPUB/ZIP 文件与目录路径冲突");
                files.add(key);
            }
            if ((entry.getMethod() != ZipEntry.STORED && entry.getMethod() != ZipEntry.DEFLATED)
                || entry.getSize() < 0 || entry.getCompressedSize() < 0 || entry.getCrc() < 0
                || entry.getCompressedSize() > limits.archiveBytes
                || entry.getSize() > entryLimit(key, limits)) {
                throw new ArchiveException("EPUB/ZIP 单个资源大小或压缩信息不符合服务端限制");
            }
            if (entry.getSize() > limits.expandedBytes - total) {
                throw new ArchiveException("EPUB/ZIP 声明展开总量超过服务端上限");
            }
            total += entry.getSize();
        }
        return result;
    }

    public static InputStream entryStream(ZipFile zip, ZipEntry entry, Limits limits, Budget budget) throws IOException {
        return bounded(zip.getInputStream(entry), entry, limits, budget, true);
    }

    /** Streaming ZIP entries may obtain size/CRC from their data descriptor at EOF. */
    public static InputStream bounded(InputStream input, ZipEntry entry, Limits limits, Budget budget,
                                      boolean closeInput) throws IOException {
        long maximum = entryLimit(entryKey(entry.getName()), limits);
        if (entry.getSize() > maximum) throw new ArchiveException("EPUB/ZIP 单个资源超过服务端上限");
        return new FilterInputStream(input) {
            private long count;
            private final CRC32 crc = new CRC32();
            private boolean ended;
            private void finished() throws IOException {
                ended = true;
                if ((entry.getSize() >= 0 && count != entry.getSize())
                    || (entry.getCrc() >= 0 && crc.getValue() != entry.getCrc())) {
                    throw new ArchiveException("EPUB/ZIP 实际资源大小或 CRC 与声明不一致");
                }
            }
            @Override public int read() throws IOException {
                byte[] one = new byte[1];
                return read(one, 0, 1) < 0 ? -1 : one[0] & 255;
            }
            @Override public int read(byte[] bytes, int offset, int length) throws IOException {
                if (length == 0) return 0;
                if (ended) return -1;
                budget.checkTime();
                int read = in.read(bytes, offset, (int) Math.min(length, maximum - count + 1));
                if (read < 0) { finished(); return -1; }
                count += read;
                if (count > maximum || (entry.getSize() >= 0 && count > entry.getSize())) {
                    throw new ArchiveException("EPUB/ZIP 实际资源大小超过声明或服务端上限");
                }
                budget.count(read);
                crc.update(bytes, offset, read);
                return read;
            }
            @Override public long skip(long amount) throws IOException {
                byte[] buffer = new byte[8192];
                long skipped = 0;
                while (skipped < amount) {
                    int read = read(buffer, 0, (int) Math.min(buffer.length, amount - skipped));
                    if (read < 0) break;
                    skipped += read;
                }
                return skipped;
            }
            @Override public boolean markSupported() { return false; }
            @Override public void reset() throws IOException { throw new IOException("Archive resource is not rewindable"); }
            @Override public void close() throws IOException { if (closeInput) in.close(); }
        };
    }

    public static void validate(File source) throws IOException {
        try (ZipFile zip = open(source, EPUB)) {
            validate(zip);
        }
    }

    public static void validate(ZipFile zip) throws IOException {
        Map<String, ZipEntry> entries = validateMetadata(zip, EPUB);
        Budget budget = new Budget(EPUB.expandedBytes);
        byte[] buffer = new byte[8192];
        for (ZipEntry entry : entries.values()) {
            try (InputStream input = entryStream(zip, entry, EPUB, budget)) {
                while (input.read(buffer) >= 0) { /* Verify, never print or retain book bytes. */ }
            }
        }
    }

    public static void noSymbolicParents(Path path) throws IOException {
        for (Path current = path.toAbsolutePath().normalize(); current != null; current = current.getParent()) {
            if (Files.isSymbolicLink(current)) throw new ArchiveException("EPUB/ZIP 缓存路径不能经过符号链接");
        }
    }

    public static Path extractionTarget(Path root, String name) throws IOException {
        entryKey(name); // Also reject encoded traversal/protocol aliases.
        Path base = root.toAbsolutePath().normalize();
        Path target = base.resolve(name.replace('\\', '/')).normalize();
        if (!target.startsWith(base) || target.equals(base)) throw new ArchiveException("EPUB/ZIP 解包路径越界");
        noSymbolicParents(target);
        return target;
    }
}
