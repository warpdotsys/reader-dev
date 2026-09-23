"""Low-rate black-box comparison using one source already configured on production.

Only the selected source rule is read over SSH. Production storage and API are not
modified; both JARs run locally with disposable data and synthetic accounts.
Neither source configuration nor chapter text is printed or persisted.
"""

import argparse
import hashlib
import http.cookiejar
import json
import re
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
def port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def source_from_production(index, namespace):
    if not re.fullmatch(r"[A-Za-z0-9_-]+", namespace):
        raise ValueError("Source namespace must contain only letters, digits, _ or -")
    remote_sources = f"/opt/reader-pro-restored/storage/data/{namespace}/bookSource.json"
    remote = (
        "python3 -c 'import json,sys; "
        f'x=json.load(open("{remote_sources}")); '
        "print(json.dumps(x[int(sys.argv[1])],ensure_ascii=False))' "
        f"{index}"
    )
    result = subprocess.run(
        ["ssh", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=yes",
         "-o", "ConnectTimeout=10", "root@cdn.medwarp.cn", remote],
        text=True, encoding="utf-8", capture_output=True, timeout=25, check=True,
    )
    source = json.loads(result.stdout)
    if not source.get("enabled", True) or not source.get("searchUrl"):
        raise ValueError("Selected source is disabled or lacks a search URL")
    if any(source.get(key) for key in ("header", "loginUrl", "loginUi", "loginCheckJs")):
        raise ValueError("Selected source has credentials or login configuration")
    return source


def request(opener, base, path, body=None, timeout=35):
    payload = json.dumps(body, ensure_ascii=False).encode("utf-8") if body is not None else None
    req = urllib.request.Request(
        base + path, data=payload,
        headers={"Content-Type": "application/json; charset=utf-8"} if payload else {},
        method="POST" if payload else "GET",
    )
    with opener.open(req, timeout=timeout) as response:
        raw = response.read()
        return response.status, json.loads(raw.decode("utf-8"))


def shape(item):
    if isinstance(item, list):
        return [shape(value) for value in item]
    if not isinstance(item, dict):
        return None
    result = {}
    for key, value in sorted(item.items()):
        if value is None:
            result[key] = "null"
        elif value == "":
            result[key] = "empty-string"
        elif isinstance(value, bool):
            result[key] = f"bool:{value}"
        elif isinstance(value, (int, float)) and value == 0:
            result[key] = "zero"
        elif value == []:
            result[key] = "empty-array"
        else:
            result[key] = "non-default"
    return result


def digest_rows(rows):
    raw = json.dumps(rows, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def success(status, value, step):
    if status != 200 or value.get("isSuccess") is not True:
        raise RuntimeError(f"{step}: HTTP {status}, {value.get('errorMsg')}")
    return value.get("data")


def read_sse(opener, base, value, source_search=False):
    arguments = {"lastIndex": -1, "searchSize": 1, "concurrentCount": 1}
    arguments["url" if source_search else "key"] = value
    params = urllib.parse.urlencode(arguments)
    endpoint = "/reader3/searchBookSourceSSE" if source_search else "/reader3/searchBookMultiSSE"
    req = urllib.request.Request(base + endpoint + "?" + params)
    with opener.open(req, timeout=55) as response:
        raw = response.read(2_000_001)
        status = response.status
        content_type = response.headers.get("Content-Type", "")
    if len(raw) > 2_000_000:
        raise RuntimeError("SSE response exceeded 2 MB safety limit")
    if status != 200 or not content_type.startswith("text/event-stream"):
        raise RuntimeError(f"SSE response was HTTP {status}, {content_type}")
    frames = []
    for block in raw.decode("utf-8").split("\n\n"):
        if not block.strip():
            continue
        lines = block.splitlines()
        event = next((line[7:].strip() for line in lines if line.startswith("event: ")), "message")
        data_lines = [line[6:] for line in lines if line.startswith("data: ")]
        if not data_lines:
            raise RuntimeError("SSE frame had no data")
        frames.append((event, json.loads("\n".join(data_lines))))
    if not frames or frames[-1][0] != "end":
        raise RuntimeError("SSE ended without a terminal end frame")
    if any(event == "error" for event, _ in frames):
        raise RuntimeError("SSE emitted an error frame")
    books = [book for event, data in frames if event == "message"
             for book in data.get("data", [])]
    return {"step": "source-sse" if source_search else "search-sse",
            "status": status, "contentType": content_type,
            "frameCount": len(frames), "count": len(books), "terminalEvent": frames[-1][0],
            "isEnd": frames[-1][1].get("isEnd"),
            "lastIndex": frames[-1][1].get("lastIndex"),
            "shape": shape(books[:1]),
            "resultSha256": digest_rows([(book.get("name"), book.get("bookUrl")) for book in books])}


def run_jar(jar, workdir, reader_port, source, query, credentials, book_url=None,
            book_result_index=0, with_sse=False):
    base = f"http://127.0.0.1:{reader_port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={reader_port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    result = {"steps": []}
    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited during startup: {process.returncode}")
            try:
                status, value = request(opener, base, "/reader3/getSystemInfo", timeout=2)
                if status == 200 and value.get("isSuccess"):
                    break
            except (OSError, ValueError, urllib.error.URLError):
                pass
            time.sleep(0.5)
        else:
            raise TimeoutError("Reader startup timeout")

        for registering in (False, True):
            status, value = request(
                opener, base, "/reader3/login",
                {"username": credentials[0], "password": credentials[1], "isLogin": registering},
            )
            success(status, value, "register" if not registering else "login")

        if with_sse:
            status, value = request(opener, base, "/reader3/saveBookSource", source)
            success(status, value, "source-save")
            result["steps"].append({"step": "source-save", "status": status,
                                    "isSuccess": value["isSuccess"],
                                    "errorMsg": value.get("errorMsg")})
            result["steps"].append(read_sse(opener, base, query))

        status, value = request(opener, base, "/reader3/searchBook",
                                {"key": query, "page": 1, "bookSource": source})
        books = success(status, value, "search")
        result["steps"].append({"step": "search", "status": status,
                                "isSuccess": value["isSuccess"], "errorMsg": value.get("errorMsg"),
                                "count": len(books), "shape": shape(books[:1]),
                                "resultSha256": digest_rows([(b.get("name"), b.get("bookUrl")) for b in books])})
        if not books:
            result["incomplete"] = "search returned no books"
            return result, None
        if book_url is None:
            if book_result_index >= len(books):
                result["incomplete"] = "selected book index is outside search results"
                return result, None
            book_url = books[book_result_index].get("bookUrl")
        if not book_url:
            result["incomplete"] = "selected book has no URL"
            return result, None
        result["bookUrlSha256"] = hashlib.sha256(book_url.encode("utf-8")).hexdigest()

        status, value = request(opener, base, "/reader3/getBookInfo",
                                {"url": book_url, "bookSource": source})
        book = success(status, value, "book-info")
        result["steps"].append({"step": "book-info", "status": status,
                                "isSuccess": value["isSuccess"], "errorMsg": value.get("errorMsg"),
                                "hasTocUrl": bool(book.get("tocUrl")), "shape": shape(book),
                                "resultSha256": digest_rows([book.get(key) for key in
                                                             ("name", "author", "bookUrl", "tocUrl")])})
        if with_sse:
            result["steps"].append(read_sse(opener, base, book_url, source_search=True))

        status, value = request(opener, base, "/reader3/getChapterList",
                                {"url": book_url, "bookSource": source, "refresh": 1})
        chapters = success(status, value, "chapter-list")
        result["steps"].append({"step": "chapter-list", "status": status,
                                "isSuccess": value["isSuccess"], "errorMsg": value.get("errorMsg"),
                                "count": len(chapters), "shape": shape(chapters[:1]),
                                "resultSha256": digest_rows([(c.get("title"), c.get("url"), c.get("index"))
                                                              for c in chapters])})
        if not chapters:
            result["incomplete"] = "chapter list is empty"
            return result, book_url

        status, value = request(opener, base, "/reader3/getBookContent",
                                {"url": book_url, "bookSource": source, "index": 0, "refresh": 1},
                                timeout=50)
        content = success(status, value, "content")
        if not isinstance(content, str):
            raise RuntimeError("content: data is not a string")
        result["steps"].append({"step": "content", "status": status,
                                "isSuccess": value["isSuccess"], "errorMsg": value.get("errorMsg"),
                                "length": len(content),
                                "sha256": hashlib.sha256(content.encode("utf-8")).hexdigest()})
        return result, book_url
    except Exception as exc:
        result["failure"] = f"{type(exc).__name__}: {exc}"
        return result, book_url
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def production_smoke(source, query, book_url):
    """Anonymous, read-only API calls; no production login or storage mutation."""
    base = "https://read.medwarp.cn"
    opener = urllib.request.build_opener()
    result = {"steps": []}
    try:
        status, value = request(opener, base, "/reader3/searchBook",
                                {"key": query, "page": 1, "bookSource": source}, timeout=40)
        books = success(status, value, "production search")
        result["steps"].append({"step": "search", "status": status,
                                "isSuccess": value["isSuccess"], "errorMsg": value.get("errorMsg"),
                                "count": len(books), "shape": shape(books[:1]),
                                "resultSha256": digest_rows([(b.get("name"), b.get("bookUrl")) for b in books])})
        if not book_url:
            result["incomplete"] = "local comparison found no book URL"
            return result
        status, value = request(opener, base, "/reader3/getBookInfo",
                                {"url": book_url, "bookSource": source}, timeout=40)
        book = success(status, value, "production book-info")
        result["steps"].append({"step": "book-info", "status": status,
                                "isSuccess": value["isSuccess"], "errorMsg": value.get("errorMsg"),
                                "hasTocUrl": bool(book.get("tocUrl")), "shape": shape(book),
                                "resultSha256": digest_rows([book.get(key) for key in
                                                             ("name", "author", "bookUrl", "tocUrl")])})
    except Exception as exc:
        result["failure"] = f"{type(exc).__name__}: {exc}"
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-namespace", required=True,
                        help="Existing production account storage namespace")
    parser.add_argument("--source-index", type=int, default=2)
    parser.add_argument("--query", default="三国演义")
    parser.add_argument("--book-result-index", type=int, default=0)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--quiet", action="store_true", help="Print only stage outcomes, not JSON shapes")
    parser.add_argument("--production-smoke", action="store_true",
                        help="Also call deployed anonymous search and book-info endpoints")
    parser.add_argument("--sse", action="store_true",
                        help="Also compare complete local SSE search streams")
    args = parser.parse_args()
    for path in (JAVA, ORIGINAL, RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    source = source_from_production(args.source_index, args.source_namespace)
    credentials = ("liveprobe" + secrets.token_hex(5), "Probe-" + secrets.token_hex(18))
    with tempfile.TemporaryDirectory(prefix="reader-live-source-") as temp:
        tempdir = Path(temp)
        original, book_url = run_jar(ORIGINAL, tempdir / "original", port(), source,
                                     args.query, credentials, book_result_index=args.book_result_index,
                                     with_sse=args.sse)
        restored, _ = run_jar(RESTORED, tempdir / "restored", port(), source,
                              args.query, credentials, book_url,
                              book_result_index=args.book_result_index, with_sse=args.sse)
    report = {
        "sourceIndex": args.source_index,
        "sourceName": source.get("bookSourceName"),
        "query": args.query,
        "bookResultIndex": args.book_result_index,
        "sse": args.sse,
        "originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
        "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
        "original": original,
        "restored": restored,
    }
    report["sameStepSummaries"] = original.get("steps") == restored.get("steps")
    accepted = []
    if len(original.get("steps", [])) == len(restored.get("steps", [])):
        for left, right in zip(original["steps"], restored["steps"]):
            if left == right:
                continue
            if (left.get("step") in ("search-sse", "source-sse") and
                    left.get("step") == right.get("step") and
                    left.get("terminalEvent") == right.get("terminalEvent") == "end" and
                    left.get("isEnd") is False and right.get("isEnd") is True and
                    left.get("lastIndex") == right.get("lastIndex") == 0 and
                    {k: v for k, v in left.items() if k != "isEnd"} ==
                    {k: v for k, v in right.items() if k != "isEnd"}):
                accepted.append(left["step"] + "-isEnd")
    report["acceptedDivergences"] = accepted
    report["parityOrAccepted"] = all(
        left == right or left.get("step") + "-isEnd" in accepted
        for left, right in zip(original.get("steps", []), restored.get("steps", []))
    ) and len(original.get("steps", [])) == len(restored.get("steps", []))
    if args.production_smoke:
        report["productionSmoke"] = production_smoke(source, args.query, book_url)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    if args.quiet:
        print(json.dumps({
            "sourceIndex": args.source_index,
            "sourceName": report["sourceName"],
            "originalSteps": [(step["step"], step.get("count")) for step in original["steps"]],
            "restoredSteps": [(step["step"], step.get("count")) for step in restored["steps"]],
            "originalFailure": original.get("failure") or original.get("incomplete"),
            "restoredFailure": restored.get("failure") or restored.get("incomplete"),
            "sameStepSummaries": report["sameStepSummaries"],
            "acceptedDivergences": accepted,
            "productionSteps": [step["step"] for step in report.get("productionSmoke", {}).get("steps", [])],
            "productionFailure": report.get("productionSmoke", {}).get("failure"),
        }, ensure_ascii=False))
    else:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    production = report.get("productionSmoke", {})
    if (original.get("failure") or restored.get("failure") or original.get("incomplete") or
            restored.get("incomplete") or not report["parityOrAccepted"] or
            production.get("failure") or production.get("incomplete")):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
