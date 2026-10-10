"""Report validation uses generated observations, not a claimed real soak."""

import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts/verify-bundled-browser-soak.py"
SPEC = importlib.util.spec_from_file_location("soak_report_verifier", SCRIPT)
VERIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFIER)
REVISION = "a" * 40
SOURCE_REVISION = "b" * 40
JAR = "c" * 64
IMAGE = "sha256:" + "d" * 64
NATIVE_RUN = 123456


def generated_documents():
    quiet = {"memoryCurrentBytes": 128 * 1024 ** 2, "memoryPeakBytes": 800 * 1024 ** 2,
             "pidsCurrent": 40, "pidsPeak": 200,
             "browserProcesses": {"worker": 0, "browser": 0, "driver": 0}}
    trace = [dict(copy.deepcopy(quiet), round=index, method="GET" if index % 2 else "POST",
                  elapsedSeconds=index * 30.0, maxRequestSeconds=29.0) for index in range(1, 21)]
    report = {"passed": True, "expectedImageRevision": REVISION, "requestedSeconds": 600,
              "accounts": 4, "rounds": 20, "successfulSearches": 88, "continuousSeconds": 600.0,
              "totalSeconds": 660.0, "coldRequestSeconds": 3.0,
              "releaseIdentity": {"version": "4.0.7", "buildRevision": REVISION},
              "targetCookieOrShapeErrors": 0, "targetMethods": {"GET": 51, "POST": 40},
              "coldQuiescent": copy.deepcopy(quiet), "warmQuiescent": copy.deepcopy(quiet),
              "finalQuiescent": copy.deepcopy(quiet), "samples": copy.deepcopy(trace[-5:]),
              "faults": [{"kind": kind, "failedReturnDataVerified": True,
                          "healthyRecoveryVerified": True, "ownedWorkerKilled": kind == "workerExit",
                          "seconds": 3.0, "afterRecovery": copy.deepcopy(quiet)}
                         for kind in ("navigationTimeout", "scriptWatchdog", "workerExit")],
              "resources": {"architecture": "x86_64", "memoryMaxBytes": 2 * 1024 ** 3,
                            "memoryPeakBytes": quiet["memoryPeakBytes"],
                            "memoryCurrentBytes": quiet["memoryCurrentBytes"],
                            "memoryEvents": {"max": 0, "oom": 0, "oom_kill": 0},
                            "pidsMax": 256, "pidsCurrent": 40, "pidsPeak": 200,
                            "pidsEvents": {"max": 0}, "swapMaxBytes": 0, "swapCurrentBytes": 0,
                            "cpuQuota": "200000", "cpuPeriod": 100000}}
    identity = {"architecture": "amd64", "revision": REVISION, "jarSha256": JAR, "imageId": IMAGE}
    return {"SOURCE_RUN.json": {"id": NATIVE_RUN, "name": "Native release artifact rehearsal",
                               "path": ".github/workflows/release-native.yml", "head_sha": SOURCE_REVISION,
                               "conclusion": "success",
                               "html_url": f"https://github.com/warpdotsys/reader-dev/actions/runs/{NATIVE_RUN}"},
            "PRELOAD_IDENTITY.json": dict(identity, phase="check"),
            "LOADED_IDENTITY.json": dict(identity, phase="loaded"),
            "RUNNING_JAR_IDENTITY.json": {"revision": REVISION, "jarSha256": JAR, "network": "none"},
            "SOAK_REPORT.json": report, "SOAK_TRACE.jsonl": trace,
            "CONTAINER_STATE.json": {"Status": "running", "Running": True, "OOMKilled": False,
                                     "Dead": False, "Restarting": False, "Paused": False,
                                     "ExitCode": 0, "Error": ""}}


class SoakReportVerifierTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.documents = generated_documents()

    def save(self):
        for name, value in self.documents.items():
            text = ("".join(json.dumps(sample) + "\n" for sample in value)
                    if name.endswith(".jsonl") else json.dumps(value))
            (self.directory / name).write_text(text, encoding="utf-8")

    def verify(self, **overrides):
        arguments = dict(architecture="amd64", revision=REVISION, source_revision=SOURCE_REVISION,
                         native_run=NATIVE_RUN, expected_jar=JAR, expected_image=IMAGE, seconds=600,
                         require_container_state=True)
        arguments.update(overrides)
        self.save()
        return VERIFIER.verify(self.directory, **arguments)

    def test_complete_generated_trace_is_accepted_without_capacity_or_cleanup_claim(self):
        result = self.verify()
        self.assertIs(result["accepted"], True)
        self.assertEqual(20, result["rounds"])
        self.assertEqual(88, result["successfulSearches"])
        self.assertEqual(7, len(result["fileSha256"]))
        self.assertIs(result["productionCapacityOrLeakAbsenceProven"], False)
        self.assertIs(result["independentPostRemovalInventoryObserved"], False)

    def test_arm64_architecture_and_kernel_identity_are_both_checked(self):
        for name in ("PRELOAD_IDENTITY.json", "LOADED_IDENTITY.json"):
            self.documents[name]["architecture"] = "arm64"
        self.documents["SOAK_REPORT.json"]["resources"]["architecture"] = "aarch64"
        self.assertEqual("arm64", self.verify(architecture="arm64")["architecture"])
        self.documents["SOAK_REPORT.json"]["resources"]["architecture"] = "x86_64"
        with self.assertRaises(ValueError):
            self.verify(architecture="arm64")

    def test_foreign_source_run_jar_image_and_revision_are_rejected(self):
        cases = [("SOURCE_RUN.json", "id", 789), ("SOURCE_RUN.json", "head_sha", REVISION),
                 ("SOURCE_RUN.json", "conclusion", "failure"),
                 ("SOURCE_RUN.json", "path", ".github/workflows/other.yml"),
                 ("SOURCE_RUN.json", "html_url", "https://other.invalid/123456"),
                 ("PRELOAD_IDENTITY.json", "jarSha256", "e" * 64),
                 ("LOADED_IDENTITY.json", "imageId", "sha256:" + "e" * 64),
                 ("LOADED_IDENTITY.json", "revision", SOURCE_REVISION),
                 ("RUNNING_JAR_IDENTITY.json", "network", "bridge")]
        for filename, key, value in cases:
            with self.subTest(filename=filename, key=key):
                self.documents = generated_documents()
                self.documents[filename][key] = value
                with self.assertRaises(ValueError):
                    self.verify()

    def test_success_summary_cannot_hide_missing_or_duplicate_rounds(self):
        for mutate in (lambda trace: trace.pop(5), lambda trace: trace.insert(5, trace[5]),
                       lambda trace: trace.reverse()):
            with self.subTest(mutate=mutate):
                self.documents = generated_documents()
                mutate(self.documents["SOAK_TRACE.jsonl"])
                with self.assertRaises(ValueError):
                    self.verify()

    def test_round_order_method_and_elapsed_time_cannot_be_fabricated_by_summary(self):
        for key, value in (("round", 999), ("method", "POST"), ("elapsedSeconds", 0),
                           ("maxRequestSeconds", -1), ("maxRequestSeconds", float("inf"))):
            with self.subTest(key=key):
                self.documents = generated_documents()
                self.documents["SOAK_TRACE.jsonl"][0][key] = value
                with self.assertRaises(ValueError):
                    self.verify()

    def test_tail_samples_and_final_duration_must_match_real_trace(self):
        for key, value in (("samples", []), ("continuousSeconds", 601.0),
                           ("requestedSeconds", 1800), ("passed", False), ("rounds", True)):
            with self.subTest(key=key):
                self.documents = generated_documents()
                self.documents["SOAK_REPORT.json"][key] = value
                with self.assertRaises(ValueError):
                    self.verify()

    def test_clock_reads_keep_raw_values_and_idle_time_cannot_complete_a_short_trace(self):
        self.documents["SOAK_REPORT.json"]["continuousSeconds"] = 600.001
        result = self.verify()
        self.assertEqual(600.001, result["continuousSeconds"])
        self.assertEqual(600.0, result["lastRoundElapsedSeconds"])
        for end, summary in ((599.999, 600.0), (600.0, 600.02), (600.0, 599.999)):
            with self.subTest(end=end, summary=summary):
                self.documents = generated_documents()
                self.documents["SOAK_TRACE.jsonl"][-1]["elapsedSeconds"] = end
                self.documents["SOAK_REPORT.json"]["samples"] = copy.deepcopy(self.documents["SOAK_TRACE.jsonl"][-5:])
                self.documents["SOAK_REPORT.json"]["continuousSeconds"] = summary
                with self.assertRaises(ValueError):
                    self.verify()

    def test_request_counts_cookies_and_generated_account_count_are_checked(self):
        for key, value in (("targetMethods", {"GET": 51, "POST": 39}),
                           ("successfulSearches", 89), ("targetCookieOrShapeErrors", 1),
                           ("targetCookieOrShapeErrors", False), ("accounts", 3)):
            with self.subTest(key=key):
                self.documents = generated_documents()
                self.documents["SOAK_REPORT.json"][key] = value
                with self.assertRaises(ValueError):
                    self.verify()

    def test_each_fault_failure_and_recovery_is_mandatory(self):
        for key, value in (("kind", "scriptWatchdog"), ("failedReturnDataVerified", False),
                           ("healthyRecoveryVerified", False), ("ownedWorkerKilled", True),
                           ("seconds", float("nan"))):
            with self.subTest(key=key):
                self.documents = generated_documents()
                self.documents["SOAK_REPORT.json"]["faults"][0][key] = value
                with self.assertRaises(ValueError):
                    self.verify()

    def test_every_quiescent_sample_checks_children_and_cumulative_peak(self):
        for key, value in (("browserProcesses", {"worker": 0, "browser": 1, "driver": 0}),
                           ("browserProcesses", {"worker": False, "browser": 0, "driver": 0}),
                           ("memoryPeakBytes", 700 * 1024 ** 2), ("pidsPeak", 199)):
            with self.subTest(key=key):
                self.documents = generated_documents()
                self.documents["SOAK_TRACE.jsonl"][0][key] = value
                with self.assertRaises(ValueError):
                    self.verify()

    def test_resources_keep_the_original_no_swap_cpu_memory_and_pid_budget(self):
        for key, value in (("memoryMaxBytes", 3 * 1024 ** 3), ("swapMaxBytes", 1024 ** 3),
                           ("swapCurrentBytes", 1), ("pidsMax", 257), ("cpuQuota", "300000"),
                           ("cpuPeriod", True), ("memoryMaxBytes", float(2 * 1024 ** 3))):
            with self.subTest(key=key):
                self.documents = generated_documents()
                self.documents["SOAK_REPORT.json"]["resources"][key] = value
                with self.assertRaises(ValueError):
                    self.verify()

    def test_missing_or_nonzero_mandatory_events_never_default_to_zero(self):
        for family, key in (("memoryEvents", "max"), ("memoryEvents", "oom"),
                            ("memoryEvents", "oom_kill"), ("pidsEvents", "max")):
            for value in (None, 1, False):
                with self.subTest(family=family, key=key, value=value):
                    self.documents = generated_documents()
                    events = self.documents["SOAK_REPORT.json"]["resources"][family]
                    if value is None:
                        del events[key]
                    else:
                        events[key] = value
                    with self.assertRaises(ValueError):
                        self.verify()

    def test_container_state_is_optional_before_cleanup_but_not_after_download(self):
        del self.documents["CONTAINER_STATE.json"]
        self.assertIs(self.verify(require_container_state=False)["preRemovalContainerStateObserved"], False)
        with self.assertRaises(ValueError):
            self.verify()

    def add_removal(self):
        self.documents["RUNNING_JAR_IDENTITY.json"]["containerId"] = "e" * 64
        self.documents["CONTAINER_REMOVAL.json"] = {
            "containerId": "e" * 64, "removalSucceeded": True,
            "postRemovalInventoryObserved": True, "ownedContainersRemaining": 0}

    def test_post_removal_inventory_has_its_own_identity_and_cannot_be_assumed(self):
        with self.assertRaises(ValueError):
            self.verify(require_removal=True)
        self.add_removal()
        self.assertIs(self.verify(require_removal=True)["independentPostRemovalInventoryObserved"], True)

    def test_removal_requires_a_successful_host_inventory_and_exact_owned_container(self):
        for key, value in (("containerId", "f" * 64), ("removalSucceeded", False),
                           ("postRemovalInventoryObserved", False), ("ownedContainersRemaining", None),
                           ("ownedContainersRemaining", 1), ("ownedContainersRemaining", False)):
            with self.subTest(key=key):
                self.documents = generated_documents()
                self.add_removal()
                self.documents["CONTAINER_REMOVAL.json"][key] = value
                with self.assertRaises(ValueError):
                    self.verify(require_removal=True)

    def test_duplicate_keys_nonfinite_numbers_and_nonobjects_are_rejected(self):
        for raw in (b'{"round":1,"round":2}', b'{"x":NaN}', b'{"x":Infinity}', b'[]'):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                VERIFIER.json_object(raw)

    def test_large_file_and_missing_regular_file_are_rejected(self):
        self.save()
        path = self.directory / "SOAK_TRACE.jsonl"
        path.write_bytes(b"x" * (VERIFIER.MAX_TRACE_BYTES + 1))
        with self.assertRaises(ValueError):
            VERIFIER.read_observation(self.directory, path.name, VERIFIER.MAX_TRACE_BYTES, {})
        with self.assertRaises(ValueError):
            VERIFIER.read_observation(self.directory, "MISSING.json", 100, {})

    def test_memory_growth_is_reported_without_claiming_it_is_a_leak_or_hiding_it(self):
        self.documents["SOAK_REPORT.json"]["finalQuiescent"]["memoryCurrentBytes"] += 1024 ** 2
        self.assertEqual(1024 ** 2, self.verify()["quiescentMemoryChangeBytes"])

    def test_cli_failure_never_prints_input_payload_or_private_path(self):
        self.save()
        (self.directory / "SOAK_REPORT.json").write_text('private-sensitive-marker-not-json', encoding="utf-8")
        result = subprocess.run([sys.executable, str(SCRIPT), str(self.directory),
                                 "--architecture", "amd64", "--expected-revision", REVISION,
                                 "--expected-source-revision", SOURCE_REVISION,
                                 "--expected-native-run", str(NATIVE_RUN), "--expected-jar", JAR,
                                 "--expected-image", IMAGE, "--seconds", "600"],
                                capture_output=True, text=True, timeout=15)
        self.assertEqual(1, result.returncode)
        self.assertEqual({"accepted": False, "reason": "OfflineSoakEvidenceRejected"}, json.loads(result.stdout))
        self.assertEqual("", result.stderr)


class SoakReportIntegrationTest(unittest.TestCase):
    def test_runtime_harness_checks_full_trace_not_only_summary(self):
        harness = (ROOT / "scripts/soak-native-image.sh").read_text(encoding="utf-8")
        self.assertIn("scripts/verify-bundled-browser-soak.py", harness)
        self.assertIn("--expected-image", harness)
        self.assertIn("--expected-jar", harness)
        self.assertIn("--expected-source-revision", harness)
        self.assertIn("--seconds", harness)
        self.assertIn("docker ps -aq --no-trunc --filter", harness)
        self.assertIn("--require-container-state --require-removal", harness)


if __name__ == "__main__":
    unittest.main()
