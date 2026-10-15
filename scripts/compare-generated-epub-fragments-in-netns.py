"""Compare the UI's exported, generated NCX/EPUB3 fragment fixtures with both JARs.

Run as root under `unshare --net --fork`. The actual kernel netns is checked
before any input is read; Java then runs as nobody. No production storage,
private book or authenticated remote source is accepted by this diagnostic.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.parse
from xml.etree import ElementTree
from zipfile import ZipFile

SPEC = importlib.util.spec_from_file_location("storage_probe",
    Path(__file__).with_name("compare-authorized-storage-in-netns.py"))
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
LIMIT = 2 * 1024 * 1024
FIXTURES = {
    "ncx": ("generated-ncx-fragments.epub", "synthetic-fragments", ["Text/shared.xhtml"]),
    "nav": ("generated-nav-fragments.epub", "synthetic-nav-fragments",
            ["Text/shared.xhtml", "Text/later.xhtml"]),
}
TIME_FIELDS = {"durChapterTime", "lastCheckTime", "latestChapterTime"}


def fixture_bytes(path, identity):
    if path.stat().st_size > LIMIT:
        raise ValueError("Generated fixture exceeds the diagnostic size limit")
    data = path.read_bytes()
    with ZipFile(path) as archive:
        entries = archive.infolist()
        allowed = {"mimetype", "META-INF/container.xml", "OEBPS/book.opf",
                   "OEBPS/toc.ncx", "OEBPS/Text/shared.xhtml"}
        if identity == "synthetic-nav-fragments":
            allowed |= {"OEBPS/Nav/toc.xhtml", "OEBPS/Text/later.xhtml"}
        names = [entry.filename for entry in entries]
        if (set(names) != allowed or len(names) != len(allowed)
                or names[0] != "mimetype" or sum(entry.file_size for entry in entries) > LIMIT):
            raise ValueError("Not one of the exported generated UI fixtures")
        if archive.read("mimetype") != b"application/epub+zip":
            raise ValueError("Generated fixture mimetype is invalid")
        package = ElementTree.fromstring(archive.read("OEBPS/book.opf"))
        identifiers = package.findall(".//{http://purl.org/dc/elements/1.1/}identifier")
        if len(identifiers) != 1 or identifiers[0].text != identity:
            raise ValueError("Generated fixture identifier does not match")
        for name in names[1:]:
            ElementTree.fromstring(archive.read(name))
    return data


def normalize(value, base, owner, workdir, key=""):
    # Preserve defaults and JSON types; mask only independently variable times.
    if key in TIME_FIELDS and type(value) is int and value > 0:
        return "__OBSERVED_TIME__"
    if isinstance(value, str):
        return value.replace(base, "__API_ROOT__").replace(str(workdir), "__WORKDIR__").replace(owner, "__OWNER__")
    if isinstance(value, list):
        return [normalize(item, base, owner, workdir) for item in value]
    if isinstance(value, dict):
        return {name: normalize(item, base, owner, workdir, name) for name, item in value.items()}
    return value


def projected(reply, base, owner, workdir):
    body = normalize(reply["value"], base, owner, workdir)
    data = body.get("data")
    if isinstance(data, dict) and isinstance(data.get("content"), str):
        content = data["content"]
        data["content"] = {"sha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
                           "characters": len(content), "replacementCharacters": content.count("\ufffd")}
    return {"status": reply["status"], "body": body,
            "replacementCharacter": reply["replacementCharacter"]}


def run(java, jar, workdir, fixtures, owner, password, startup_failure_log=None):
    workdir.mkdir()
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    base = f"http://127.0.0.1:{port}"
    command = [str(java), "-XX:ActiveProcessorCount=2", "-Xms128m", "-Xmx768m", "-jar", str(jar),
        f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
        "--reader.server.bindAddress=127.0.0.1", "--reader.app.secure=true",
        "--reader.app.licenseCheckEnabled=false", "--reader.app.shelfUpdateInteval=0",
        "--reader.app.remoteBookSourceUpdateInterval=0", "--reader.app.autoBackupUserData=false",
        "--reader.app.autoClearInactiveUser=0"]
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(command, cwd=workdir, stdout=log, stderr=log)
        try:
            account = BASE.new_opener()
            deadline = time.monotonic() + 75
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    if startup_failure_log is not None:
                        # Only this generated run's pre-login startup, never production logs.
                        # Exclusive output preserves an earlier failed attempt.
                        with startup_failure_log.open("xb") as evidence:
                            with (workdir / "reader.log").open("rb") as source:
                                evidence.write(source.read(65536))
                    raise RuntimeError("Isolated Reader exited before readiness")
                try:
                    BASE.success(BASE.call(account, base, "getSystemInfo"))
                    break
                except (OSError, ValueError, RuntimeError):
                    time.sleep(0.25)
            else:
                raise TimeoutError("Isolated Reader startup timed out")
            BASE.success(BASE.call(account, base, "login",
                {"username": owner, "password": password, "isLogin": False}))
            login = BASE.success(BASE.call(account, base, "login",
                {"username": owner, "password": password, "isLogin": True}))
            token = login.get("accessToken", "")
            result = {"accessTokenNamespaced": token.startswith(owner + ":"), "cases": {}}
            other = BASE.new_opener()
            BASE.success(BASE.call(other, base, "login",
                {"username": owner + "b", "password": password, "isLogin": False}))
            for label, data in fixtures.items():
                url = f"storage/data/{owner}/{label}.epub"
                target = workdir / url / "index.epub"
                target.parent.mkdir(parents=True)
                target.write_bytes(data)
                saved = BASE.call(account, base, "saveBook", {"bookUrl": url, "originName": url,
                    "origin": "loc_book", "name": "Generated " + label, "author": "测试作者",
                    "type": 0, "tocUrl": "toc"})
                BASE.success(saved)
                escaped = urllib.parse.quote(url, safe="")
                toc = BASE.call(account, base, "getChapterList?url=" + escaped + "&refresh=1")
                chapters = BASE.success(toc)
                expected = FIXTURES[label][2]
                if ([row.get("url") for row in chapters] != expected
                        or [row.get("index") for row in chapters] != list(range(len(expected)))):
                    raise ValueError("Generated fixture did not preserve the expected resource-level indices")
                content = []
                for index in range(len(expected)):
                    reply = BASE.call(account, base, "getBookContent?url=" + escaped
                                      + f"&index={index}&epubContent=1")
                    BASE.success(reply)
                    content.append(projected(reply, base, owner, workdir))
                shelf = BASE.success(BASE.call(account, base, "getBookshelf?refresh=0"))
                book = next(item for item in shelf if item.get("bookUrl") == url)
                if book.get("durChapterIndex") != len(expected) - 1:
                    raise ValueError("Legacy content read did not save the final resource index")
                raw_route = "file/download?home=__HOME__&path=" + urllib.parse.quote(label + ".epub/index.epub", safe="") + "&stream=1"
                with account.open(base + "/reader3/" + raw_route, timeout=30) as response:
                    raw = response.read(LIMIT + 1)
                    download = {"status": response.status, "bytes": len(raw),
                        "sha256": hashlib.sha256(raw).hexdigest(),
                        "contentType": response.headers.get("Content-Type", ""),
                        "contentDisposition": response.headers.get("Content-Disposition", "")}
                if download["status"] != 200 or raw != data:
                    raise ValueError("Generated raw EPUB download bytes changed")
                anonymous = BASE.call(BASE.new_opener(), base, "getChapterList?url=" + escaped)
                denied = BASE.call(other, base, "getChapterList?url=" + escaped)
                token_toc = BASE.call(BASE.new_opener(), base, "getChapterList?url=" + escaped
                    + "&accessToken=" + urllib.parse.quote(token, safe=""))
                BASE.success(token_toc)
                facts = {"sourceUnchanged": BASE.digest(target) == hashlib.sha256(data).hexdigest(),
                    "anonymousDenied": anonymous["value"].get("isSuccess") is False
                        and anonymous["value"].get("data") == "NEED_LOGIN",
                    "otherNamespaceDenied": denied["value"].get("isSuccess") is False,
                    "tokenChapterListMatchesSession": BASE.same_json(token_toc["value"], toc["value"])}
                if not all(facts.values()):
                    raise ValueError("Generated fixture namespace or read-only boundary failed")
                result["cases"][label] = {"save": projected(saved, base, owner, workdir),
                    "toc": projected(toc, base, owner, workdir), "content": content, "download": download,
                    "shelf": normalize(book, base, owner, workdir), "facts": facts}
            return result
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("java", "original", "restored", "fixtures", "report"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    BASE.isolate_then_drop()  # Real kernel guard before reads, acknowledgment or launch.
    if BASE.digest(args.original) != BASE.ORIGINAL_SHA256:
        parser.error("Original JAR SHA-256 does not match the read-only baseline")
    report = args.report.resolve()
    if not report.is_relative_to(Path("/var/tmp")) or report.exists():
        parser.error("Report must be a new file beneath /var/tmp")
    fixtures = {label: fixture_bytes(args.fixtures / name, identity)
                for label, (name, identity, _) in FIXTURES.items()}
    original_hash, restored_hash = BASE.digest(args.original), BASE.digest(args.restored)
    owner, password = "fragmentprobe" + secrets.token_hex(5), "Probe-" + secrets.token_hex(18)
    with tempfile.TemporaryDirectory(prefix="reader-generated-fragments-") as temporary:
        root = Path(temporary)
        original = run(args.java, args.original, root / "original", fixtures, owner, password,
                       report.with_name(report.stem + "-original-startup-failure.log"))
        restored = run(args.java, args.restored, root / "restored", fixtures, owner, password,
                       report.with_name(report.stem + "-restored-startup-failure.log"))
    if BASE.digest(args.original) != original_hash or BASE.digest(args.restored) != restored_hash:
        raise RuntimeError("Read-only JAR input changed")
    equal = BASE.same_json(original, restored)
    result = {"originalJarSha256": original_hash, "restoredJarSha256": restored_hash,
        "fixtureSha256": {label: hashlib.sha256(data).hexdigest() for label, data in fixtures.items()},
        "originalExecuted": True, "network": "private-loopback-only-netns",
        "javaUserId": 65534, "privateBookRead": False, "jarInputsUnchanged": True,
        "original": original, "restored": restored, "equal": equal,
        "projection": "Full JSON except named positive timestamps; XHTML replaced by exact normalized-byte hash",
        "maskedTimeFields": sorted(TIME_FIELDS),
        "limitations": ["Generated two-fixture backend/API sample, not the original UI",
            "No original or production remote WebView, authenticated real source, or production storage",
            "Frontend anchor pixels and iframe sandbox are verified by a separate browser journey"]}
    with report.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    print(json.dumps({"equal": equal, "originalExecuted": True, "fixtureCount": len(fixtures),
                      "originalJarSha256": original_hash, "restoredJarSha256": restored_hash}))
    if not equal:
        raise SystemExit("Generated fragment API results differ; inspect the aggregate report")


if __name__ == "__main__":
    main()
