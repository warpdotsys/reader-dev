import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import warnings
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location('reader_jar_reproducibility', ROOT / 'scripts/verify-reader-jar-reproducibility.py')
GATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATE)


class ReaderJarReproducibilityTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='reader-generated-jar-reproducibility-')
        self.addCleanup(self.temp.cleanup)
        self.first = Path(self.temp.name) / 'first.jar'
        self.second = Path(self.temp.name) / 'second.jar'

    def jar(self, path, *, date=(1980, 2, 1, 0, 0, 0), content=None, reverse=False,
            comment=b'', duplicate=False, missing=False):
        entries = sorted(GATE.REQUIRED_ENTRIES) + ['BOOT-INF/lib/generated.jar']
        if missing:
            entries.remove('BOOT-INF/classes/camoufox/worker.py')
        if reverse:
            entries.reverse()
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', UserWarning)
            with ZipFile(path, 'w') as jar:
                jar.comment = comment
                for name in entries + (entries[:1] if duplicate else []):
                    item = ZipInfo(name, date)
                    item.compress_type = ZIP_DEFLATED
                    item.external_attr = 0o100644 << 16
                    jar.writestr(item, (content or {}).get(name, b'generated-non-private-entry'))

    def test_exact_independent_inputs_can_pass(self):
        self.jar(self.first)
        self.jar(self.second)
        report = GATE.compare(self.first, self.second)
        self.assertTrue(report['passed'])
        self.assertTrue(report['archiveBytesMatch'])
        self.assertEqual(0, report['contentDifferences'])

    def test_timestamp_only_difference_still_fails_exact_gate(self):
        self.jar(self.first)
        self.jar(self.second, date=(2026, 10, 8, 0, 0, 2))
        report = GATE.compare(self.first, self.second)
        self.assertFalse(report['passed'])
        self.assertEqual(0, report['contentDifferences'])
        self.assertEqual(7, report['zipMetadataDifferences'])

    def test_class_content_change_is_not_normalized_or_hidden(self):
        self.jar(self.first)
        self.jar(self.second, content={'BOOT-INF/classes/com/htmake/reader/ReaderApplicationKt.class': b'different-generated-class'})
        report = GATE.compare(self.first, self.second)
        self.assertFalse(report['passed'])
        self.assertEqual(1, report['contentDifferences'])

    def test_order_only_difference_still_fails_exact_gate(self):
        self.jar(self.first)
        self.jar(self.second, reverse=True)
        report = GATE.compare(self.first, self.second)
        self.assertFalse(report['passed'])
        self.assertFalse(report['entryOrderMatches'])
        self.assertEqual(0, report['contentDifferences'])

    def test_archive_comment_only_difference_still_fails(self):
        self.jar(self.first)
        self.jar(self.second, comment=b'PRIVATE_GENERATED_COMMENT')
        report = GATE.compare(self.first, self.second)
        self.assertFalse(report['passed'])
        self.assertNotIn('PRIVATE', json.dumps(report))

    def test_same_file_is_not_a_second_build(self):
        self.jar(self.first)
        with self.assertRaisesRegex(ValueError, 'RepeatedArchiveInput'):
            GATE.compare(self.first, self.first)

    def test_hard_link_is_not_a_second_build(self):
        self.jar(self.first)
        os.link(self.first, self.second)
        with self.assertRaisesRegex(ValueError, 'RepeatedArchiveInput'):
            GATE.compare(self.first, self.second)

    def test_duplicate_or_missing_required_entries_fail(self):
        self.jar(self.first)
        for change in ({'duplicate': True}, {'missing': True}):
            self.jar(self.second, **change)
            with self.assertRaises(ValueError):
                GATE.compare(self.first, self.second)

    def test_comparison_is_read_only(self):
        self.jar(self.first)
        self.jar(self.second)
        before = (self.first.read_bytes(), self.second.read_bytes())
        GATE.compare(self.first, self.second)
        self.assertEqual(before, (self.first.read_bytes(), self.second.read_bytes()))

    def test_invalid_or_oversized_inputs_fail_without_path_or_content(self):
        self.jar(self.first)
        self.second.write_bytes(b'PRIVATE_NOT_A_JAR')
        with patch('sys.argv', ['gate', '--first', str(self.first), '--second', str(self.second)]), \
                patch('sys.stdout', new_callable=io.StringIO) as output:
            self.assertEqual(1, GATE.main())
        self.assertNotIn(str(self.second), output.getvalue())
        self.assertNotIn('PRIVATE', output.getvalue())
        self.jar(self.second)
        with patch.object(GATE, 'MAX_ARCHIVE_BYTES', 1), self.assertRaises(ValueError):
            GATE.compare(self.first, self.second)

    def test_entry_count_and_expansion_limits_are_enforced(self):
        self.jar(self.first)
        self.jar(self.second)
        for field in ('MAX_ENTRIES', 'MAX_ENTRY_BYTES', 'MAX_UNCOMPRESSED_BYTES'):
            with self.subTest(field=field), patch.object(GATE, field, 1), self.assertRaises(ValueError):
                GATE.compare(self.first, self.second)

    def test_only_exact_pinned_driver_gets_the_larger_finite_budget(self):
        observed_bytes = 203821698
        self.assertEqual('BOOT-INF/lib/driver-bundle-1.63.0.jar', GATE.PINNED_DRIVER_BUNDLE)
        header = SimpleNamespace(filename=GATE.PINNED_DRIVER_BUNDLE, file_size=observed_bytes, flag_bits=0)
        GATE.validate_expansion([header])
        for name in ('BOOT-INF/lib/driver-bundle-1.62.0.jar', 'BOOT-INF/classes/generated.bin'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                GATE.validate_expansion([SimpleNamespace(filename=name, file_size=observed_bytes, flag_bits=0)])

    def test_pinned_driver_limit_and_global_expansion_limit_still_fail_closed(self):
        header = SimpleNamespace(filename=GATE.PINNED_DRIVER_BUNDLE,
                                 file_size=GATE.MAX_DRIVER_BUNDLE_BYTES + 1, flag_bits=0)
        with self.assertRaises(ValueError):
            GATE.validate_expansion([header])
        header.file_size = 203821698
        with patch.object(GATE, 'MAX_UNCOMPRESSED_BYTES', 1), self.assertRaises(ValueError):
            GATE.validate_expansion([header])
        header.flag_bits = 1
        with self.assertRaises(ValueError):
            GATE.validate_expansion([header])

    def test_driver_budget_is_tied_to_the_real_pinned_dependency(self):
        build = (ROOT / 'build.gradle.kts').read_text(encoding='utf-8')
        self.assertIn('implementation("com.microsoft.playwright:playwright:1.63.0")', build)

    def test_cli_keeps_exact_difference_red(self):
        self.jar(self.first)
        self.jar(self.second, reverse=True)
        with patch('sys.argv', ['gate', '--first', str(self.first), '--second', str(self.second)]), \
                patch('sys.stdout', new_callable=io.StringIO) as output:
            self.assertEqual(1, GATE.main())
        self.assertFalse(json.loads(output.getvalue())['passed'])

    def test_ci_preserves_original_tests_before_second_clean_build(self):
        workflow = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
        saved_tests = workflow.index('Preserve actual Java Kotlin contracts including failures and skips')
        second_build = workflow.index('Verify a second clean build without a task output cache')
        self.assertLess(saved_tests, second_build)
        self.assertIn('clean bootJar --max-workers=2 --no-build-cache', workflow[second_build:])
        self.assertIn('--first "$RUNNER_TEMP/reader-jar-reproducibility/first.jar"', workflow[second_build:])
        self.assertIn('reader-jar-reproducibility-${{ github.sha }}', workflow[second_build:])
        self.assertIn('if: always()', workflow[second_build:])

    def test_real_boot_jar_configuration_opts_out_of_volatile_metadata(self):
        build = (ROOT / 'build.gradle.kts').read_text(encoding='utf-8')
        task = build[build.index('tasks.getByName<org.springframework.boot.gradle.tasks.bundling.BootJar>("bootJar")'):]
        task = task.split('\n}', 1)[0]
        self.assertIn('isPreserveFileTimestamps = false', task)
        self.assertIn('isReproducibleFileOrder = true', task)

    def test_equal_archives_containing_interpreter_cache_are_not_a_clean_release(self):
        for name in ('BOOT-INF/classes/camoufox/__pycache__/',
                     'BOOT-INF/classes/camoufox/__pycache__/worker.cpython-312.pyc',
                     'BOOT-INF/classes/generated.pyc', 'BOOT-INF/classes/generated.pyo'):
            with self.subTest(name=name):
                for path in (self.first, self.second):
                    self.jar(path)
                    with ZipFile(path, 'a') as jar:
                        jar.writestr(name, b'generated-interpreter-cache')
                with self.assertRaisesRegex(ValueError, 'PythonResourceCachePresent'):
                    GATE.compare(self.first, self.second)

    def test_process_resources_excludes_caches_instead_of_deleting_or_normalizing(self):
        build = (ROOT / 'build.gradle.kts').read_text(encoding='utf-8')
        task = build[build.index('tasks.named<ProcessResources>("processResources")'):]
        task = task.split('\n}', 1)[0]
        self.assertIn('exclude("**/__pycache__/**", "**/*.pyc", "**/*.pyo")', task)
        self.assertNotIn('delete(', task)

    def test_readable_worker_source_change_still_fails_complete_byte_comparison(self):
        self.jar(self.first)
        self.jar(self.second, content={'BOOT-INF/classes/camoufox/worker.py': b'different-generated-readable-source'})
        report = GATE.compare(self.first, self.second)
        self.assertFalse(report['passed'])
        self.assertEqual(1, report['contentDifferences'])

    def test_similarly_named_readable_resources_and_dependency_bytes_remain_in_scope(self):
        names = ('BOOT-INF/classes/generated.pyc.json', 'BOOT-INF/classes/generated.pyo.txt',
                 'BOOT-INF/classes/generated/__pycache__-note.txt', 'BOOT-INF/lib/generated-python-dependency.jar')
        for path in (self.first, self.second):
            self.jar(path)
            with ZipFile(path, 'a') as jar:
                for name in names:
                    jar.writestr(name, b'generated-required-resource')
        self.assertTrue(GATE.compare(self.first, self.second)['passed'])
        with ZipFile(self.second, 'a') as jar:
            jar.writestr('BOOT-INF/classes/generated-new-source.py', b'new-generated-source')
        report = GATE.compare(self.first, self.second)
        self.assertFalse(report['passed'])
        self.assertEqual(1, report['secondOnlyEntries'])


if __name__ == '__main__':
    unittest.main()
