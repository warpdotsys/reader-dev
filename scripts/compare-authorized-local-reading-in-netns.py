"""Compare only the owner's authorized local EPUB, without network egress.

This is an opt-in local diagnostic, not a CI fixture. Private JSON, chapter
titles, HTML, credentials and original paths are never written to its report.
Run under root `unshare --net --fork`; the kernel isolation is checked before
any private input is read or either JAR is launched.
"""
import argparse
import hashlib
from html.parser import HTMLParser
import importlib.util
import json
from pathlib import Path
import secrets
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse

SPEC = importlib.util.spec_from_file_location("storage_probe",
    Path(__file__).with_name("compare-authorized-storage-in-netns.py"))
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
ALLOWED_TITLE = "黎明之剑"
ALLOWED_SHA256 = "bd8c68194bb26a3c3bc5edc7b6baa7023c584441f1c2835f71fab3a3bf3ced31"


def select_book(shelf):
    matches = [book for book in shelf if book.get("origin") == "loc_book"
               and book.get("name") == ALLOWED_TITLE]
    if len(matches) != 1:
        raise ValueError("Expected exactly one authorized local title")
    return matches[0]


class BodyText(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.depth = self.hidden = 0
        self.parts = []

    def handle_starttag(self, tag, attrs):
        if tag == "body":
            self.depth += 1
        if tag in ("script", "style"):
            self.hidden += 1

    def handle_endtag(self, tag):
        if tag == "body":
            self.depth = max(0, self.depth - 1)
        if tag in ("script", "style"):
            self.hidden = max(0, self.hidden - 1)

    def handle_data(self, data):
        if self.depth and not self.hidden:
            self.parts.append(data)


def html_facts(content):
    parser = BodyText()
    parser.feed(content)
    text = " ".join(" ".join(parser.parts).split())
    return {"htmlSha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
            "bodyTextSha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
            "bodyCharacters": len(text), "replacementCharacters": text.count("\ufffd")}


def envelope(reply):
    value = reply["value"]
    return {"status": reply["status"], "isSuccess": value.get("isSuccess") is True,
            "errorEmpty": value.get("errorMsg") == "",
            "envelopeFields": sorted(value)}


def run(java, jar, workdir, book, asset, username, password):
    workdir.mkdir()
    url = f"storage/data/{username}/authorized.epub"
    target = workdir / url / "index.epub"
    target.parent.mkdir(parents=True)
    shutil.copyfile(asset, target)
    record = dict(book, bookUrl=url, originName=url)
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    base = f"http://127.0.0.1:{port}"
    command = [str(java), "-Xms128m", "-Xmx768m", "-jar", str(jar),
        f"--reader.app.workDir={workdir}", f"--reader.server.port={port}",
        "--reader.server.bindAddress=127.0.0.1", "--reader.app.secure=true",
        "--reader.app.licenseCheckEnabled=false", "--reader.app.shelfUpdateInteval=0",
        "--reader.app.remoteBookSourceUpdateInterval=0",
        "--reader.app.autoBackupUserData=false", "--reader.app.autoClearInactiveUser=0"]
    with (workdir / "reader.log").open("wb") as log:
        process = subprocess.Popen(command, cwd=workdir, stdout=log, stderr=log)
        try:
            account = BASE.new_opener()
            deadline = time.monotonic() + 75
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError("Isolated Reader exited before readiness")
                try:
                    BASE.success(BASE.call(account, base, "getSystemInfo"))
                    break
                except (OSError, ValueError, RuntimeError):
                    time.sleep(0.25)
            else:
                raise TimeoutError("Isolated Reader startup timed out")
            BASE.success(BASE.call(account, base, "login", {"username": username,
                "password": password, "isLogin": False}))
            shelf_file = workdir / "storage/data" / username / "bookshelf.json"
            shelf_file.write_text(json.dumps([record], ensure_ascii=False), encoding="utf-8")
            before = shelf_file.read_bytes()
            escaped = urllib.parse.quote(url, safe="")
            toc = BASE.call(account, base, "getChapterList?url=" + escaped)
            chapters = BASE.success(toc)
            if not isinstance(chapters, list) or len(chapters) != book["totalChapterNum"]:
                raise RuntimeError("Authorized EPUB chapter count does not match its stored record")
            projection = [{"index": item.get("index"), "url": item.get("url"),
                           "title": item.get("title")} for item in chapters]
            after_toc = shelf_file.read_bytes()
            chapter_results = {}
            for index in sorted({0, 1, len(chapters) // 2, len(chapters) - 1}):
                reply = BASE.call(account, base, "getBookContent?url=" + escaped
                    + f"&index={index}&cache=1&epubContent=1")
                data = BASE.success(reply)
                if not isinstance(data, dict) or not isinstance(data.get("content"), str):
                    raise RuntimeError("Authorized EPUB did not return chapter HTML")
                resource = data.get("url", "").removeprefix("__API_ROOT__")
                if not resource.startswith("/book-assets/") or resource.startswith("//"):
                    raise RuntimeError("Authorized EPUB returned an unexpected asset URL")
                request = urllib.parse.quote(resource, safe="/")
                with account.open(base + request, timeout=30) as response:
                    raw = response.read()
                    asset_status = response.status
                    asset_type = response.headers.get("Content-Type", "")
                # The legacy asset route injects reader JavaScript into its
                # extracted XHTML. Compare its bytes to a fresh HTML API read,
                # not to the pre-injection read; do not modify the EPUB source.
                after_asset = BASE.call(account, base, "getBookContent?url=" + escaped
                    + f"&index={index}&cache=1&epubContent=1")
                current_html = BASE.success(after_asset)["content"]
                chapter_results[str(index)] = {**envelope(reply), **html_facts(data["content"]),
                    "assetStatus": asset_status, "assetContentType": asset_type,
                    "legacyAssetInjectionObserved": raw != data["content"].encode("utf-8"),
                    "assetMatchesHtmlAfterInjection": raw == current_html.encode("utf-8"),
                    "bodyTextUnchangedByInjection": html_facts(current_html)["bodyTextSha256"]
                        == html_facts(data["content"])["bodyTextSha256"]}
            other = BASE.new_opener()
            BASE.success(BASE.call(other, base, "login", {"username": username + "b",
                "password": password, "isLogin": False}))
            denial = BASE.call(other, base, "getChapterList?url=" + escaped)
            anonymous = BASE.call(BASE.new_opener(), base, "getChapterList?url=" + escaped)
            return {"toc": {**envelope(toc), "count": len(chapters),
                       "projectionSha256": hashlib.sha256(json.dumps(projection,
                           ensure_ascii=False, sort_keys=True).encode("utf-8")).hexdigest(),
                       "titleReplacementCharacters": sum(item.get("title", "").count("\ufffd")
                                                          for item in chapters)},
                    "chapters": chapter_results,
                    "tocUpdatedDisposableShelf": after_toc != before,
                    "facts": {"shelfUnchangedAfterTocByCachedReads": shelf_file.read_bytes() == after_toc,
                        "progressUnchanged": all(json.loads(shelf_file.read_bytes())[0].get(key)
                            == record.get(key) for key in ("durChapterIndex", "durChapterTitle",
                                                          "durChapterPos", "durChapterTime")),
                        "sourceUnchanged": BASE.digest(target) == ALLOWED_SHA256,
                        "otherNamespaceDenied": denial["value"].get("isSuccess") is False,
                        "anonymousDenied": anonymous["value"].get("isSuccess") is False
                            and anonymous["value"].get("data") == "NEED_LOGIN"}}
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
    for name in ("java", "original", "restored", "shelf", "asset", "report"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    BASE.isolate_then_drop()  # Kernel guard before private reads or JAR launch.
    if BASE.digest(args.original) != BASE.ORIGINAL_SHA256:
        parser.error("Original JAR does not match the read-only baseline")
    if BASE.digest(args.asset) != ALLOWED_SHA256:
        parser.error("Asset is not the single authorized original EPUB")
    report = args.report.resolve()
    if not report.is_relative_to(Path("/var/tmp")) or report.exists():
        parser.error("Report must be a new file beneath /var/tmp")
    shelf_bytes = args.shelf.read_bytes()
    book = select_book(json.loads(shelf_bytes.decode("utf-8")))
    original_hash, restored_hash = BASE.digest(args.original), BASE.digest(args.restored)
    username, password = "localprobe" + secrets.token_hex(5), "Probe-" + secrets.token_hex(12)
    with tempfile.TemporaryDirectory(prefix="reader-authorized-local-diff-") as temporary:
        root = Path(temporary)
        original = run(args.java, args.original, root / "original", book, args.asset, username, password)
        restored = run(args.java, args.restored, root / "restored", book, args.asset, username, password)
    if (args.shelf.read_bytes() != shelf_bytes or BASE.digest(args.asset) != ALLOWED_SHA256
            or BASE.digest(args.original) != original_hash or BASE.digest(args.restored) != restored_hash):
        raise RuntimeError("Read-only inputs changed during comparison")
    result = {"originalJarSha256": original_hash, "restoredJarSha256": restored_hash,
        "assetSha256": ALLOWED_SHA256, "assetBytes": args.asset.stat().st_size,
        "authorizedBookCount": 1, "privateDataPublished": False,
        "network": "private-loopback-only-netns", "originalExecuted": True,
        "sourceUnchanged": True, "original": original, "restored": restored,
        "tocEqual": BASE.same_json(original["toc"], restored["toc"]),
        "sampledChaptersEqual": BASE.same_json(original["chapters"], restored["chapters"]),
        "limitations": ["Only one owner-authorized EPUB and four sampled chapters",
                        "Synthetic accounts; input paths remapped within each isolated namespace",
                        "No production progress writes, private raw responses or original paths published"]}
    report.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False))
    if not all(original["facts"].values()) or not all(restored["facts"].values()):
        raise SystemExit("Read-only or namespace assertions failed; inspect aggregate report")


if __name__ == "__main__":
    main()
