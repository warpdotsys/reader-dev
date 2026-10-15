"""Pure unit tests for the read-only candidate identity profile.

No test creates a candidate, starts a service, formats a disk, or reads a
release archive.  Temporary artifacts are synthetic bytes only.
"""
import copy
from dataclasses import replace
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = ROOT / "scripts/local-reader/candidate-profile.py"
SPEC = importlib.util.spec_from_file_location("local_reader_candidate_profile", MODULE_PATH)
candidate = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
sys.modules[SPEC.name] = candidate
SPEC.loader.exec_module(candidate)


def digest(label: str) -> str:
    return hashlib.sha256(label.encode("ascii")).hexdigest()


def fixture() -> dict:
    identity = "stage2-611-rc1"
    work = "/mnt/d/Reader-Pro-Candidate-" + identity
    return {
        "schemaVersion": 1, "candidateId": identity,
        "release": {"version": "6.1.1-rc.1", "revision": "a" * 40},
        "artifacts": {"imageId": "sha256:" + digest("image"), "archiveSha256": digest("archive"),
                      "readerJarSha256": digest("reader"), "licenseJarSha256": digest("license"),
                      "softwareManifestSha256": digest("software")},
        "paths": {"workDir": work, "mountRoot": "/mnt/reader-candidate-" + identity,
                  "imageDisk": work + "/image.ext4", "dataDisk": work + "/data.ext4"},
        "runtime": {"unit": "reader-candidate-" + identity + ".service", "name": "reader-candidate-" + identity,
                    "readerPort": 19063, "licensePort": 19064, "cpuQuotaPercent": 200,
                    "memoryMaxBytes": 2 * 1024 ** 3, "memoryHighBytes": int(1.5 * 1024 ** 3),
                    "memorySwapMaxBytes": 0, "tasksMax": 256},
        "disks": {"imageBytes": 16 * 1024 ** 3, "dataBytes": 4 * 1024 ** 3},
        "logging": {"currentMaxBytes": 2 * 1024 ** 2, "previousFiles": 1,
                    "outerMaxBytes": 2 * 1024 ** 2, "outerRetainedFiles": 2},
    }


class CandidateProfileTest(unittest.TestCase):
    def test_accepts_immutable_candidate_identity(self):
        profile = candidate.validate_profile(fixture())
        self.assertEqual(profile.reader_port, 19063)
        self.assertEqual(profile.work_dir, "/mnt/d/Reader-Pro-Candidate-stage2-611-rc1")
        with self.assertRaisesRegex(Exception, "cannot assign"):
            profile.name = "changed"  # type: ignore[misc]
        self.assertEqual(candidate.profile_to_dict(profile), fixture())

    def test_rejects_duplicate_unknown_and_wrong_types(self):
        raw = json.dumps(fixture())[:-1] + ',"candidateId":"other"}'
        with self.assertRaises(candidate.DuplicateKeyError):
            candidate.parse_profile_json(raw)
        bad = fixture(); bad["unexpected"] = True
        with self.assertRaises(candidate.ProfileError):
            candidate.validate_profile(bad)
        bad = fixture(); bad["runtime"]["readerPort"] = True
        with self.assertRaises(candidate.ProfileError):
            candidate.validate_profile(bad)
        bad = fixture(); bad["schemaVersion"] = True
        with self.assertRaises(candidate.ProfileError):
            candidate.validate_profile(bad)

    def test_rejects_legacy_scope_paths_names_ports_and_version(self):
        for target, value in (("workDir", candidate.OLD_WORK_DIR), ("mountRoot", candidate.OLD_MOUNT_ROOT),
                              ("unit", candidate.OLD_UNIT), ("name", candidate.OLD_NAME)):
            bad = fixture()
            bad["paths" if target in ("workDir", "mountRoot") else "runtime"][target] = value
            with self.subTest(target=target), self.assertRaises(candidate.ProfileError):
                candidate.validate_profile(bad)
        for port in (18961, 18963, 18964):
            bad = fixture(); bad["runtime"]["readerPort"] = port
            with self.subTest(port=port), self.assertRaises(candidate.ProfileError):
                candidate.validate_profile(bad)
        bad = fixture(); bad["release"]["version"] = "6.1.0"
        with self.assertRaises(candidate.ProfileError):
            candidate.validate_profile(bad)

    def test_rejects_tampered_identity_path_and_budget(self):
        for edit in (
                lambda x: x["paths"].__setitem__("workDir", "/mnt/d/Reader-Pro-Candidate-stage2-611-rc1/../other"),
                lambda x: x["paths"].__setitem__("dataDisk", "/mnt/d/other/data.ext4"),
                lambda x: x["runtime"].__setitem__("cpuQuotaPercent", 201),
                lambda x: x["runtime"].__setitem__("memoryMaxBytes", 2 * 1024 ** 3 + 1)):
            bad = fixture(); edit(bad)
            with self.assertRaises(candidate.ProfileError):
                candidate.validate_profile(bad)

    def test_load_rejects_symlink_and_path_verifier_never_creates(self):
        profile = candidate.validate_profile(fixture())
        with tempfile.TemporaryDirectory() as raw:
            base = Path(raw)
            source = base / "source.json"; source.write_text(json.dumps(fixture()), encoding="utf-8")
            link = base / "profile.json"
            try:
                link.symlink_to(source)
            except (NotImplementedError, OSError):
                self.skipTest("symlink unavailable")
            with self.assertRaises(candidate.ProfileError):
                candidate.load_profile(link)
            owned = base / "owned"; owned.mkdir()
            (owned / "redirect").symlink_to(source)
            unsafe = replace(profile, work_dir=str(owned / "redirect"))
            with self.assertRaises(candidate.ProfileError):
                candidate.verify_profile_paths(unsafe)
        # Fixed WSL paths do not exist in this Windows test process; validation is read-only.
        candidate.verify_profile_paths(profile)

    def test_artifact_verifier_requires_measured_hashes_and_external_image_identity(self):
        profile = candidate.validate_profile(fixture())
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            paths = {}
            for name in ("archive", "reader", "license", "software"):
                item = root / name; item.write_bytes(name.encode("ascii")); paths[name] = item
            tailored = copy.deepcopy(fixture())
            for source, field in (("archive", "archiveSha256"), ("reader", "readerJarSha256"),
                                  ("license", "licenseJarSha256"), ("software", "softwareManifestSha256")):
                tailored["artifacts"][field] = digest(source)
            profile = candidate.validate_profile(tailored)
            candidate.verify_artifact_files(profile, archive=paths["archive"], reader_jar=paths["reader"],
                                            license_jar=paths["license"], software_manifest=paths["software"],
                                            observed_image_id=profile.image_id)
            paths["reader"].write_bytes(b"tampered")
            with self.assertRaises(candidate.ProfileError):
                candidate.verify_artifact_files(profile, archive=paths["archive"], reader_jar=paths["reader"],
                                                license_jar=paths["license"], software_manifest=paths["software"],
                                                observed_image_id=profile.image_id)
            paths["reader"].write_bytes(b"reader")
            with self.assertRaises(candidate.ProfileError):
                candidate.verify_artifact_files(profile, archive=paths["archive"], reader_jar=paths["reader"],
                                                license_jar=paths["license"], software_manifest=paths["software"],
                                                observed_image_id="sha256:" + digest("other"))


if __name__ == "__main__":
    unittest.main()
