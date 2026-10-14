"""Small generated unit data; these are not complete-image pressure observations."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET
import zipfile

ROOT = Path(__file__).parents[3]
SPEC = importlib.util.spec_from_file_location('pressure_fixture', ROOT / 'scripts/epub-pressure/fixture.py')
MODULE = importlib.util.module_from_spec(SPEC)
import sys
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class GeneratedPressureFixtureTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='reader-synthetic-fixture-unit-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.path = self.root / 'small.epub'

    def create(self):
        return MODULE.create(self.path, MODULE.SMALL)

    def test_small_profile_is_labelled_unit_only_not_pressure_acceptance(self):
        value = self.create()
        self.assertEqual('small-unit-only', value['profile'])
        self.assertEqual(10, value['entries'])
        self.assertEqual(3, value['chapters'])
        self.assertTrue(value['generatedFromPublicConstantsOnly'])
        self.assertFalse(value['originalOrPrivateInputRead'])
        self.assertEqual(value, MODULE.verify(self.path, MODULE.SMALL))

    def test_same_public_seed_and_metadata_produce_identical_bytes_in_this_runtime(self):
        first = self.create()
        second = MODULE.create(self.root / 'second.epub', MODULE.SMALL)
        self.assertEqual(first, second)
        self.assertEqual(self.path.read_bytes(), (self.root / 'second.epub').read_bytes())

    def test_refuses_to_overwrite_even_generated_input(self):
        value = self.create()
        with self.assertRaises(ValueError):
            self.create()
        self.assertEqual(value['sha256'], MODULE.digest(self.path))

    def test_unsupported_custom_size_or_user_input_profile_rejected_before_creation(self):
        for profile in (None, 'a local book', MODULE.Profile(1, 1, 1, 1, 1, 1)):
            with self.subTest(profile=profile), self.assertRaises(ValueError):
                MODULE.create(self.path, profile)
        self.assertFalse(self.path.exists())

    def test_small_fixture_cannot_be_accepted_as_large_pressure_data(self):
        self.create()
        with self.assertRaises(ValueError):
            MODULE.verify(self.path, MODULE.PRESSURE)

    def test_synthetic_chapters_are_utf8_valid_with_real_unique_titles_and_paragraphs(self):
        raw = MODULE.chapter(805, 60 * 1024)
        self.assertEqual(60 * 1024, len(raw))
        doc = ET.fromstring(raw)
        ns = {'h': 'http://www.w3.org/1999/xhtml'}
        self.assertEqual(MODULE.title(805), doc.find('h:body/h:h1', ns).text)
        self.assertEqual(64, len(doc.findall('.//h:p', ns)))
        self.assertNotEqual(raw, MODULE.chapter(806, 60 * 1024))
        self.assertNotIn('\ufffd', raw.decode())
        self.assertGreater(len(''.join(doc.itertext())), 1000)

    def test_chapter_does_not_silently_reduce_required_body_to_fit_budget(self):
        with self.assertRaises(ValueError):
            MODULE.chapter(805, 10)

    def test_manifest_ncx_and_spine_map_to_existing_valid_chapters(self):
        self.create()
        with zipfile.ZipFile(self.path) as archive:
            ns = {'o': 'http://www.idpf.org/2007/opf'}
            opf = ET.fromstring(archive.read('OEBPS/content.opf'))
            ids = {item.get('id'): item.get('href') for item in opf.findall('o:manifest/o:item', ns)}
            for ref in opf.findall('o:spine/o:itemref', ns):
                self.assertIn('OEBPS/' + ids[ref.get('idref')], archive.namelist())
            ncx = ET.fromstring(archive.read('OEBPS/toc.ncx'))
            self.assertEqual(3, len(ncx.findall('.//{http://www.daisy.org/z3986/2005/ncx/}navPoint')))
            self.assertEqual('mimetype', archive.infolist()[0].filename)
            self.assertEqual(zipfile.ZIP_STORED, archive.infolist()[0].compress_type)
            self.assertEqual(b'application/epub+zip', archive.read('mimetype'))

    def test_pressure_shape_is_fixed_below_current_frontend_limits(self):
        profile = MODULE.PRESSURE
        self.assertEqual(850, profile.chapters)
        self.assertEqual(1690, 4 + profile.chapters + profile.styles + profile.binary_blocks)
        self.assertEqual(36 * MODULE.MIB, profile.binary_blocks * profile.binary_bytes)
        self.assertLess(profile.chapter_bytes, 4 * MODULE.MIB)
        self.assertLess(profile.binary_bytes, 32 * MODULE.MIB)
        total = profile.chapters * profile.chapter_bytes + profile.styles * profile.style_bytes + profile.binary_blocks * profile.binary_bytes
        self.assertGreater(total, 88 * MODULE.MIB)
        self.assertLess(total, 92 * MODULE.MIB)


if __name__ == '__main__':
    unittest.main()
