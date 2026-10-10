"""Exact original/current JAR baseline only; no historical or bundled browser execution.

Both sides use generated accounts in separate new tmpfs stores. The host must
authorize this exact no-egress namespace before Java starts. Keep differential
red results red; successful readiness is not three-way/production acceptance.
"""
import argparse
import hashlib
import http.cookiejar
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

ORIGINAL_SHA = "b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c"
JAVA = "/opt/java/openjdk/bin/java"


def validate_inputs(unit, restored_sha):
    if not isinstance(unit, str) or not re.fullmatch(r"reader-baseline-api-[0-9]{8}-[a-z]\.service", unit):
        raise ValueError("Expected a dedicated baseline unit, never a production cgroup")
    if not isinstance(restored_sha, str) or not re.fullmatch(r"[0-9a-f]{64}", restored_sha) or restored_sha == ORIGINAL_SHA:
        raise ValueError("Expected an independently verified restored JAR identity")


def authentication_defaults(document, username):
    data = document.get("data")
    assert isinstance(data, dict) and data.get("username") == username
    token = data.get("accessToken")
    assert isinstance(token, str) and 0 < len(token) <= 2048
    assert all(type(data.get(key)) is int and data[key] > 0 for key in ("createdAt", "lastLoginAt"))
    # Do not normalize the received JSON or claim whole-response equality.
    # These exact three protocol fields are stochastic; every other default,
    # unknown field and null stays in the selected static-default comparison.
    return {key: value for key, value in data.items() if key not in ("accessToken", "createdAt", "lastLoginAt")}


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def identity(pid="self"):
    proc = Path("/proc") / str(pid)
    fields = dict(line.split(":", 1) for line in (proc / "status").read_text().splitlines() if ":" in line)
    return {"uid": int(fields["Uid"].split()[0]), "gid": int(fields["Gid"].split()[0]),
        "capabilitiesZero": all(int(fields[key].strip(), 16) == 0 for key in
            ("CapEff", "CapPrm", "CapBnd", "CapInh", "CapAmb")),
        "noNewPrivileges": fields["NoNewPrivs"].strip() == "1",
        "effectiveCpuSet": sorted(os.sched_getaffinity(0 if pid == "self" else int(pid))),
        "cgroup": (proc / "cgroup").read_text().strip().split("::", 1)[1],
        "netInode": os.stat(proc / "ns/net").st_ino}


def redact(value):
    if isinstance(value, dict):
        return {key: ({"redacted": True, "type": type(item).__name__}
                if isinstance(item, (str, dict, list)) and any(part in key.lower() for part in
                    ("token", "password", "salt", "cookie")) else redact(item))
                for key, item in value.items()}
    if isinstance(value, list):
        return [redact(item) for item in value]
    return value


class NonJsonResponse(Exception):
    def __init__(self, observation):
        self.observation = observation


def request(opener, base, path, body=None, json_expected=True):
    encoded = None if body is None else json.dumps(body).encode("utf-8")
    req = urllib.request.Request(base + path, data=encoded,
        headers={} if encoded is None else {"Content-Type": "application/json; charset=utf-8"})
    try:
        response = opener.open(req, timeout=3)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read(1048577)
        assert len(raw) <= 1048576
        document = None
        if json_expected:
            try:
                document = json.loads(raw)
            except (UnicodeDecodeError, ValueError):
                raise NonJsonResponse({"path": urllib.parse.urlsplit(path).path, "status": response.status, "bytes": len(raw),
                    "contentType": response.headers.get("Content-Type"), "sha256": hashlib.sha256(raw).hexdigest(),
                    "redirected": response.geturl() != base + path,
                    "looksHtml": b"<!doctype html" in raw.lower() or b"<html" in raw.lower(),
                    "empty": not bool(raw), "responseBodyPublished": False}) from None
        return {"status": response.status, "bytes": len(raw),
                "contentType": response.headers.get("Content-Type"), "sha256": hashlib.sha256(raw).hexdigest(),
                "document": document}


def successful(value):
    return value["status"] == 200 and isinstance(value["document"], dict) and value["document"].get("isSuccess") is True


def stop_owned(child):
    if child.poll() is None:
        child.terminate()
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=5)
    assert child.poll() is not None
    return child.returncode


def run_side(side, jar, port, username, password, initial, rows):
    work = Path("/storage") / side
    assert not work.exists()
    work.mkdir()
    cookies = http.cookiejar.CookieJar()
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPCookieProcessor(cookies))
    base = f"http://127.0.0.1:{port}"
    result = {"side": side, "jarSha256": sha(jar), "started": False, "ready": False,
              "generatedOnly": True, "generatedAuthenticationAccepted": False, "logoutIsolationAccepted": False}
    rows.append(result)
    log_path = Path("/tmp") / (side + "-generated-startup.log")
    child = None
    with log_path.open("xb") as log:
        try:
            child = subprocess.Popen([JAVA, "-jar", str(jar), "--reader.app.workDir=" + str(work),
                "--reader.server.port=" + str(port), "--reader.server.bindAddress=127.0.0.1",
                "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
                "--reader.app.webviewRenderer=remote", "--reader.app.remote-webview-api=http://127.0.0.1:8050",
                "--spring.profiles.active=prod"], cwd=work, stdin=subprocess.DEVNULL,
                stdout=log, stderr=subprocess.STDOUT)
            result["started"] = True
            child_identity = identity(child.pid)
            assert child_identity == initial
            result["javaWithinExactGuardedNamespaceAndBudget"] = True
            deadline = time.monotonic() + 75
            while time.monotonic() < deadline:
                if child.poll() is not None:
                    raise RuntimeError("Owned Java exited before readiness")
                try:
                    system = request(opener, base, "/reader3/getSystemInfo")
                    if successful(system):
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(0.25)
            else:
                raise TimeoutError("Owned Java readiness deadline")
            result["ready"] = True
            result["systemInfoEnvelope"] = {"status": system["status"],
                "returnDataKeys": sorted(system["document"]), "isSuccess": system["document"]["isSuccess"],
                "errorMsg": system["document"].get("errorMsg"), "dataType": type(system["document"].get("data")).__name__}
            home = request(opener, base, "/", json_expected=False)
            assert home["status"] == 200 and home["bytes"] > 0
            result["homepage"] = home
            result["phase"] = "registration"
            registration = request(opener, base, "/reader3/login", {"username": username, "password": password, "isLogin": False})
            result["registration"] = {**registration, "document": redact(registration["document"])}
            result["phase"] = "login"
            login = request(opener, base, "/reader3/login", {"username": username, "password": password, "isLogin": True})
            result["login"] = {**login, "document": redact(login["document"])}
            assert successful(registration) and successful(login)
            result["registrationStaticDefaults"] = authentication_defaults(registration["document"], username)
            result["loginStaticDefaults"] = authentication_defaults(login["document"], username)
            result["authenticationTransientFieldsNotComparedLiterally"] = ["accessToken", "createdAt", "lastLoginAt"]
            token = login["document"]["data"]["accessToken"]
            result["accessTokenShape"] = {"type": "str", "characters": len(token)}
            token_path = "/reader3/getBookshelf?" + urllib.parse.urlencode({"accessToken": token})
            token_opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
            token_shelf = request(token_opener, base, token_path)
            result["accessTokenBeforeLogout"] = {**token_shelf, "document": redact(token_shelf["document"])}
            assert successful(token_shelf) and token_shelf["document"].get("data") == []
            result["phase"] = "emptyShelf"
            shelf = request(opener, base, "/reader3/getBookshelf")
            result["emptyShelf"] = {**shelf, "document": redact(shelf["document"])}
            assert successful(shelf) and shelf["document"].get("data") == []
            result["generatedAuthenticationAccepted"] = True
            result["registration"] = {**registration, "document": redact(registration["document"])}
            result["login"] = {**login, "document": redact(login["document"])}
            result["emptyShelf"] = shelf
            result["sessionCookieNamesOnly"] = sorted({cookie.name for cookie in cookies})
            result["phase"] = "defaultGroups"
            groups = request(opener, base, "/reader3/getBookGroups")
            assert successful(groups)
            result["defaultGroups"] = {**groups, "document": redact(groups["document"])}
            result["phase"] = "logout"
            logout = request(opener, base, "/reader3/logout", {})
            result["logout"] = {**logout, "document": redact(logout["document"])}
            assert successful(logout)
            result["phase"] = "afterLogout"
            after_logout = request(opener, base, "/reader3/getBookshelf")
            result["afterLogout"] = {**after_logout, "document": redact(after_logout["document"])}
            assert isinstance(after_logout["document"], dict) and after_logout["document"].get("isSuccess") is False
            result["logoutIsolationAccepted"] = True
            token_after_logout = request(token_opener, base, token_path)
            result["accessTokenAfterLogout"] = {**token_after_logout, "document": redact(token_after_logout["document"])}
            result["logoutRevokesExistingAccessTokenObserved"] = not successful(token_after_logout)
            result["afterLogout"] = {**after_logout, "document": redact(after_logout["document"])}
            result["remainingSessionCookieNamesOnly"] = sorted({cookie.name for cookie in cookies})
            result["persistentGeneratedRelativeFilePaths"] = sorted(path.relative_to(work).as_posix() for path in work.rglob("*") if path.is_file())
            assert identity() == initial
        except Exception as error:
            result["failureType"] = type(error).__name__
            if isinstance(error, NonJsonResponse):
                result["nonJsonResponse"] = error.observation
            raise
        finally:
            if child is not None:
                result["ownedJavaExitCode"] = stop_owned(child)
                result["ownedJavaStopped"] = child.poll() is not None
            result["privateStartupLogBytes"] = log_path.stat().st_size
            result["rawStartupLogPublished"] = False
    return result


def main(argv=None):
    if sys.flags.optimize:
        raise SystemExit("Refuses disabled safety assertions")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--unit", required=True)
    parser.add_argument("--expected-restored-sha256", required=True)
    args = parser.parse_args(argv)
    validate_inputs(args.unit, args.expected_restored_sha256)
    report = {"scope": "exact b26/e59 sequential generated baseline API; not browser three-way",
              "passed": False, "threeWayExecuted": False, "strictCompatibilityAccepted": False,
              "realCredentialsImported": False, "privateBookBodyRead": False, "productionChanged": False,
              "browserExecutionRequested": False, "renderedUiVerified": False, "sides": []}
    started = time.monotonic()
    collector = None
    try:
        initial = identity()
        assert initial["uid"] == initial["gid"] == 10001 and os.getgroups() == []
        assert initial["capabilitiesZero"] and initial["noNewPrivileges"] and initial["effectiveCpuSet"] == [0, 1]
        assert sorted(name for _, name in socket.if_nameindex()) == ["lo"]
        parent = Path("/sys/fs/cgroup") / initial["cgroup"].lstrip("/")
        assert parent.parent.name == args.unit
        spec = importlib.util.spec_from_file_location("baseline_parent_resources", "/verification-scripts/report-browser-cgroup.py")
        collector = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(collector)
        collector.ROOT = parent.parent
        collector.verify_report(collector.collect_report(report["scope"]), require_no_swap=True,
                                expected_memory_high=1610612736)
        print(json.dumps({"runtimeIdentity": {**initial, "interfaces": ["lo"], "supplementaryGroups": []}}), flush=True)
        deadline = time.monotonic() + 25
        gate = Path("/verification-gate/host-verified.txt")
        while gate.read_text() != "HOST_CGROUP_AND_NAMESPACE_VERIFIED":
            if time.monotonic() >= deadline:
                raise TimeoutError("Independent host authorization absent")
            time.sleep(0.05)
        report["independentHostAuthorizationObservedBeforeJava"] = True
        assert sha(Path("/app/reader.jar")) == ORIGINAL_SHA
        assert sha(Path("/verification-inputs/current-ci-reader.jar")) == args.expected_restored_sha256
        assert os.environ.get("JAVA_TOOL_OPTIONS") == "-Xms256m -Xmx768m -Dfile.encoding=UTF-8"
        version = subprocess.run([JAVA, "-version"], capture_output=True, check=True, timeout=10)
        assert len(version.stderr) < 8192
        report["javaVersion"] = version.stderr.decode("utf-8")
        username = "generatedbaseline" + secrets.token_hex(5)
        password = "Probe-" + secrets.token_hex(12)
        original = run_side("original", Path("/app/reader.jar"), 18890, username, password, initial, report["sides"])
        restored = run_side("restored", Path("/verification-inputs/current-ci-reader.jar"), 18891, username, password, initial, report["sides"])
        report["bothReadinessReached"] = True
        report["bothGeneratedAuthLogoutContractsAccepted"] = True
        report["selectedComparisons"] = {"systemInfoEnvelope": original["systemInfoEnvelope"] == restored["systemInfoEnvelope"],
            "registrationStaticDefaults": original["registrationStaticDefaults"] == restored["registrationStaticDefaults"],
            "loginStaticDefaults": original["loginStaticDefaults"] == restored["loginStaticDefaults"],
            "accessTokenShape": original["accessTokenShape"] == restored["accessTokenShape"],
            "accessTokenBeforeLogoutLiteralJson": original["accessTokenBeforeLogout"]["document"] == restored["accessTokenBeforeLogout"]["document"],
            "accessTokenAfterLogoutLiteralJson": original["accessTokenAfterLogout"]["document"] == restored["accessTokenAfterLogout"]["document"],
            "emptyShelfLiteralJson": original["emptyShelf"]["document"] == restored["emptyShelf"]["document"],
            "defaultGroupsLiteralJson": original["defaultGroups"]["document"] == restored["defaultGroups"]["document"],
            "afterLogoutLiteralJson": original["afterLogout"]["document"] == restored["afterLogout"]["document"]}
        report["selectedExactComparisonsAccepted"] = all(report["selectedComparisons"].values())
        report["passed"] = report["selectedExactComparisonsAccepted"]
    except Exception as error:
        report["failureType"] = type(error).__name__
    finally:
        if collector is not None:
            try:
                report["aggregateResources"] = collector.collect_report(report["scope"])
                collector.verify_report(report["aggregateResources"], require_no_swap=True, expected_memory_high=1610612736)
                report["aggregateBudgetAccepted"] = True
            except Exception as error:
                report["aggregateBudgetAccepted"] = False
                report["resourceFailureType"] = type(error).__name__
                report["passed"] = False
        report["seconds"] = round(time.monotonic() - started, 3)
        assert len(json.dumps(report).encode("utf-8")) < 128 * 1024
        print(json.dumps({"innerReport": report}), flush=True)
    return 0 if report["passed"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
