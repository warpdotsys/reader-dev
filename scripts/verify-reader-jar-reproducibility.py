"""Compare two separately built Boot JARs, including every entry and ZIP bytes.

Only counters and digests are emitted. Never extract files, normalize either
archive, remove differing entries, or accept a semantic-only match as success.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys
from zipfile import BadZipFile, ZipFile

MAX_ARCHIVE_BYTES = 512 * 1024 ** 2
MAX_UNCOMPRESSED_BYTES = 1024 ** 3
MAX_ENTRY_BYTES = 128 * 1024 ** 2
MAX_ENTRIES = 40000
CHUNK = 1024 * 1024
REQUIRED_ENTRIES = frozenset((
    'META-INF/MANIFEST.MF',
    'org/springframework/boot/loader/JarLauncher.class',
    'BOOT-INF/classes/com/htmake/reader/ReaderApplicationKt.class',
    'BOOT-INF/classes/camoufox/worker.py',
    'BOOT-INF/classes/web-vue3/index.html',
    'BOOT-INF/classes/web/index.html',
))


def digest(stream):
    value = hashlib.sha256()
    for chunk in iter(lambda: stream.read(CHUNK), b''):
        value.update(chunk)
    return value.hexdigest()


def inspect(path):
    path = Path(path)
    size = path.stat().st_size
    if not 0 < size <= MAX_ARCHIVE_BYTES:
        raise ValueError('ArchiveSizeOutOfBounds')
    with path.open('rb') as stream:
        archive_sha = digest(stream)
    with ZipFile(path) as jar:
        items = jar.infolist()
        if not 0 < len(items) <= MAX_ENTRIES:
            raise ValueError('ArchiveEntryCountOutOfBounds')
        names = [item.filename for item in items]
        if len(names) != len(set(names)):
            raise ValueError('DuplicateArchiveEntry')
        if (not REQUIRED_ENTRIES.issubset(names) or
                not any(name.startswith('BOOT-INF/lib/') and name.endswith('.jar') for name in names)):
            raise ValueError('RequiredBootJarEntryMissing')
        if (sum(item.file_size for item in items) > MAX_UNCOMPRESSED_BYTES or
                any(not 0 <= item.file_size <= MAX_ENTRY_BYTES or item.flag_bits & 1 for item in items)):
            raise ValueError('ArchiveExpansionOutOfBounds')
        entries = {}
        for item in items:
            with jar.open(item) as stream:
                entry_sha = digest(stream)
            entries[item.filename] = {
                'sha256': entry_sha, 'bytes': item.file_size,
                'metadata': (item.date_time, item.compress_type, item.flag_bits,
                             item.create_system, item.create_version, item.extract_version,
                             item.external_attr, item.internal_attr, item.extra, item.comment),
            }
        return dict(archiveSha256=archive_sha, archiveBytes=size, names=names,
                    entries=entries, archiveComment=jar.comment)


def compare(first_path, second_path):
    first_path, second_path = Path(first_path), Path(second_path)
    # Identical paths, symlinks and hard links are not independent build inputs.
    if first_path.samefile(second_path):
        raise ValueError('RepeatedArchiveInput')
    first, second = inspect(first_path), inspect(second_path)
    first_names, second_names = set(first['names']), set(second['names'])
    common = first_names & second_names
    content_differences = sum(
        (first['entries'][name]['sha256'], first['entries'][name]['bytes']) !=
        (second['entries'][name]['sha256'], second['entries'][name]['bytes']) for name in common)
    metadata_differences = sum(first['entries'][name]['metadata'] != second['entries'][name]['metadata'] for name in common)
    exact = first['archiveSha256'] == second['archiveSha256'] and first['archiveBytes'] == second['archiveBytes']
    report = dict(schemaVersion=1, scope='two separate Boot JAR files; complete bytes and all entries, without normalization',
                  firstJarSha256=first['archiveSha256'], secondJarSha256=second['archiveSha256'],
                  firstJarBytes=first['archiveBytes'], secondJarBytes=second['archiveBytes'],
                  firstEntryCount=len(first_names), secondEntryCount=len(second_names),
                  firstOnlyEntries=len(first_names - second_names), secondOnlyEntries=len(second_names - first_names),
                  contentDifferences=content_differences, zipMetadataDifferences=metadata_differences,
                  entryOrderMatches=first['names'] == second['names'],
                  archiveCommentMatches=first['archiveComment'] == second['archiveComment'],
                  archiveBytesMatch=exact,
                  passed=exact and first_names == second_names and content_differences == metadata_differences == 0)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--first', required=True)
    parser.add_argument('--second', required=True)
    args = parser.parse_args()
    try:
        report = compare(args.first, args.second)
    except (OSError, ValueError, BadZipFile, RuntimeError):
        # Paths and malformed archive text are not diagnostic payloads.
        print(json.dumps(dict(schemaVersion=1, passed=False, failureCategory='InvalidReproducibilityInput')))
        return 1
    print(json.dumps(report, sort_keys=True, indent=2))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
