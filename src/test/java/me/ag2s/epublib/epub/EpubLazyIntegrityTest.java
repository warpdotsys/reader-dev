package me.ag2s.epublib.epub;

import me.ag2s.epublib.domain.MediaTypes;
import me.ag2s.epublib.domain.Resources;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import java.io.File;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.util.Arrays;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipOutputStream;

import static org.junit.Assert.*;

/** Small generated payloads only: never allocates a production-sized ZIP bomb. */
public class EpubLazyIntegrityTest {
    @Rule public TemporaryFolder temporary = new TemporaryFolder();

    private File alteredArchive(int centralOffset, int value) throws Exception {
        File file = temporary.newFile("generated.epub");
        byte[] body = new byte[128 * 1024];
        Arrays.fill(body, (byte) 'x');
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(file.toPath()))) {
            zip.putNextEntry(new ZipEntry("OEBPS/chapter.xhtml"));
            zip.write(body);
            zip.closeEntry();
        }
        byte[] bytes = Files.readAllBytes(file.toPath());
        ByteBuffer view = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN);
        boolean found = false;
        for (int i = 0; i + 46 <= bytes.length; i++) {
            if (view.getInt(i) == 0x02014b50) {
                view.putInt(i + centralOffset, value);
                found = true;
                break;
            }
        }
        assertTrue(found);
        Files.write(file.toPath(), bytes);
        return file;
    }

    private void mustReject(File file) throws Exception {
        try (ZipFile zip = new ZipFile(file)) {
            Resources resources = ResourcesLoader.loadResources(zip, "utf-8", Arrays.asList(MediaTypes.mediaTypes));
            try {
                resources.getByHref("OEBPS/chapter.xhtml").getData();
                fail("Untrusted resource bytes must be rejected, not returned as success");
            } catch (IOException expected) {
                assertNotNull(expected.getMessage());
            }
        }
    }

    @Test public void lazyDataDoesNotTrustUnderreportedCentralSize() throws Exception {
        mustReject(alteredArchive(24, 1));
    }

    @Test public void lazyDataChecksTheActualCrc() throws Exception {
        mustReject(alteredArchive(16, 1));
    }
}
