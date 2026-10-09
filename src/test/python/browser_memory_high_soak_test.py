"""Generated receipts only; no Docker, systemd, kernel mutation or real data."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "generated_high_soak_baseline", Path(__file__).with_name("bundled_browser_soak_report_test.py"))
BASELINE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASELINE)
VERIFIER = BASELINE.VERIFIER
SLICE = "readerhigh" + "a" * 16 + ".slice"
CID = "e" * 64


def generated_high_documents():
    documents = BASELINE.generated_documents()
    documents["RUNNING_JAR_IDENTITY.json"].update(containerId=CID, memoryPolicy="high-1536m")
    documents["MEMORY_HIGH_MEMBERSHIP.json"] = {
        "containerId": CID, "slice": SLICE, "hostPid": 1234,
        "membership": f"0::/{SLICE}/docker-{CID}.scope",
        "verifiedBeforeProbe": True, "policy": "high-1536m"}
    for name, peak, high, pids in (
            ("MEMORY_HIGH_PRESTART.json", 10 * 1024**2, 0, 1),
            ("MEMORY_HIGH_AFTER_SOAK.json", 1537 * 1024**2, 2893, 220),
            ("MEMORY_HIGH_FINAL.json", 1537 * 1024**2, 3000, 220)):
        resources = copy.deepcopy(documents["SOAK_REPORT.json"]["resources"])
        resources.update(cgroupRoot=f"/sys/fs/cgroup/{SLICE}", memoryHighBytes=1610612736,
                         memoryPeakBytes=peak, memoryCurrentBytes=peak, pidsPeak=pids, pidsCurrent=pids)
        resources["memoryEvents"].update(high=high, oom_group_kill=0)
        documents[name] = resources
    documents["MEMORY_HIGH_CLEANUP.json"] = {
        "slice": SLICE, "anchor": SLICE.removesuffix(".slice") + ".service",
        "populatedAfterContainerRemoval": 0, "ownedUnitsInactive": True, "policy": "high-1536m"}
    documents["CONTAINER_REMOVAL.json"] = {
        "containerId": CID, "removalSucceeded": True,
        "postRemovalInventoryObserved": True, "ownedContainersRemaining": 0}
    return documents


class BrowserMemoryHighSoakTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.documents = generated_high_documents()

    def verify(self, **overrides):
        for name, value in self.documents.items():
            text = ("".join(json.dumps(sample) + "\n" for sample in value)
                    if name.endswith(".jsonl") else json.dumps(value))
            (self.directory / name).write_text(text, encoding="utf-8")
        arguments = dict(architecture="amd64", revision=BASELINE.REVISION,
                         source_revision=BASELINE.SOURCE_REVISION, native_run=BASELINE.NATIVE_RUN,
                         expected_jar=BASELINE.JAR, expected_image=BASELINE.IMAGE, seconds=600,
                         require_container_state=True, require_removal=True, memory_policy="high-1536m")
        arguments.update(overrides)
        return VERIFIER.verify(self.directory, **arguments)

    def test_parent_policy_and_cleanup_are_independently_accepted_with_visible_pressure(self):
        policy = self.verify()["memoryPolicy"]
        self.assertEqual(3000, policy["parentHighEvents"])
        self.assertEqual(1537 * 1024**2, policy["parentPeakBytes"])
        self.assertIs(policy["parentCleanupVerified"], True)
        self.assertIs(policy["defaultImageOrProductionPolicyChanged"], False)

    def test_explicit_policy_cannot_be_silently_assumed_or_relabelled(self):
        for policy in ("unchanged", "", False, "high-1024m"):
            with self.subTest(policy=policy), self.assertRaises(ValueError):
                self.verify(memory_policy=policy)
        self.documents["RUNNING_JAR_IDENTITY.json"].pop("memoryPolicy")
        with self.assertRaises(ValueError):
            self.verify()

    def test_missing_parent_observations_fail_even_with_accepted_leaf_resources(self):
        for name in ("MEMORY_HIGH_PRESTART.json", "MEMORY_HIGH_MEMBERSHIP.json",
                     "MEMORY_HIGH_AFTER_SOAK.json", "MEMORY_HIGH_FINAL.json", "MEMORY_HIGH_CLEANUP.json"):
            def read(filename):
                if filename == name:
                    raise FileNotFoundError("Missing generated observation")
                return self.documents[filename]
            with self.subTest(name=name), self.assertRaises(FileNotFoundError):
                VERIFIER.verify_memory_policy("high-1536m", self.documents["RUNNING_JAR_IDENTITY.json"], read, True)

    def test_only_the_exact_running_container_parent_and_host_membership_are_accepted(self):
        for key, value in (("containerId", "f" * 64), ("slice", "system.slice"),
                           ("slice", "readerhigh../../other.slice"), ("hostPid", True), ("hostPid", 1),
                           ("verifiedBeforeProbe", False), ("membership", f"0::/other/{CID}")):
            self.documents = generated_high_documents()
            self.documents["MEMORY_HIGH_MEMBERSHIP.json"][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                self.verify()

    def test_nonzero_hard_events_or_wrong_budgets_fail_without_lowering_leaf_assertions(self):
        for family, key in (("memoryEvents", "max"), ("memoryEvents", "oom"),
                            ("memoryEvents", "oom_kill"), ("memoryEvents", "oom_group_kill"),
                            ("pidsEvents", "max")):
            self.documents = generated_high_documents()
            self.documents["MEMORY_HIGH_AFTER_SOAK.json"][family][key] = 1
            with self.subTest(family=family, key=key), self.assertRaises(ValueError):
                self.verify()
        for key, value in (("memoryMaxBytes", 3 * 1024**3), ("memoryHighBytes", "max"),
                           ("memoryHighBytes", True), ("pidsMax", 512), ("swapMaxBytes", 1),
                           ("swapCurrentBytes", 1), ("cpuQuota", "300000"), ("cpuPeriod", True),
                           ("cgroupRoot", "/sys/fs/cgroup/other.slice")):
            self.documents = generated_high_documents()
            self.documents["MEMORY_HIGH_AFTER_SOAK.json"][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                self.verify()

    def test_raw_cumulative_parent_counters_cannot_regress_or_disappear(self):
        for key, value in (("memoryPeakBytes", 1), ("pidsPeak", 1), ("memoryCurrentBytes", 3 * 1024**3)):
            self.documents = generated_high_documents()
            self.documents["MEMORY_HIGH_FINAL.json"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify()
        for high in (None, True, -1, 100):
            self.documents = generated_high_documents()
            self.documents["MEMORY_HIGH_FINAL.json"]["memoryEvents"]["high"] = high
            with self.subTest(high=high), self.assertRaises(ValueError):
                self.verify()

    def test_parent_cleanup_is_separate_from_container_removal(self):
        for key, value in (("slice", "other.slice"), ("anchor", "other.service"),
                           ("populatedAfterContainerRemoval", False), ("populatedAfterContainerRemoval", 1),
                           ("ownedUnitsInactive", False), ("policy", "unchanged")):
            self.documents = generated_high_documents()
            self.documents["MEMORY_HIGH_CLEANUP.json"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify()
        # The preliminary receipt cannot claim cleanup before it is observed.
        self.documents = generated_high_documents()
        self.assertIs(self.verify(require_removal=False)["memoryPolicy"]["parentCleanupVerified"], False)

    def test_unchanged_historical_reports_remain_unchanged_not_upgraded_to_the_new_policy(self):
        self.documents = BASELINE.generated_documents()
        policy = self.verify(require_removal=False, memory_policy="unchanged")["memoryPolicy"]
        self.assertEqual({"selection": "unchanged", "parentEarlyReclaimVerified": False}, policy)

    def test_harness_keeps_the_default_offline_budget_and_new_policy_is_explicit(self):
        root = Path(__file__).resolve().parents[3]
        harness = (root / "scripts/soak-native-image.sh").read_text(encoding="utf-8")
        helper = (root / "scripts/reader-browser-budget-slice.sh").read_text(encoding="utf-8")
        self.assertIn('READER_SOAK_MEMORY_POLICY:-unchanged', harness)
        self.assertIn('--network none', harness)
        self.assertIn('--memory=2g --memory-swap=2g --pids-limit=256 --cpus=2', harness)
        self.assertIn('MemoryHigh=1610612736 MemoryMax=2147483648 MemorySwapMax=0 TasksMax=256', helper)
        self.assertNotIn('memory.peak', helper)
        self.assertNotIn('drop_caches', helper)
        self.assertNotIn('self-hosted', (root / ".github/workflows/browser-image.yml").read_text(encoding="utf-8").replace('no QEMU or self-hosted runner', ''))


if __name__ == "__main__":
    unittest.main()
