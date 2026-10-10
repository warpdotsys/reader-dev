"""Generated report doubles only; not an executed browser/HTTPS acceptance."""
import hashlib
import importlib.util
import io
from pathlib import Path
from unittest.mock import Mock
import unittest

ROOT = Path(__file__).parents[3]
spec = importlib.util.spec_from_file_location("tls_report_guard", ROOT / "scripts/verify-camoufox-tls.py")
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
JAR, WORKER, REVISION = "a" * 64, "b" * 64, "c" * 40


def generated_report():
    report = {"schemaVersion": 1, "jarSha256": JAR, "workerSha256": WORKER,
        "architecture": "amd64", "revision": REVISION, "generatedOnly": True,
        "fixtureOnlyPrivateDistributionPolicy": True, "httpsTested": True,
        "realCredentialsImported": False, "privateBookBodyRead": False, "readerJarStarted": False,
        "hostTrustStoreChanged": False, "ignoreHttpsErrorsUsed": False,
        "workerLaunchOverridden": False, "fullGoalComplete": False,
        "identity": {"uid": 10001, "gid": 10001, "groups": [], "interfaces": ["lo"],
            "capsZero": True, "noNewPrivileges": True},
        "resources": {"memoryMaxBytes": 2147483648, "memoryPeakBytes": 1000000,
            "pidsMax": 256, "cpuQuota": "200000", "cpuPeriod": 100000,
            "memoryEvents": {"max": 0, "oom": 0, "oom_kill": 0, "oom_group_kill": 0},
            "pidsEvents": {"max": 0}, "swapCurrentBytes": 0, "swapMaxBytes": 0}, "results": []}
    for case in guard.CASES[:10]:
        actors = ["primary", "primary" if case.startswith("same-") else "secondary"]
        if case == "resources":
            actors = ["primary", "primary", "secondary"]
        if case == "resources-redirect":
            actors = ["primary"] * 4 + ["secondary"]
        rows, tunnels = [], []
        for index, actor in enumerate(actors):
            primary = actor == "primary"
            host, port = ("127.0.0.1", 44301) if primary else ("127.0.0.2", 44302)
            headers = [["Host", host + ":" + str(port)], ["User-Agent", guard.USER_AGENT]]
            if primary:
                headers += [["Authorization", guard.AUTH], ["X-Generated-Trace", guard.TRACE]]
            method, body = "GET", b""
            if case.endswith(("307", "308")):
                method, body = "POST", guard.JSON_BODY
                headers += [["Content-Type", "application/json; charset=UTF-8"]]
                if index == 1 and case.startswith("same-"):
                    headers += [["Cookie", guard.COOKIE]]
            elif case == "cross-post" and index == 0:
                method, body = "POST", b"q=generated-header-only"
                headers += [["Content-Type", "application/x-www-form-urlencoded; charset=UTF-8"]]
            rows.append({"case": case, "actor": actor, "headers": headers, "method": method,
                "bodyByteCount": len(body), "bodySha256": hashlib.sha256(body).hexdigest(),
                "targetSourcePort": 40000 + index})
            tunnels.append({"method": "CONNECT", "host": host, "port": port,
                "targetSourcePort": 40000 + index, "clientToServerBytes": 10, "serverToClientBytes": 20})
        cookies = []
        if case.endswith(("307", "308")):
            cookies = [{"name": "generated_session", "value": "GENERATED_SESSION_ONLY",
                "path": "/probe/", "domain": "127.0.0.1", "sameSite": "Lax", "httpOnly": True,
                "hostOnly": True, "secure": True, "deleted": False}]
        report["results"].append({"case": case, "failureType": None, "rejections": 0,
            "everyActualTargetConnectionUsedProxy": True, "targetRequests": rows, "tunnels": tunnels,
            "observedTargetSourcePorts": [r["targetSourcePort"] for r in rows],
            "workerResult": {"body": "认证头差分书-" + case, "cookies": cookies}})
    for case, code in zip(guard.CASES[10:], ("SSL_ERROR_BAD_CERT_DOMAIN", "SEC_ERROR_UNKNOWN_ISSUER")):
        report["results"].append({"case": case, "failureType": "Error", "tlsErrorCodes": [code],
            "workerResult": None, "negativeHttpRequests": 0, "targetRequests": [],
            "tunnels": [{"method": "CONNECT", "host": "127.0.0.1", "port": 44303,
                "clientToServerBytes": 10, "serverToClientBytes": 20}]})
    return report


class CamoufoxTlsReportTest(unittest.TestCase):
    def accept(self, report):
        return guard.validate(report, JAR, WORKER, REVISION, "amd64")

    def reject(self, mutation):
        report = generated_report()
        mutation(report)
        with self.assertRaises((ValueError, SystemExit)):
            self.accept(report)

    def testCompleteGeneratedDouble(self):
        value = self.accept(generated_report())
        self.assertEqual(24, value["targetRequests"])
        self.assertFalse(value["readerApiOrProductionAccepted"])

    def testRejectsWrongJar(self):
        self.reject(lambda r: r.update(jarSha256="d" * 64))

    def testRejectsWrongPackagedWorker(self):
        self.reject(lambda r: r.update(workerSha256="d" * 64))

    def testRejectsDifferentSourceRevision(self):
        self.reject(lambda r: r.update(revision="d" * 40))

    def testRejectsHeaderLeakage(self):
        self.reject(lambda r: r["results"][1]["targetRequests"][1]["headers"].append(["Authorization", guard.AUTH]))

    def testRejectsStaleHttpsHost(self):
        self.reject(lambda r: r["results"][2]["targetRequests"][1]["headers"][0].__setitem__(1, "127.0.0.1:44301"))

    def testRejectsChangedConnectDestination(self):
        self.reject(lambda r: r["results"][0]["tunnels"][0].update(port=44309))

    def testRejectsUncorrelatedTargetPort(self):
        self.reject(lambda r: r["results"][0]["targetRequests"][0].update(targetSourcePort=40099))

    def testRejectsChangedPostBytes(self):
        self.reject(lambda r: r["results"][4]["targetRequests"][1].update(bodySha256=hashlib.sha256(b"wrong").hexdigest()))

    def testRejectsWrongPostMediaType(self):
        self.reject(lambda r: r["results"][4]["targetRequests"][1]["headers"].append(["Content-Type", "text/plain"]))

    def testRejectsCookieCrossHost(self):
        self.reject(lambda r: r["results"][6]["targetRequests"][1]["headers"].append(["Cookie", guard.COOKIE]))

    def testRejectsCookieMetadataLoss(self):
        self.reject(lambda r: r["results"][4]["workerResult"]["cookies"][0].update(secure=False))

    def testRejectsFailedPageScript(self):
        self.reject(lambda r: r["results"][8]["workerResult"].update(body="not executed"))

    def testRejectsUnprovenNegativeCertificateCause(self):
        self.reject(lambda r: r["results"][11].update(tlsErrorCodes=[]))

    def testRejectsHttpAfterInvalidCertificate(self):
        self.reject(lambda r: r["results"][10].update(negativeHttpRequests=1))

    def testRejectsTlsBypass(self):
        self.reject(lambda r: r.update(ignoreHttpsErrorsUsed=True))

    def testRejectsHostTrustMutation(self):
        self.reject(lambda r: r.update(hostTrustStoreChanged=True))

    def testRejectsOutsideNetwork(self):
        self.reject(lambda r: r["identity"].update(interfaces=["lo", "eth0"]))

    def testRejectsRoot(self):
        self.reject(lambda r: r["identity"].update(uid=0))

    def testRejectsCapability(self):
        self.reject(lambda r: r["identity"].update(capsZero=False))

    def testRejectsNonzeroSwapBudget(self):
        self.reject(lambda r: r["resources"].update(swapMaxBytes=1073741824))

    def testRejectsOom(self):
        self.reject(lambda r: r["resources"]["memoryEvents"].update(oom_kill=1))

    def testRejectsSkippedCase(self):
        self.reject(lambda r: r["results"].pop())

    def testRejectsExcessHeaderFields(self):
        self.reject(lambda r: r["results"][0]["targetRequests"][0]["headers"].extend([["X-Fake", "x"]] * 65))

    def testRejectsExcessHeaderBytes(self):
        self.reject(lambda r: r["results"][0]["targetRequests"][0]["headers"].append(["X-Fake", "x" * 8193]))

    def testJarHashStreamsAndWorksWithoutPython311Api(self):
        spec = importlib.util.spec_from_file_location("tls_fixture_without_browser_import",
            ROOT / "scripts/smoke-camoufox-tls.py")
        fixture = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(fixture)
        payload = b"generated-only-stream" * 100000
        stream = Mock(wraps=io.BytesIO(payload))
        stream.__enter__ = Mock(return_value=stream)
        stream.__exit__ = Mock(return_value=False)
        path = Mock()
        path.open.return_value = stream
        self.assertEqual(hashlib.sha256(payload).hexdigest(), fixture.hash_file(path))
        self.assertTrue(stream.read.call_count >= 2)
        self.assertTrue(all(call.args == (1048576,) for call in stream.read.call_args_list))


if __name__ == "__main__":
    unittest.main()
