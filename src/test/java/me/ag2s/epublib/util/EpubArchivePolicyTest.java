package me.ag2s.epublib.util;

import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import java.io.File;
import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipOutputStream;
import static org.junit.Assert.*;

public class EpubArchivePolicyTest {
    @Rule public TemporaryFolder temporary = new TemporaryFolder();
    private interface Checked { void run() throws Exception; }
    private void rejects(Checked action) throws Exception {
        try { action.run(); fail("Expected a real archive error"); }
        catch (EpubArchivePolicy.ArchiveException expected) { assertNotNull(expected.getMessage()); }
    }
    private File archive(String... names) throws Exception {
        File file = temporary.newFile();
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(file.toPath()))) {
            for (String name : names) {
                zip.putNextEntry(new ZipEntry(name));
                zip.write(new byte[100]);
                zip.closeEntry();
            }
        }
        return file;
    }
    @Test public void compressedSizeCheckedBeforeOpeningTheArchive() throws Exception {
        File source = archive("a.bin");
        rejects(() -> EpubArchivePolicy.open(source, new EpubArchivePolicy.Limits(1, 1000, 1000, 1000, 10)));
    }
    @Test public void perFileTextTotalAndEntryBudgetsAreIndependent() throws Exception {
        File source = archive("a.xhtml", "b.bin");
        try (ZipFile zip = new ZipFile(source)) {
            rejects(() -> EpubArchivePolicy.validateMetadata(zip, new EpubArchivePolicy.Limits(10000, 1000, 99, 99, 10)));
            rejects(() -> EpubArchivePolicy.validateMetadata(zip, new EpubArchivePolicy.Limits(10000, 1000, 1000, 99, 10)));
            rejects(() -> EpubArchivePolicy.validateMetadata(zip, new EpubArchivePolicy.Limits(10000, 199, 1000, 1000, 10)));
            rejects(() -> EpubArchivePolicy.validateMetadata(zip, new EpubArchivePolicy.Limits(10000, 1000, 1000, 1000, 1)));
            assertEquals(2, EpubArchivePolicy.validateMetadata(zip, EpubArchivePolicy.EPUB).size());
        }
    }
    @Test public void decodedAliasesAndFileDirectoryConflictsAreRejected() throws Exception {
        for (File source : new File[] {archive("a.xhtml", "%61.xhtml"), archive("folder/child", "folder")}) {
            try (ZipFile zip = new ZipFile(source)) { rejects(() -> EpubArchivePolicy.validateMetadata(zip, EpubArchivePolicy.EPUB)); }
        }
    }
    @Test public void protocolsAbsolutePathsEncodedAndBackslashTraversalAreRejected() throws Exception {
        for (String path : new String[] {"../a", "..\\a", "%2e%2e/a", "/a", "%2foutside", "https:evil", "C:\\outside", "a\0b"}) {
            rejects(() -> EpubArchivePolicy.entryKey(path));
        }
        assertEquals("book/a+b.xhtml", EpubArchivePolicy.entryKey("book/a+b.xhtml"));
        assertEquals("book/a.xhtml", EpubArchivePolicy.entryKey("book/x/../a.xhtml"));
    }
    @Test public void unknownStreamingSizeIsStillBoundedByActualBytes() throws Exception {
        ZipEntry entry = new ZipEntry("generated.bin");
        EpubArchivePolicy.Limits limits = new EpubArchivePolicy.Limits(1000, 1000, 16, 16, 10);
        try (InputStream input = EpubArchivePolicy.bounded(new ByteArrayInputStream(new byte[100]), entry,
                limits, new EpubArchivePolicy.Budget(1000), true)) {
            rejects(() -> input.readAllBytes());
        }
    }
    @Test public void aggregateActualBudgetFailsWithoutSilentlyTruncatingData() throws Exception {
        ZipEntry entry = new ZipEntry("generated.bin");
        try (InputStream input = EpubArchivePolicy.bounded(new ByteArrayInputStream(new byte[100]), entry,
                EpubArchivePolicy.EPUB, new EpubArchivePolicy.Budget(16), true)) {
            rejects(() -> input.readAllBytes());
        }
    }
    @Test public void excessiveCentralIndexIsRejectedBeforeZipFileAllocation() throws Exception {
        File source = archive("a.bin");
        byte[] bytes = Files.readAllBytes(source.toPath());
        java.nio.ByteBuffer view = java.nio.ByteBuffer.wrap(bytes).order(java.nio.ByteOrder.LITTLE_ENDIAN);
        view.putInt(bytes.length - 22 + 12, (int) EpubArchivePolicy.MAX_CENTRAL_DIRECTORY_BYTES + 1);
        Files.write(source.toPath(), bytes);
        rejects(() -> EpubArchivePolicy.open(source, EpubArchivePolicy.EPUB));
    }
    @Test public void smallZip64DirectoryRemainsReadable() throws Exception {
        File source = archive("a.bin");
        byte[] classic = Files.readAllBytes(source.toPath());
        int end = classic.length - 22;
        java.nio.ByteBuffer old = java.nio.ByteBuffer.wrap(classic).order(java.nio.ByteOrder.LITTLE_ENDIAN);
        java.nio.ByteBuffer zip64 = java.nio.ByteBuffer.allocate(classic.length + 76)
                .order(java.nio.ByteOrder.LITTLE_ENDIAN);
        zip64.put(classic, 0, end);
        zip64.putInt(0x06064b50).putLong(44).putShort((short) 45).putShort((short) 45);
        zip64.putInt(0).putInt(0).putLong(1).putLong(1);
        zip64.putLong(Integer.toUnsignedLong(old.getInt(end + 12)));
        zip64.putLong(Integer.toUnsignedLong(old.getInt(end + 16)));
        zip64.putInt(0x07064b50).putInt(0).putLong(end).putInt(1);
        zip64.put(classic, end, 22);
        int newEnd = end + 76;
        zip64.putShort(newEnd + 8, (short) 65535).putShort(newEnd + 10, (short) 65535);
        zip64.putInt(newEnd + 12, -1).putInt(newEnd + 16, -1);
        Files.write(source.toPath(), zip64.array());
        EpubArchivePolicy.validate(source);
        try (ZipFile zip = EpubArchivePolicy.open(source, EpubArchivePolicy.EPUB)) {
            assertEquals(1, zip.size());
        }
    }
    @Test public void commentedArchiveAndStoredChineseNamesRemainReadable() throws Exception {
        File source = temporary.newFile();
        byte[] body = "Generated test only".getBytes(java.nio.charset.StandardCharsets.UTF_8);
        java.util.zip.CRC32 crc = new java.util.zip.CRC32();
        crc.update(body);
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(source.toPath()))) {
            zip.setComment("生成注释 PK\u0005\u0006 不是真正结束目录");
            ZipEntry entry = new ZipEntry("书籍/章节+a.xhtml");
            entry.setMethod(ZipEntry.STORED);
            entry.setSize(body.length);
            entry.setCrc(crc.getValue());
            zip.putNextEntry(entry);
            zip.write(body);
            zip.closeEntry();
        }
        EpubArchivePolicy.validate(source);
    }
    @Test public void truncatedAndInconsistentEndRecordsFailClosed() throws Exception {
        File source = archive("a.bin");
        byte[] original = Files.readAllBytes(source.toPath());
        Files.write(source.toPath(), java.util.Arrays.copyOf(original, original.length - 1));
        rejects(() -> EpubArchivePolicy.open(source, EpubArchivePolicy.EPUB));
        java.nio.ByteBuffer view = java.nio.ByteBuffer.wrap(original).order(java.nio.ByteOrder.LITTLE_ENDIAN);
        view.putShort(original.length - 22 + 8, (short) 2);
        Files.write(source.toPath(), original);
        rejects(() -> EpubArchivePolicy.open(source, EpubArchivePolicy.EPUB));
    }
}
