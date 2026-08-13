// 真实 zip 解压（zip crate），供 File::unzip / ZipUtils 使用

use std::io::Read;

pub fn unzip_to(zip_path: &str, dest: &str) -> bool {
    let file = match std::fs::File::open(zip_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return false,
    };
    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.mangled_name();
        let outpath = std::path::Path::new(dest).join(name);
        if entry.is_dir() {
            if std::fs::create_dir_all(&outpath).is_err() {
                return false;
            }
            continue;
        }
        if let Some(parent) = outpath.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return false;
            }
        }
        let mut out = match std::fs::File::create(&outpath) {
            Ok(f) => f,
            Err(_) => return false,
        };
        let mut buf = Vec::new();
        if entry.read_to_end(&mut buf).is_err() {
            return false;
        }
        if std::io::Write::write_all(&mut out, &buf).is_err() {
            return false;
        }
    }
    true
}

pub fn zip_from_dir(src_dir: &str, zip_path: &str) -> bool {
    let file = match std::fs::File::create(zip_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let base = std::path::Path::new(src_dir);
    let mut entries: Vec<String> = Vec::new();
    collect_files(base, base, &mut entries);
    for rel in entries {
        let full = base.join(&rel);
        let name = rel.replace('\\', "/");
        if full.is_dir() {
            let _ = zip.add_directory(&name, options);
        } else if let Ok(bytes) = std::fs::read(&full) {
            let _ = zip.start_file(&name, options);
            let _ = std::io::Write::write_all(&mut zip, &bytes);
        }
    }
    match zip.finish() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn collect_files(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().to_string();
            if path.is_dir() {
                out.push(rel);
                collect_files(base, &path, out);
            } else {
                out.push(rel);
            }
        }
    }
}
