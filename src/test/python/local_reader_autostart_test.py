"""Generated Windows-only checks for the candidate logon-task helper.

Every scheduler operation goes through a generated adapter stored below the
generated candidate root. The tests never enumerate, register, export, or
unregister a real Windows scheduled task.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import uuid
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "local-reader" / "ReaderCandidateAutostart.ps1"
PS = shutil.which("powershell.exe")
WINDOWS = os.name == "nt" and PS is not None and Path("D:/").is_dir()
NS = {"t": "http://schemas.microsoft.com/windows/2004/02/mit/task"}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def adapter_source() -> str:
    # Chinese ObjectNotFound proves the product code does not parse English
    # scheduler messages to distinguish absence from an operation failure.
    return r'''$state = Join-Path $PSScriptRoot 'generated-fake-task.xml'
$missing = {
  param($TaskName, $TaskPath)
  if (-not (Test-Path -LiteralPath $state -PathType Leaf)) {
    $exception = New-Object Management.Automation.ItemNotFoundException '生成测试任务不存在'
    throw (New-Object Management.Automation.ErrorRecord $exception, 'GeneratedTaskMissing', ([Management.Automation.ErrorCategory]::ObjectNotFound), $state)
  }
  [pscustomobject]@{ TaskName = $TaskName; TaskPath = $TaskPath }
}
@{
  Get = $missing.GetNewClosure()
  Export = ({ param($TaskName, $TaskPath) Get-Content -LiteralPath $state -Raw -Encoding Unicode }).GetNewClosure()
  Register = ({ param($TaskName, $TaskPath, $Xml)
    if (Test-Path -LiteralPath $state -PathType Leaf) { throw (New-Object InvalidOperationException '生成测试任务已存在') }
    Set-Content -LiteralPath $state -Value $Xml -Encoding Unicode -NoNewline
  }).GetNewClosure()
  Unregister = ({ param($TaskName, $TaskPath)
    if (-not (Test-Path -LiteralPath $state -PathType Leaf)) { throw (New-Object Management.Automation.ItemNotFoundException '生成测试任务不存在') }
    Remove-Item -LiteralPath $state -Force
  }).GetNewClosure()
}
'''


@unittest.skipUnless(WINDOWS, "requires isolated Windows D: fixture and Windows PowerShell")
class CandidateAutostartTests(unittest.TestCase):
    def setUp(self):
        self.candidate_id = "stage2-autostart-" + uuid.uuid4().hex[:12]
        self.root = Path("D:/Reader-Pro-Candidate-" + self.candidate_id)
        self.assertFalse(self.root.exists(), "generated candidate root unexpectedly exists")
        self.root.mkdir(parents=True)
        source = self.root / "runtime-software/source"
        source.mkdir(parents=True)
        self.launcher = source / "ReaderCandidate.ps1"
        self.launcher.write_text("param([string]$Action, [string]$CandidateId)\nexit 0\n", encoding="utf-8")
        Path(str(self.launcher) + ".sha256").write_text(sha256(self.launcher), encoding="ascii")
        self.adapter = self.root / "candidate-test-scheduler-adapter.ps1"
        self.adapter.write_text(adapter_source(), encoding="utf-8")
        self.profile = self.root / "candidate-profile.json"
        self.profile.write_text(json.dumps({
            "schemaVersion": 1, "candidateId": self.candidate_id,
            "paths": {"workDir": "/mnt/d/Reader-Pro-Candidate-" + self.candidate_id,
                      "mountRoot": "/mnt/reader-candidate-" + self.candidate_id},
            "artifacts": {"softwareManifestSha256": "a" * 64},
        }), encoding="utf-8")

    def tearDown(self):
        if self.root.exists() or self.root.is_symlink():
            shutil.rmtree(self.root)

    def invoke(self, action="Plan", *extra, expect=0):
        env = os.environ.copy()
        env["READER_CANDIDATE_AUTOSTART_TEST_MODE"] = "1"
        command = [PS, "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "RemoteSigned",
                   "-File", str(SCRIPT), "-Action", action, "-ProfilePath", str(self.profile),
                   "-TestSchedulerAdapterPath", str(self.adapter), *extra]
        result = subprocess.run(command, text=True, capture_output=True, env=env, timeout=30)
        self.assertEqual(expect, result.returncode, result.stdout + result.stderr)
        return result

    def test_plan_generates_current_user_limited_task_without_scheduler_mutation(self):
        plan = json.loads(self.invoke().stdout)
        self.assertEqual("Plan", plan["action"])
        self.assertFalse(plan["executeRequested"])
        self.assertFalse(plan["wouldRegister"])
        self.assertFalse((self.root / "generated-fake-task.xml").exists())
        xml = ET.fromstring(plan["taskXml"])
        self.assertEqual("ReaderProCurrentUser", xml.find("t:Principals/t:Principal", NS).attrib["id"])
        self.assertEqual("InteractiveToken", xml.findtext("t:Principals/t:Principal/t:LogonType", namespaces=NS))
        self.assertEqual("LeastPrivilege", xml.findtext("t:Principals/t:Principal/t:RunLevel", namespaces=NS))
        self.assertEqual("true", xml.findtext("t:Settings/t:Hidden", namespaces=NS))
        self.assertEqual(1, len(xml.findall("t:Triggers/t:LogonTrigger", NS)))
        self.assertEqual([], xml.findall("t:Triggers/t:BootTrigger", NS))

    def test_install_and_uninstall_use_fake_adapter_and_explicit_gates(self):
        blocked = self.invoke("Install", expect=1)
        self.assertIn("-Execute", blocked.stderr)
        self.assertFalse((self.root / "generated-fake-task.xml").exists())
        installed = self.invoke("Install", "-Execute", "-ConfirmCandidateId", self.candidate_id)
        self.assertTrue(json.loads(installed.stdout)["applied"])
        self.assertTrue((self.root / "generated-fake-task.xml").is_file())
        inspected = self.invoke("Inspect")
        self.assertTrue(json.loads(inspected.stdout)["exactManagedMatch"])
        removed = self.invoke("Uninstall", "-Execute", "-ConfirmCandidateId", self.candidate_id)
        self.assertTrue(json.loads(removed.stdout)["applied"])
        self.assertFalse((self.root / "generated-fake-task.xml").exists())

    def test_rejects_extra_action_without_deleting_fake_task(self):
        plan = json.loads(self.invoke().stdout)
        task_xml = plan["taskXml"].replace(
            "</Actions>",
            "<Exec><Command>C:\\Windows\\System32\\cmd.exe</Command><Arguments>/c whoami</Arguments></Exec></Actions>",
        )
        state = self.root / "generated-fake-task.xml"
        state.write_text(task_xml, encoding="utf-16")
        rejected = self.invoke("Uninstall", "-Execute", "-ConfirmCandidateId", self.candidate_id, expect=1)
        self.assertIn("拒绝删除", rejected.stderr)
        self.assertTrue(state.exists(), "identity mismatch must leave the foreign task unchanged")

    def test_reparse_point_ancestor_is_rejected(self):
        shutil.rmtree(self.root)
        target = Path(tempfile.mkdtemp(prefix="reader-autostart-target-", dir="D:/"))
        try:
            subprocess.run(["cmd.exe", "/d", "/c", "mklink", "/J", str(self.root), str(target)],
                           capture_output=True, text=True, check=True, timeout=15)
            (target / "candidate-profile.json").write_text("{}", encoding="utf-8")
            result = self.invoke(expect=1)
            self.assertIn("重解析点", result.stderr)
        finally:
            if self.root.exists() or self.root.is_symlink():
                os.rmdir(self.root)
            shutil.rmtree(target, ignore_errors=True)


class SourceContractTests(unittest.TestCase):
    def test_profile_grammar_and_real_scheduler_guards_are_present(self):
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertIn("'^[a-z][a-z0-9-]{2,62}$'", source)
        self.assertIn("Split([char]'\\'", source)
        self.assertIn("Get-AbsoluteWindowsPath", source)
        self.assertNotIn("IsPathFullyQualified", source)
        self.assertIn("Get-Sha256Hex", source)
        self.assertIn("$script:UsingTestSchedulerAdapter", source)
        self.assertIn("-not $Execute", source)
        self.assertNotIn("-Xml $xml -Force", source)


if __name__ == "__main__":
    unittest.main()
