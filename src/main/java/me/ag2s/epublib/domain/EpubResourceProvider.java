package me.ag2s.epublib.domain;

import java.io.IOException;
import java.io.InputStream;
import java.io.File;
import me.ag2s.epublib.util.EpubArchivePolicy;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

/**
 * @author jake
 */
public class EpubResourceProvider implements LazyResourceProvider {

  private final String epubFilename;

  /**
   * @param epubFilename the file name for the epub we're created from.
   */
  public EpubResourceProvider(String epubFilename) {
    this.epubFilename = epubFilename;
  }

  @Override
  public InputStream getResourceStream(String href) throws IOException {
    ZipFile zipFile = EpubArchivePolicy.open(new File(epubFilename), EpubArchivePolicy.EPUB);
    try {
      EpubArchivePolicy.validateMetadata(zipFile, EpubArchivePolicy.EPUB);
      ZipEntry zipEntry = zipFile.getEntry(href);
      if (zipEntry == null) throw new IOException("EPUB 资源不存在");
      return new ResourceInputStream(EpubArchivePolicy.entryStream(zipFile, zipEntry,
          EpubArchivePolicy.EPUB, new EpubArchivePolicy.Budget(EpubArchivePolicy.EPUB.entryBytes)), zipFile);
    } catch (IOException | RuntimeException failure) {
      zipFile.close();
      throw failure;
    }
  }
}
