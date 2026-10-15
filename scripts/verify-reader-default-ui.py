"""Observe the default Vue 3 entry of an isolated running Reader image.

Only bounded loopback GETs of packaged static resources; no account, cookies,
book data, source rules or production URL. This is not browser UI acceptance.
"""
import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import sys
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener
from zipfile import ZipFile


SCOPE = "default image Vue 3 entry and packaged static bytes; not rendered UI acceptance"
ROUTES = ("/", "/login", "/search", "/reader/generated-book")
MAX_BODY = 8 * 1024 * 1024
MAX_TOTAL = 16 * 1024 * 1024
MAX_ASSETS = 16
SHA = re.compile(r"[0-9a-f]{64}\Z")
ASSET = re.compile(r"/static/[A-Za-z0-9_-]+\.(js|css)\Z")
ROOT = "BOOT-INF/classes/web-vue3/"


def sha256(body):
    return hashlib.sha256(body).hexdigest()


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, new_url):
        raise ValueError("Static UI redirects are not allowed")


class EntryAssets(HTMLParser):
    def __init__(self):
        super().__init__()
        self.assets = []
        self.app = False
        self.module = False

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "div" and attrs.get("id") == "app":
            self.app = True
        if tag == "script" and attrs.get("type") == "module":
            self.module = True
            self.assets.append(attrs.get("src", ""))
        if tag == "link" and attrs.get("rel") in ("stylesheet", "modulepreload"):
            self.assets.append(attrs.get("href", ""))


def jar_entry(jar, path):
    entries = [item for item in jar.infolist() if item.filename == path]
    if len(entries) != 1 or entries[0].file_size > MAX_BODY:
        raise ValueError("Missing, duplicate or oversized packaged UI entry")
    return jar.read(entries[0])


def observe(base, jar_path):
    address = urlsplit(base)
    if (address.scheme != "http" or address.hostname != "127.0.0.1" or
            address.username or address.password or not address.port or
            address.path not in ("", "/") or address.query or address.fragment):
        raise ValueError("Only an explicit local isolated Reader port is allowed")
    base = base.rstrip("/")
    opener = build_opener(ProxyHandler({}), NoRedirect())
    total = 0

    def get(path, kind):
        nonlocal total
        request = Request(base + path, headers={"Accept": "text/html" if kind == "html" else "*/*"})
        try:
            response = opener.open(request, timeout=10)
        except HTTPError as error:
            error.close()
            raise ValueError("Static UI request failed") from None
        with response:
            status = response.status
            mime = response.headers.get_content_type()
            charset = response.headers.get_content_charset()
            if status != 200 or (kind == "html" and (mime != "text/html" or charset != "utf-8")):
                raise ValueError("Invalid UI entry status or encoding")
            if kind == "js" and mime not in ("application/javascript", "text/javascript"):
                raise ValueError("Invalid JavaScript content type")
            if kind == "css" and mime != "text/css":
                raise ValueError("Invalid CSS content type")
            body = response.read(MAX_BODY + 1)
            total += len(body)
            if len(body) > MAX_BODY or total > MAX_TOTAL:
                raise ValueError("UI observation byte budget exceeded")
            return body, status, mime

    digest = hashlib.sha256()
    with Path(jar_path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    report = {"schemaVersion": 1, "scope": SCOPE, "ui": "vue3", "jarSha256": digest.hexdigest(),
              "routes": [], "assets": [], "legacyEntryDifferent": False, "passed": False}
    with ZipFile(jar_path) as jar:
        expected = jar_entry(jar, ROOT + "index.html")
        text = expected.decode("utf-8", errors="strict")
        if "<title>夜读 · Reader</title>" not in text:
            raise ValueError("Packaged Vue 3 title is missing")
        parser = EntryAssets()
        parser.feed(text)
        if (not parser.app or not parser.module or not 2 <= len(parser.assets) <= MAX_ASSETS or
                len(set(parser.assets)) != len(parser.assets) or
                any(not ASSET.fullmatch(path) for path in parser.assets) or
                not any(path.endswith(".css") for path in parser.assets)):
            raise ValueError("Packaged Vue 3 entry assets are invalid")
        legacy = jar_entry(jar, "BOOT-INF/classes/web/index.html")
        if legacy == expected:
            raise ValueError("Vue 3 entry must differ from packaged legacy rollback")
        report["legacyEntryDifferent"] = True
        for path in ROUTES:
            body, status, mime = get(path, "html")
            if body != expected:
                raise ValueError("Default entry is not the packaged Vue 3 HTML")
            report["routes"].append({"path": path, "status": status, "contentType": mime,
                                     "sha256": sha256(body), "packagedBytesMatch": True})
        for path in parser.assets:
            kind = "js" if path.endswith(".js") else "css"
            body, status, mime = get(path, kind)
            if body != jar_entry(jar, ROOT + path.lstrip("/")):
                raise ValueError("Served asset differs from the packaged Vue 3 bytes")
            report["assets"].append({"path": path, "status": status, "contentType": mime,
                                     "sha256": sha256(body), "packagedBytesMatch": True})
    report["passed"] = True
    verify_report(report, report["jarSha256"])
    return report


def verify_report(report, expected_jar_sha):
    if (not isinstance(report, dict) or set(report) !=
            {"schemaVersion", "scope", "ui", "jarSha256", "routes", "assets", "legacyEntryDifferent", "passed"} or
            type(report["schemaVersion"]) is not int or report["schemaVersion"] != 1 or
            report["scope"] != SCOPE or report["ui"] != "vue3" or
            type(report["jarSha256"]) is not str or not SHA.fullmatch(report["jarSha256"]) or
            report["jarSha256"] != expected_jar_sha or report["legacyEntryDifferent"] is not True or
            report["passed"] is not True or not isinstance(report["routes"], list) or
            not isinstance(report["assets"], list) or len(report["routes"]) != len(ROUTES) or
            not 2 <= len(report["assets"]) <= MAX_ASSETS):
        raise ValueError("Invalid default Vue 3 observation report")
    if [entry.get("path") for entry in report["routes"] if isinstance(entry, dict)] != list(ROUTES):
        raise ValueError("Missing or reordered default Vue 3 entry routes")
    digests = set()
    assets = set()
    for group in ("routes", "assets"):
        for entry in report[group]:
            if (not isinstance(entry, dict) or set(entry) !=
                    {"path", "status", "contentType", "sha256", "packagedBytesMatch"} or
                    type(entry["path"]) is not str or type(entry["status"]) is not int or entry["status"] != 200 or
                    type(entry["sha256"]) is not str or not SHA.fullmatch(entry["sha256"]) or
                    entry["packagedBytesMatch"] is not True):
                raise ValueError("Invalid default Vue 3 response observation")
            if group == "routes":
                if entry["contentType"] != "text/html":
                    raise ValueError("Invalid entry MIME type")
                digests.add(entry["sha256"])
            else:
                path = entry["path"]
                if not ASSET.fullmatch(path) or path in assets:
                    raise ValueError("Invalid or duplicate default Vue 3 asset")
                assets.add(path)
                accepted = ("application/javascript", "text/javascript") if path.endswith(".js") else ("text/css",)
                if entry["contentType"] not in accepted:
                    raise ValueError("Invalid asset MIME type")
    if len(digests) != 1 or not any(path.endswith(".js") for path in assets) or not any(path.endswith(".css") for path in assets):
        raise ValueError("Inconsistent entry or missing JavaScript/CSS observations")


def strict_json(text):
    def object_pairs(pairs):
        value = {}
        for key, item in pairs:
            if key in value:
                raise ValueError("Duplicate report field")
            value[key] = item
        return value
    return json.loads(text, object_pairs_hook=object_pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError("Invalid number")))


def main():
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="mode", required=True)
    probe = commands.add_parser("probe")
    probe.add_argument("--reader-base", required=True)
    probe.add_argument("--jar", required=True)
    check = commands.add_parser("check")
    check.add_argument("report")
    check.add_argument("--expected-jar-sha", required=True)
    args = parser.parse_args()
    try:
        if args.mode == "probe":
            print(json.dumps(observe(args.reader_base, args.jar), sort_keys=True))
        else:
            path = Path(args.report)
            if path.stat().st_size > 16384:
                raise ValueError("Default UI report exceeds the size budget")
            verify_report(strict_json(path.read_text(encoding="utf-8")), args.expected_jar_sha)
    except Exception:
        # No HTML, raw network errors or dynamic server messages in CI logs.
        print("Default packaged Vue 3 entry gate failed", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
