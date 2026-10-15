"""Deterministic, wholly synthetic EPUB; never takes a book or body as input."""
from dataclasses import dataclass
import hashlib
from pathlib import Path
import xml.etree.ElementTree as ET
import zipfile

MIB = 1024 * 1024
FIXTURE_VERSION = 'reader-generated-epub-pressure-v1'


@dataclass(frozen=True)
class Profile:
    chapters: int
    chapter_bytes: int
    styles: int
    style_bytes: int
    binary_blocks: int
    binary_bytes: int


PRESSURE = Profile(850, 60 * 1024, 800, 4096, 36, MIB)
SMALL = Profile(3, 60 * 1024, 2, 4096, 1, 65536)


def title(index):
    return f'第{index + 1:04d}章 纯生成压力回归'


def chapter(index, size):
    heading = title(index)
    opening = ('<?xml version="1.0" encoding="UTF-8"?>'
               '<html xmlns="http://www.w3.org/1999/xhtml"><head>'
               f'<title>{heading}</title><link rel="stylesheet" href="../styles/s0000.css"/>'
               f'</head><body><h1>{heading}</h1>')
    text = '这里只包含程序生成的中文与 UTF-8 阅读回归样本。切换、刷新和标题必须对应请求章节。'
    paragraphs = ''.join(f'<p>生成章节{index:04d}段落{p:03d}：' + text * 3 + '</p>' for p in range(64))
    prefix, suffix = (opening + paragraphs + '<!--').encode(), b'--></body></html>'
    padding = size - len(prefix) - len(suffix)
    if padding < 0:
        raise ValueError('Synthetic chapter budget is too small')
    return prefix + b'x' * padding + suffix


def member(name, raw, stored=False):
    info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    info.create_system, info.external_attr = 3, 0o100644 << 16
    info._compresslevel = 6
    return info, raw


def create(path, profile=PRESSURE):
    if profile not in (PRESSURE, SMALL):
        raise ValueError('Only fixed generated profiles are supported')
    path = Path(path)
    if path.is_symlink() or path.exists():
        raise ValueError('Refusing to overwrite any input')
    path.parent.mkdir(parents=True, exist_ok=True)
    manifest = ['<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>']
    spine, navigation = [], []
    for i in range(profile.chapters):
        manifest.append(f'<item id="c{i}" href="text/c{i:04d}.xhtml" media-type="application/xhtml+xml"/>')
        spine.append(f'<itemref idref="c{i}"/>')
        navigation.append(f'<navPoint id="n{i}" playOrder="{i + 1}"><navLabel><text>{title(i)}</text>'
                          f'</navLabel><content src="text/c{i:04d}.xhtml"/></navPoint>')
    for i in range(profile.styles):
        manifest.append(f'<item id="s{i}" href="styles/s{i:04d}.css" media-type="text/css"/>')
    for i in range(profile.binary_blocks):
        manifest.append(f'<item id="b{i}" href="data/b{i:04d}.bin" media-type="application/octet-stream"/>')
    opf = ('<?xml version="1.0" encoding="UTF-8"?>'
           '<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uid">'
           '<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">'
           f'<dc:identifier id="uid">{FIXTURE_VERSION}</dc:identifier>'
           '<dc:title>纯生成 EPUB 压力回归</dc:title><dc:language>zh-CN</dc:language>'
           '<dc:creator>Reader generated tests</dc:creator></metadata><manifest>'
           + ''.join(manifest) + '</manifest><spine toc="ncx">' + ''.join(spine) + '</spine></package>')
    ncx = ('<?xml version="1.0" encoding="UTF-8"?>'
           '<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">'
           f'<head><meta name="dtb:uid" content="{FIXTURE_VERSION}"/></head>'
           '<docTitle><text>纯生成 EPUB 压力回归</text></docTitle><navMap>'
           + ''.join(navigation) + '</navMap></ncx>')
    container = ('<?xml version="1.0" encoding="UTF-8"?>'
                 '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">'
                 '<rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>'
                 '</rootfiles></container>')
    with path.open('xb') as output, zipfile.ZipFile(output, 'w') as archive:
        for name, raw, stored in [('mimetype', b'application/epub+zip', True),
                                  ('META-INF/container.xml', container.encode(), False),
                                  ('OEBPS/content.opf', opf.encode(), False),
                                  ('OEBPS/toc.ncx', ncx.encode(), False)]:
            info, raw = member(name, raw, stored)
            archive.writestr(info, raw)
        for i in range(profile.chapters):
            archive.writestr(*member(f'OEBPS/text/c{i:04d}.xhtml', chapter(i, profile.chapter_bytes)))
        for i in range(profile.styles):
            prefix = b'body{font-family:sans-serif;line-height:1.7}/*'
            raw = prefix + b'x' * (profile.style_bytes - len(prefix) - 2) + b'*/'
            archive.writestr(*member(f'OEBPS/styles/s{i:04d}.css', raw))
        for i in range(profile.binary_blocks):
            # Fixed SHAKE seed; no random source, personal data, downloaded file or image.
            raw = hashlib.shake_256((FIXTURE_VERSION + f'/block/{i}').encode()).digest(profile.binary_bytes)
            archive.writestr(*member(f'OEBPS/data/b{i:04d}.bin', raw, stored=True))
    return verify(path, profile)


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''):
            result.update(block)
    return result.hexdigest()


def verify(path, profile=PRESSURE):
    path = Path(path)
    if profile not in (PRESSURE, SMALL) or path.is_symlink() or not path.is_file():
        raise ValueError('Expected a generated regular fixture')
    compressed = path.stat().st_size
    if not 0 < compressed <= 64 * MIB:
        raise ValueError('Fixture does not fit the unchanged frontend download budget')
    with zipfile.ZipFile(path) as archive:
        infos = archive.infolist()
        expected_count = 4 + profile.chapters + profile.styles + profile.binary_blocks
        if len(infos) != expected_count or len({i.filename for i in infos}) != expected_count:
            raise ValueError('Fixture inventory mismatch')
        if infos[0].filename != 'mimetype' or infos[0].compress_type != zipfile.ZIP_STORED:
            raise ValueError('Invalid EPUB mimetype layout')
        if archive.read('mimetype') != b'application/epub+zip':
            raise ValueError('Invalid EPUB mimetype')
        expanded = sum(i.file_size for i in infos)
        if expected_count > 8192 or expanded > 256 * MIB:
            raise ValueError('Fixture exceeds unchanged archive limits')
        for info in infos:
            is_text = info.filename.endswith(('.xhtml', '.xml', '.opf', '.ncx', '.css'))
            limit = (4 if is_text else 32) * MIB
            if info.file_size > limit or info.compress_type not in (0, 8):
                raise ValueError('Fixture entry exceeds unchanged limits')
        if archive.testzip() is not None:
            raise ValueError('Fixture CRC integrity failed')
        package = ET.fromstring(archive.read('OEBPS/content.opf'))
        ns = {'opf': 'http://www.idpf.org/2007/opf'}
        items = package.findall('opf:manifest/opf:item', ns)
        refs = package.findall('opf:spine/opf:itemref', ns)
        if len(refs) != profile.chapters or len(items) != expected_count - 3:
            raise ValueError('Fixture manifest/spine mismatch')
        targets = {i.get('id'): i.get('href') for i in items}
        for i, ref in enumerate(refs):
            if targets.get(ref.get('idref')) != f'text/c{i:04d}.xhtml':
                raise ValueError('Fixture chapter mapping mismatch')
        for i in {0, profile.chapters - 1, min(805, profile.chapters - 1), min(806, profile.chapters - 1)}:
            raw = archive.read(f'OEBPS/text/c{i:04d}.xhtml')
            if len(raw) != profile.chapter_bytes:
                raise ValueError('Fixture chapter byte budget mismatch')
            doc = ET.fromstring(raw)
            heading = doc.find('{http://www.w3.org/1999/xhtml}body/{http://www.w3.org/1999/xhtml}h1')
            if heading is None or heading.text != title(i) or len(doc.findall('.//{http://www.w3.org/1999/xhtml}p')) != 64:
                raise ValueError('Fixture reading assertions lack generated text')
    if profile == PRESSURE and not (36 * MIB <= compressed <= 40 * MIB and 88 * MIB <= expanded <= 92 * MIB):
        raise ValueError('Pressure fixture is not at the intended large-book scale')
    return dict(version=FIXTURE_VERSION, profile='pressure' if profile == PRESSURE else 'small-unit-only',
                bytes=compressed, expandedBytes=expanded, entries=expected_count, chapters=profile.chapters,
                sha256=digest(path), generatedFromPublicConstantsOnly=True, originalOrPrivateInputRead=False)
