"""Generated metadata/guards only; never downloads an image or starts Docker/Java."""
import copy
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/run-hosted-three-way.py"
SPEC = importlib.util.spec_from_file_location("hosted_three_way", SCRIPT)
PROBE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROBE)
SOURCE, REVISION, JAR = "a" * 40, "b" * 40, "c" * 64
IMAGE = "sha256:" + "d" * 64


def source_receipts():
    run = {"id": 7, "head_sha": SOURCE, "status": "completed", "conclusion": "success",
           "name": "Native release artifact rehearsal", "path": ".github/workflows/release-native.yml",
           "event": "pull_request", "head_repository": {"full_name": "warpdotsys/reader-dev"}}
    jobs = {"total_count": 6, "jobs": [{"id": index + 1, "name": name, "status": "completed", "conclusion": "success"}
        for index, name in enumerate(sorted(PROBE.EXPECTED_JOBS))]}
    comparison = {"ahead_by": 1, "behind_by": 0, "files": [], "base_commit": {"sha": SOURCE}, "commits": [{"sha": REVISION}]}
    return run, jobs, comparison


def actual_shape():
    provenance = {key: True for key in ("probeCompleted", "budgetAccepted", "overallAccepted", "originalJarUnchanged",
        "restoredJarUnchanged", "historicalRendererStoppedBeforeCamoufox")}
    provenance.update(originalJarSha256=PROBE.ORIGINAL_SHA, restoredJarSha256=JAR, runtimeImageId=IMAGE, archivedRenderer=PROBE.ARCHIVED,
        aggregateBudget={"cpuQuota": 200000, "cpuPeriod": 100000, "memoryMaxBytes": 2147483648,
                         "swapMaxBytes": 0, "pidsMax": 512, "memoryEvents": {"max": 0, "oom": 0, "oom_kill": 0}, "pidsEvents": {"max": 0}})
    comparison = {"originalExecuted": True, "comparisonMode": "same-run-original-remote-camoufox",
        "originalJarSha256": PROBE.ORIGINAL_SHA, "restoredJarSha256": JAR, "camoufoxJarSha256": JAR,
        "fullGeneratedReaderJsonRecorded": True, "metadataProbe": {"actualGetBookInfo": True}}
    for side in ("original", "restored", "camoufox"):
        comparison[side] = {"searches": [{} for _ in range(5)], "metadata": {"bookInfoApiCalls": 1}}
    return provenance, comparison


class HostedThreeWayTest(unittest.TestCase):
    def test_exact_source_and_all_actual_jobs_can_validate(self):
        PROBE.validate_source(*source_receipts(), 7, SOURCE, REVISION)

    def test_bad_source_run_identity_state_repo_workflow_and_privileged_event_rejected(self):
        for key, value in (("id", 8), ("head_sha", REVISION), ("status", "in_progress"), ("conclusion", "failure"),
                           ("name", "Another workflow"), ("path", ".github/workflows/release.yml"), ("event", "pull_request_target"),
                           ("head_repository", {"full_name": "generated-fork/reader-dev"})):
            run, jobs, comparison = source_receipts()
            run[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                PROBE.validate_source(run, jobs, comparison, 7, SOURCE, REVISION)

    def test_skipped_missing_or_duplicate_native_jobs_rejected(self):
        for mutate in (lambda jobs: jobs["jobs"].pop(), lambda jobs: jobs["jobs"][0].update(conclusion="skipped"),
                       lambda jobs: jobs["jobs"][0].update(status="in_progress"), lambda jobs: jobs["jobs"][0].update(id=jobs["jobs"][1]["id"]),
                       lambda jobs: jobs.update(total_count=5)):
            run, jobs, comparison = source_receipts()
            mutate(jobs)
            with self.assertRaises(ValueError):
                PROBE.validate_source(run, jobs, comparison, 7, SOURCE, REVISION)

    def test_changed_content_wrong_comparison_base_or_commit_rejected(self):
        for key, value in (("files", [{"filename": "generated-change"}]), ("behind_by", 1), ("ahead_by", 2),
                           ("base_commit", {"sha": REVISION}), ("commits", [{"sha": SOURCE}])):
            run, jobs, comparison = source_receipts()
            comparison[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                PROBE.validate_source(run, jobs, comparison, 7, SOURCE, REVISION)

    def test_same_source_snapshot_requires_empty_commit_comparison(self):
        run, jobs, comparison = source_receipts()
        comparison.update(ahead_by=0, commits=[])
        PROBE.validate_source(run, jobs, comparison, 7, SOURCE, SOURCE)

    def test_non_hosted_cli_refuses_before_any_command_or_source_inputs(self):
        environment = dict(os.environ)
        environment.pop("GITHUB_ACTIONS", None)
        result = subprocess.run([sys.executable, "-B", str(SCRIPT), "--native-run", "7", "--source", SOURCE,
            "--revision", REVISION, "--mode", "metadata"], capture_output=True, text=True, env=environment, timeout=10)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("GitHub-hosted runner", result.stderr)

    def test_real_hosted_environment_guard_does_not_accept_self_hosted_arm_or_wrong_repo(self):
        environment = {"GITHUB_ACTIONS": "true", "RUNNER_ENVIRONMENT": "github-hosted", "RUNNER_OS": "Linux",
            "RUNNER_ARCH": "X64", "GITHUB_REPOSITORY": "warpdotsys/reader-dev", "GITHUB_WORKSPACE": str(ROOT),
            "GITHUB_RUN_ID": "9", "GITHUB_RUN_ATTEMPT": "1"}
        with mock.patch.object(PROBE.sys, "platform", "linux"):
            PROBE.require_hosted(environment)
            for key, value in (("RUNNER_ENVIRONMENT", "self-hosted"), ("RUNNER_ARCH", "ARM64"),
                               ("GITHUB_REPOSITORY", "generated-fork/reader-dev"), ("GITHUB_RUN_ID", "../"), ("GITHUB_RUN_ATTEMPT", "0")):
                with self.subTest(key=key), self.assertRaises(ValueError):
                    PROBE.require_hosted({**environment, key: value})

    def test_generated_complete_shape_does_not_skip_the_mandatory_outcome_fields(self):
        PROBE.validate_metadata_outcome(*actual_shape(), JAR, IMAGE)
        for key in ("probeCompleted", "budgetAccepted", "overallAccepted", "originalJarUnchanged",
                    "restoredJarUnchanged", "historicalRendererStoppedBeforeCamoufox"):
            provenance, comparison = actual_shape()
            provenance[key] = False
            with self.subTest(key=key), self.assertRaises(ValueError):
                PROBE.validate_metadata_outcome(provenance, comparison, JAR, IMAGE)

    def test_actual_comparator_uppercase_digests_keep_identity_and_raw_evidence(self):
        provenance, comparison = actual_shape()
        for key in ("originalJarSha256", "restoredJarSha256", "camoufoxJarSha256"):
            comparison[key] = comparison[key].upper()
        before = copy.deepcopy(comparison)
        PROBE.validate_metadata_outcome(provenance, comparison, JAR, IMAGE)
        self.assertEqual(before, comparison, "Never rewrite a recorded comparison")
        for key in ("originalJarSha256", "restoredJarSha256", "camoufoxJarSha256"):
            for value in ("E" * 64, " " + before[key], before[key] + " ",
                          "sha256:" + before[key], before[key][:-1], None, 7):
                changed = copy.deepcopy(before)
                changed[key] = value
                with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                    PROBE.validate_metadata_outcome(provenance, changed, JAR, IMAGE)

    def test_wrong_archive_preserves_actual_hash_and_removes_only_unstarted_owned_container(self):
        cid = "a" * 64
        generated = b"Generated wrong baseline, not a user JAR"
        state = {"Id": cid, "Config": {"Labels": {"com.medwarp.reader.three-way-original": "9-1"}},
                 "State": {"Running": False, "StartedAt": "0001-01-01T00:00:00Z"}}
        calls = []
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory)

            def fake_command(*args, **kwargs):
                calls.append(args)
                if args[:3] == ("docker", "image", "inspect"):
                    output = json.dumps([{"Id": IMAGE, "Architecture": "amd64", "RepoDigests": []}])
                elif args[:2] == ("docker", "create"):
                    output = cid
                elif args[:2] == ("docker", "cp"):
                    Path(args[3]).write_bytes(generated)
                    output = ""
                elif args[:2] == ("docker", "inspect"):
                    output = json.dumps([state])
                else:
                    output = ""
                return subprocess.CompletedProcess(args, 0, output, "")

            with mock.patch.dict(os.environ, {"GITHUB_RUN_ID": "9", "GITHUB_RUN_ATTEMPT": "1"}), \
                 mock.patch.object(PROBE, "command", side_effect=fake_command), self.assertRaises(ValueError):
                PROBE.extract_original(report)
            receipt = json.loads((report / "ORIGINAL_ARCHIVE_IDENTITY.json").read_text())
            self.assertEqual(hashlib.sha256(generated).hexdigest(), receipt["actualJarSha256"])
            self.assertEqual(len(generated), receipt["jarBytes"])
            self.assertIs(False, receipt["originalIdentityAccepted"])
            self.assertIs(True, receipt["extractionContainerNeverStarted"])
            self.assertIs(True, receipt["extractionContainerRemovalObserved"])
            self.assertIn(("docker", "rm", cid), calls)
            self.assertFalse(any(args[:2] in (("docker", "load"), ("docker", "run"), ("docker", "start")) for args in calls))

    def test_archive_extraction_never_overwrites_existing_file_before_docker(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory)
            (report / "original.jar").write_bytes(b"Generated protected fixture")
            with mock.patch.object(PROBE, "command") as command, self.assertRaises(ValueError):
                PROBE.extract_original(report)
            command.assert_not_called()

    def test_missing_original_preflight_refuses_before_native_commands(self):
        with tempfile.TemporaryDirectory() as directory, mock.patch.object(PROBE, "command") as command:
            with self.assertRaises(ValueError):
                PROBE.require_original_preflight(Path(directory))
            command.assert_not_called()

    def test_source_hash_substitutions_and_budget_pressure_not_accepted(self):
        for key, value in (("originalJarSha256", JAR), ("restoredJarSha256", PROBE.ORIGINAL_SHA), ("runtimeImageId", "mutable-tag"),
                           ("archivedRenderer", "mutable-reference")):
            provenance, comparison = actual_shape()
            provenance[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                PROBE.validate_metadata_outcome(provenance, comparison, JAR, IMAGE)
        for key, value in (("cpuQuota", 400000), ("memoryMaxBytes", 4294967296), ("swapMaxBytes", 1073741824),
                           ("pidsMax", 1024), ("memoryEvents", {"max": 1, "oom": 0, "oom_kill": 0}), ("pidsEvents", {"max": 1})):
            provenance, comparison = actual_shape()
            provenance["aggregateBudget"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                PROBE.validate_metadata_outcome(provenance, comparison, JAR, IMAGE)

    def test_missing_actual_side_or_get_book_info_cannot_borrow_other_sides(self):
        for side in ("original", "restored", "camoufox"):
            for replacement in (None, {"searches": [], "metadata": {"bookInfoApiCalls": 1}},
                                {"searches": [{} for _ in range(5)], "metadata": {"bookInfoApiCalls": 0}}):
                provenance, comparison = actual_shape()
                comparison[side] = replacement
                with self.subTest(side=side), self.assertRaises(ValueError):
                    PROBE.validate_metadata_outcome(provenance, comparison, JAR, IMAGE)

    def test_post_removal_observation_is_actual_query_and_never_timeout_as_zero(self):
        provenance = {"slice": "readerwebview000000000000.slice", "removedOwnedTestContainers": ["a" * 64, "b" * 64]}
        with mock.patch.object(PROBE.Path, "exists", return_value=False), \
             mock.patch.object(PROBE, "command", return_value=subprocess.CompletedProcess([], 0, "", "")) as query:
            self.assertEqual(0, PROBE.observe_removal(provenance)["ownedContainersRemaining"])
            query.assert_called_once()
        with mock.patch.object(PROBE, "command", side_effect=subprocess.TimeoutExpired("generated-query", 1)), self.assertRaises(subprocess.TimeoutExpired):
            PROBE.observe_removal(provenance)
        with mock.patch.object(PROBE.Path, "exists", return_value=False), \
             mock.patch.object(PROBE, "command", return_value=subprocess.CompletedProcess([], 0, "a" * 64, "")), self.assertRaises(RuntimeError):
            PROBE.observe_removal(provenance)

    def test_ownership_scope_rejected_before_query(self):
        for provenance in ({"slice": "../../unowned"}, {"slice": "readerwebview000000000000.slice", "removedOwnedTestContainers": ["a" * 64]}
                           , {"slice": "readerwebview000000000000.slice", "removedOwnedTestContainers": ["a" * 64, "a" * 64]}):
            with mock.patch.object(PROBE, "command") as query, self.assertRaises(ValueError):
                PROBE.observe_removal(provenance)
            query.assert_not_called()

    def test_workflow_consumes_native_artifacts_without_rebuild_and_never_uploads_jars(self):
        workflow = (ROOT / ".github/workflows/browser-image.yml").read_text(encoding="utf-8")
        job = workflow.split("  generated-three-way:\n", 1)[1].split("  existing-artifact-regression:\n", 1)[0]
        self.assertIn("runs-on: ubuntu-24.04", job)
        self.assertIn("--validate-source-only", job)
        self.assertLess(job.index("--validate-source-only"), job.index("actions/download-artifact@"))
        self.assertLess(job.index("--validate-original-only"), job.index("actions/download-artifact@"))
        self.assertIn("digest-mismatch: error", job)
        self.assertIn("reader-three-way-report/*.json", job)
        self.assertNotIn("gradlew", job)
        self.assertNotIn("docker build", job)
        self.assertNotIn("secrets.", job)
        self.assertNotIn("self-hosted", job)
        self.assertIn("utf8-characterization", workflow)


if __name__ == "__main__":
    unittest.main()
