use std::io::Write;
use reader::me_ag2s_epublib_epub_epubreader::EpubReader;
use reader::me_ag2s_epublib_epub_resourcesloader::ZipFile;

#[test]
fn test_epub_read_lazy() {
    let dir = std::env::temp_dir().join(format!("epub_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("book.epub");
    let file = std::fs::File::create(&path).unwrap();
    let mut zw = zip::ZipWriter::new(file);
    zw.start_file("mimetype", zip::write::SimpleFileOptions::default()).unwrap();
    zw.write_all(b"application/epub+zip").unwrap();
    zw.start_file("OEBPS/content.opf", zip::write::SimpleFileOptions::default()).unwrap();
    let opf = "<?xml version=\"1.0\"?><package version=\"2.0\" xmlns=\"http://www.idpf.org/2007/opf\" unique-identifier=\"BookId\"><metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>测试书名</dc:title></metadata><manifest><item id=\"c1\" href=\"chap1.xhtml\" media-type=\"application/xhtml+xml\"/></manifest><spine><itemref idref=\"c1\"/></spine></package>";
    zw.write_all(opf.as_bytes()).unwrap();
    zw.start_file("OEBPS/chap1.xhtml", zip::write::SimpleFileOptions::default()).unwrap();
    zw.write_all("<html><head><title>c1</title></head><body><p>Hello 章节内容</p></body></html>".as_bytes()).unwrap();
    zw.finish().unwrap();

    let book = EpubReader::new()
        .read_epub_lazy(ZipFile::new(&path.to_string_lossy()), "utf-8")
        .expect("epub should read without panic");
    assert_eq!(book.get_title(), "测试书名", "epub title parsed");
    let contents = book.get_contents();
    assert!(!contents.is_empty(), "contents should be parsed");
    let data = contents
        .iter()
        .find(|r| r.get_href().contains("chap1.xhtml"))
        .map(|r| r.get_data().map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default())
        .unwrap_or_default();
    assert!(data.contains("Hello"), "chapter content should be read, got: {data}");
}

#[test]
fn test_zip_entry_iterator() {
    let dir = std::env::temp_dir().join(format!("zip_entries_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("a.zip");
    let file = std::fs::File::create(&path).unwrap();
    let mut zw = zip::ZipWriter::new(file);
    zw.start_file("d1/", zip::write::SimpleFileOptions::default()).unwrap();
    zw.write_all(b"").unwrap();
    zw.start_file("a.txt", zip::write::SimpleFileOptions::default()).unwrap();
    zw.write_all(b"hello").unwrap();
    zw.finish().unwrap();

    let zf = ZipFile::new(&path.to_string_lossy());
    let mut names = Vec::new();
    let mut total = 0u64;
    for e in zf.entries() {
        names.push(e.get_name());
        total += e.get_size();
        let _ = e.is_directory();
    }
    assert!(names.iter().any(|n| n == "a.txt"), "entries should list a.txt, got {names:?}");
    assert_eq!(total, 5, "a.txt content size");
    let stream = zf.get_input_stream(&zip_entry_by_name(&zf, "a.txt"));
    assert_eq!(stream, b"hello", "input stream should read content");
}

fn zip_entry_by_name(zf: &ZipFile, name: &str) -> reader::me_ag2s_epublib_epub_resourcesloader::ZipEntry {
    zf.entries().find(|e| e.get_name() == name).expect("entry found")
}

#[test]
fn test_dom_parse_opf() {
    let opf = "<?xml version=\"1.0\"?><package version=\"2.0\" xmlns=\"http://www.idpf.org/2007/opf\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" unique-identifier=\"BookId\"><metadata><dc:title>测试书名</dc:title></metadata><manifest><item id=\"c1\" href=\"chap1.xhtml\" media-type=\"application/xhtml+xml\"/></manifest><spine><itemref idref=\"c1\"/></spine></package>";
    let doc = reader::me_ag2s_epublib_epub_domutil::Document::parse(opf);
    let titles = doc.get_elements_by_tag_name("title");
    eprintln!("DBG titles len = {}", titles.get_length());
    let meta = doc.get_elements_by_tag_name("metadata");
    eprintln!("DBG metadata len = {}", meta.get_length());
    let items = doc.get_elements_by_tag_name("item");
    eprintln!("DBG item len = {}", items.get_length());
    let itemrefs = doc.get_elements_by_tag_name("itemref");
    eprintln!("DBG itemref len = {}", itemrefs.get_length());
    assert_eq!(titles.get_length(), 1, "title should be found");
}
#[test]
fn test_read_metadata_titles() {
    let opf = "<?xml version=\"1.0\"?><package version=\"2.0\" xmlns=\"http://www.idpf.org/2007/opf\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" unique-identifier=\"BookId\"><metadata><dc:title>测试书名</dc:title></metadata><manifest><item id=\"c1\" href=\"chap1.xhtml\" media-type=\"application/xhtml+xml\"/></manifest><spine><itemref idref=\"c1\"/></spine></package>";
    let doc = reader::me_ag2s_epublib_epub_domutil::Document::parse(opf);
    let root = doc.get_document_element();
    eprintln!("DBG root tag = {:?} null={}", root.tag_name, root.is_null());
    let md = reader::me_ag2s_epublib_epub_packagedocumentmetadatareader::PackageDocumentMetadataReader::read_metadata(&doc);
    eprintln!("DBG titles = {:?}", md.get_titles());
    let titles = md.get_titles();
    assert!(!titles.is_empty(), "metadata titles should not be empty");
}
#[test]
fn test_resourceutil_document_chain() {
    let opf = "<?xml version=\"1.0\"?><package version=\"2.0\"><metadata><dc:title>链路测试</dc:title></metadata></package>";
    let res = reader::me_ag2s_epublib_domain_resource::Resource::with_data_and_href(opf.as_bytes().to_vec(), "content.opf".to_string());
    let doc = reader::me_ag2s_epublib_util_resourceutil::ResourceUtil::get_as_document(&res).unwrap_or_else(|_| reader::me_ag2s_epublib_util_resourceutil::Document::new(String::new()));
    eprintln!("DBG html len = {}", doc.html.len());
    let dom = doc.to_dom_document();
    let titles = dom.get_elements_by_tag_name("title");
    eprintln!("DBG titles count = {}", titles.get_length());
    assert_eq!(titles.get_length(), 1, "full chain should find title");
}
#[test]
fn test_reader_read_all() {
    let opf = "<?xml version=\"1.0\"?><package version=\"2.0\"><metadata><dc:title>链路测试</dc:title></metadata></package>";
    let res = reader::me_ag2s_epublib_domain_resource::Resource::with_data_and_href(opf.as_bytes().to_vec(), "content.opf".to_string());
    let mut reader = res.get_reader().expect("reader");
    let mut text = String::new();
    let mut rounds = 0;
    loop {
        let mut buf = ['\0'; 64];
        let n = reader.read(&mut buf, 0, 64).unwrap_or(-1);
        rounds += 1;
        if n <= 0 { break; }
        text.push_str(&buf[..n as usize].iter().collect::<String>());
        if rounds > 100 { break; }
    }
    eprintln!("DBG rounds={} text len = {}", rounds, text.len());
    eprintln!("DBG text head = {}", &text[..text.len().min(60)]);
    assert!(text.contains("链路测试"), "reader should decode all content");
}
#[test]
fn test_epub_export() {
    use reader::me_ag2s_epublib_domain_epubbook::EpubBook;
    use reader::me_ag2s_epublib_domain_resource::Resource;
    use reader::me_ag2s_epublib_epub_epubwriter::{EpubWriter, OutputStream};
    let mut book = EpubBook::new();
    let mut md = reader::me_ag2s_epublib_domain_metadata::Metadata::new(); md.set_titles(vec!["导出书名".to_string()]); book.set_metadata(md);
    book.add_resource(Resource::with_data_and_href(b"<html><body>content</body></html>".to_vec(), "text/chap1.xhtml".to_string()));
    let out = OutputStream::new_for_file(String::new());
    let data_rc = out.data.clone();
    EpubWriter::new().write(book, out).expect("epub write");
    let bytes = data_rc.borrow().clone();
    assert!(!bytes.is_empty(), "epub output non-empty");
    // 解压检查关键文件
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip parse");
    let mut names = Vec::new();
    for i in 0..archive.len() {
        if let Ok(e) = archive.by_index(i) {
            names.push(e.name().to_string());
        }
    }
    assert!(names.iter().any(|n| n == "mimetype"), "mimetype entry: {names:?}");
    assert!(names.iter().any(|n| n.contains("content.opf")), "opf entry: {names:?}");
    let opf = archive.by_name("OEBPS/content.opf").map(|mut e| { let mut s = String::new(); use std::io::Read; let _ = e.read_to_string(&mut s); s }).unwrap_or_default();
    assert!(opf.contains("<package") && opf.contains("<metadata"), "opf should contain package/metadata, got: {opf}");
    assert!(opf.contains("导出书名"), "opf should contain title");
}