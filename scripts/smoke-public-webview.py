#!/usr/bin/env python3
"""Opt-in, low-rate public site smoke for the Reader image's built-in browser.

Use only with a disposable Reader work directory. This script registers an
isolated synthetic account and one public, credential-free test source; it
does not read production storage or download book chapters.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import http.cookiejar
import json
import secrets
import urllib.parse
import urllib.request


SOURCE_URL = "https://m.jjjxsw.com"
PAGE_URL = SOURCE_URL + "/txt/"


def call(opener, base, path, body=None, timeout=60):
    data = json.dumps(body, ensure_ascii=False).encode("utf-8") if body is not None else None
    request = urllib.request.Request(
        base + path, data=data,
        headers={"Content-Type": "application/json; charset=utf-8"} if data else {},
        method="POST" if data else "GET")
    with opener.open(request, timeout=timeout) as response:
        value = json.loads(response.read().decode("utf-8"))
        if response.status != 200 or value.get("isSuccess") is not True:
            raise RuntimeError(f"{path}: HTTP {response.status}, "
                               f"isSuccess={value.get('isSuccess')}, "
                               f"errorMsg={value.get('errorMsg')}")
        return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reader-base", default="http://127.0.0.1:18890")
    args = parser.parse_args()
    parsed = urllib.parse.urlparse(args.reader_base)
    if parsed.scheme != "http" or parsed.hostname not in ("127.0.0.1", "localhost"):
        parser.error("Reader must be an isolated loopback HTTP endpoint")

    opener = urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    call(opener, args.reader_base, "/reader3/getSystemInfo")
    username = "publicwebview" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    for is_login in (False, True):
        call(opener, args.reader_base, "/reader3/login", {
            "username": username, "password": password, "isLogin": is_login})

    source = {
        "bookSourceUrl": SOURCE_URL,
        "bookSourceName": "Public WebView compatibility probe",
        "searchUrl": PAGE_URL + ', {"webView": true}',
        "ruleSearch": {
            "bookList": ".booklist_a .list_a",
            "name": ".main strong@text",
            "author": ".main span a@text",
            "bookUrl": ".main a@href",
        },
        "ruleToc": {"chapterList": ".chapter"},
    }
    call(opener, args.reader_base, "/reader3/saveBookSource", source)
    value = call(opener, args.reader_base, "/reader3/searchBook", {
        "key": "public-compatibility-probe", "page": 1,
        "bookSourceUrl": SOURCE_URL}, timeout=90)
    books = value.get("data")
    if not isinstance(books, list) or not books or len(books) > 100:
        raise RuntimeError("Public WebView search returned no bounded book list")
    if any(not isinstance(book, dict) or not book.get("name") or
           not (book.get("bookUrl") or "").startswith(SOURCE_URL + "/txt/")
           for book in books):
        raise RuntimeError("Public WebView search returned an invalid book URL or name")
    projection = [(book["name"], book["bookUrl"]) for book in books]
    checksum = hashlib.sha256(json.dumps(
        projection, ensure_ascii=False, separators=(",", ":")).encode("utf-8")).hexdigest()
    print(json.dumps({"source": PAGE_URL, "status": 200, "isSuccess": True,
                      "errorMsg": value.get("errorMsg"), "bookCount": len(books),
                      "projectionSha256": checksum,
                      "observedAt": datetime.now(timezone.utc).isoformat()},
                     ensure_ascii=False))


if __name__ == "__main__":
    main()
