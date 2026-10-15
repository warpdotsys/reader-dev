"""Fail-closed acceptance of bounded, generated packaged-worker HTTPS evidence."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re

CASES = ("same-get", "cross-get", "cross-post", "resources", "same-post-307",
         "same-post-308", "cross-post-307", "cross-post-308", "script-cross-get",
         "resources-redirect", "wrong-host", "untrusted")
AUTH = "Bearer GENERATED_NOT_A_REAL_CREDENTIAL"
TRACE = "GENERATED_HEADER_TRACE_ONLY"
USER_AGENT = "ReaderGeneratedHeaderProbe/1.0"
COOKIE = "generated_session=GENERATED_SESSION_ONLY"
JSON_BODY = "q=生成中文&value=保持原始字节".encode("utf-8")


def require(value, reason):
    if not value:
        raise ValueError(reason)


def values(row, name):
    return [value for key, value in row["headers"] if key.lower() == name.lower()]


def validate(report, jar_sha, worker_sha, revision, architecture):
    require(report.get("schemaVersion") == 1 and report.get("jarSha256") == jar_sha and
            report.get("workerSha256") == worker_sha and report.get("revision") == revision and
            report.get("architecture") == architecture, "TLS artifact identity mismatch")
    for key in ("generatedOnly", "fixtureOnlyPrivateDistributionPolicy", "fixtureOnlyPrivateFontconfigTmpfs",
                "fixtureOnlyPrivateAppDataTmpfs", "httpsTested"):
        require(report.get(key) is True, "Missing TLS isolation evidence: " + key)
    for key in ("realCredentialsImported", "privateBookBodyRead", "readerJarStarted",
                "hostTrustStoreChanged", "ignoreHttpsErrorsUsed", "workerLaunchOverridden",
                "fullGoalComplete"):
        require(report.get(key) is False, "Invalid TLS scope: " + key)
    identity = report["identity"]
    require(identity["uid"] == identity["gid"] == 10001 and
            isinstance(identity["groups"], list) and set(identity["groups"]) <= {10001} and
            identity["interfaces"] == ["lo"] and identity["capsZero"] is True and
            identity["noNewPrivileges"] is True, "TLS container identity mismatch")
    spec = importlib.util.spec_from_file_location("tls_resource_validator",
        Path(__file__).with_name("report-browser-cgroup.py"))
    resources = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(resources)
    resources.verify_report(report["resources"], require_no_swap=True)
    results = report["results"]
    require(isinstance(results, list) and len(results) == 12 and
            tuple(row.get("case") for row in results) == CASES, "Missing or duplicated TLS cases")
    total = fields = 0
    for result in results[:10]:
        case = result["case"]
        require(result["failureType"] is None and result["rejections"] == 0 and
                result["everyActualTargetConnectionUsedProxy"] is True, "HTTPS request did not complete safely")
        rows, tunnels = result["targetRequests"], result["tunnels"]
        count = {"resources": 3, "resources-redirect": 5}.get(case, 2)
        require(len(rows) == len(tunnels) == len(result["observedTargetSourcePorts"]) == count,
                "Incorrect actual HTTPS request/tunnel count")
        require(len({t["targetSourcePort"] for t in tunnels}) == count,
                "Duplicate target TCP source port")
        require(sorted(r["targetSourcePort"] for r in rows) ==
                sorted(result["observedTargetSourcePorts"]), "Target port evidence mismatch")
        by_port = {t["targetSourcePort"]: t for t in tunnels}
        for row in rows:
            pairs = row["headers"]
            require(isinstance(pairs, list) and 1 <= len(pairs) <= 64 and
                    all(isinstance(p, list) and len(p) == 2 and
                        all(isinstance(v, str) for v in p) for p in pairs) and
                    sum(len(v.encode("utf-8")) for p in pairs for v in p) <= 8192,
                    "Invalid bounded HTTP header observation")
            primary = row["actor"] == "primary"
            require(row["actor"] in ("primary", "secondary") and row["case"] == case,
                    "Unexpected HTTPS target")
            hosts = values(row, "Host")
            require(len(hosts) == 1 and re.fullmatch(r"127\.0\.0\.[12]:[0-9]{1,5}", hosts[0]),
                    "Invalid actual HTTP Host")
            host, port = hosts[0].rsplit(":", 1)
            tunnel = by_port.get(row["targetSourcePort"])
            require(host == ("127.0.0.1" if primary else "127.0.0.2") and
                    tunnel is not None and tunnel["method"] == "CONNECT" and
                    (host, int(port)) == (tunnel["host"], tunnel["port"]) and
                    0 < tunnel["clientToServerBytes"] <= 4194304 and
                    0 < tunnel["serverToClientBytes"] <= 4194304,
                    "HTTP Host differs from the actual proxy CONNECT target")
            require(values(row, "Authorization") == ([AUTH] if primary else []) and
                    values(row, "X-Generated-Trace") == ([TRACE] if primary else []) and
                    values(row, "User-Agent") == [USER_AGENT], "Cross-origin header leakage or same-origin loss")
            fields += len(pairs)
        require(sum(r["actor"] == "secondary" for r in rows) ==
                (0 if case.startswith("same-") else 1), "Wrong redirect destination")
        output = result["workerResult"]
        require("认证头差分书-" + case in output["body"], "Actual page JavaScript did not complete")
        if case.endswith(("307", "308")):
            require(all(r["method"] == "POST" and r["bodyByteCount"] == len(JSON_BODY) and
                        r["bodySha256"] == hashlib.sha256(JSON_BODY).hexdigest() and
                        values(r, "Content-Type") == ["application/json; charset=UTF-8"] for r in rows),
                    "307/308 changed JSON body bytes or media type")
            require(values(rows[0], "Cookie") == [] and values(rows[1], "Cookie") ==
                    ([COOKIE] if case.startswith("same-") else []), "Secure Cookie crossed host boundary")
            cookies = output["cookies"]
            require(len(cookies) == 1, "Missing structured Cookie result")
            cookie = cookies[0]
            require(all(cookie.get(k) == v for k, v in {
                "name": "generated_session", "value": "GENERATED_SESSION_ONLY", "path": "/probe/",
                "domain": "127.0.0.1", "sameSite": "Lax"}.items()) and
                all(cookie.get(k) is True for k in ("httpOnly", "secure", "hostOnly")) and
                cookie.get("deleted") is False, "Cookie metadata changed")
        elif case == "cross-post":
            require([r["method"] for r in rows] == ["POST", "GET"] and
                    [r["bodyByteCount"] for r in rows] == [23, 0] and
                    rows[0]["bodySha256"] == hashlib.sha256(b"q=generated-header-only").hexdigest() and
                    values(rows[0], "Content-Type") == ["application/x-www-form-urlencoded; charset=UTF-8"] and
                    values(rows[1], "Content-Type") == [], "303 form redirect changed contract")
        else:
            require(all(r["method"] == "GET" and r["bodyByteCount"] == 0 for r in rows),
                    "GET unexpectedly carried a body")
        require(all(r["bodySha256"] == hashlib.sha256(b"").hexdigest() for r in rows if r["method"] == "GET"),
                "Empty GET bytes changed")
        if not case.endswith(("307", "308")):
            require(output["cookies"] == [], "Unexpected Cookie state")
        total += len(rows)
    require(total == 24, "Incomplete actual HTTPS target observations")
    for result, code in zip(results[10:], ("SSL_ERROR_BAD_CERT_DOMAIN", "SEC_ERROR_UNKNOWN_ISSUER")):
        require(result["failureType"] == "Error" and result["tlsErrorCodes"] == [code] and
                result["workerResult"] is None and result["negativeHttpRequests"] == 0 and
                result["targetRequests"] == [] and len(result["tunnels"]) == 1,
                "Invalid certificate was accepted or its failure cause was not proven")
        tunnel = result["tunnels"][0]
        require(tunnel["method"] == "CONNECT" and tunnel["host"] == "127.0.0.1" and
                0 < tunnel["clientToServerBytes"] <= 4194304 and
                0 < tunnel["serverToClientBytes"] <= 4194304, "Negative TLS handshake was not observed")
    return {"acceptedPackagedWorkerHttpsSubset": True, "positiveCases": 10,
            "negativeCertificateControls": 2, "targetRequests": total,
            "parsedHeaderFields": fields, "readerApiOrProductionAccepted": False}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("--jar-sha", required=True)
    parser.add_argument("--worker-sha", required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--architecture", choices=("amd64", "arm64"), required=True)
    arguments = parser.parse_args(argv)
    require(re.fullmatch(r"[0-9a-f]{64}", arguments.jar_sha) and
            re.fullmatch(r"[0-9a-f]{64}", arguments.worker_sha) and
            re.fullmatch(r"[0-9a-f]{40}", arguments.revision), "Invalid expected TLS identity")
    require(arguments.report.is_file() and not arguments.report.is_symlink() and
            0 < arguments.report.stat().st_size <= 524288, "TLS report must be a bounded regular file")
    report = json.loads(arguments.report.read_bytes())
    print(json.dumps(validate(report, arguments.jar_sha, arguments.worker_sha,
                              arguments.revision, arguments.architecture)))


if __name__ == "__main__":
    main()
