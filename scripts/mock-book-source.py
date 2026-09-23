"""Deterministic loopback-only fixture for remote book-source differential tests."""

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from threading import Lock
import time
from urllib.parse import parse_qs, urlsplit


PAGES = {
    "/book": "<html><h1>差分测试书</h1><span class='author'>测试作者</span>"
    "<a class='toc' href='/toc'>章节目录</a></html>",
    "/toc": "<html><div class='chapter'><a href='/chapter/1'>第一章 起点</a></div>"
    "<div class='chapter'><a href='/chapter/2'>第二章 继续</a></div></html>",
    "/chapter/1": "<html><div class='content'><p>第一段，中文与 UTF-8。</p>"
    "<p>第二段，符号 &amp; 空格。</p></div></html>",
    "/chapter/2": "<html><div class='content'><p>终章内容固定。</p></div></html>",
}

_search_lock = Lock()
_active_searches = 0
_max_active_searches = 0


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        global _active_searches, _max_active_searches
        parsed = urlsplit(self.path)
        if parsed.path == "/health":
            body = "ready"
        elif parsed.path == "/source.json":
            base = f"http://127.0.0.1:{self.server.server_port}"
            body = json.dumps([{
                "bookSourceUrl": base,
                "bookSourceName": "Remote import fixture",
                "searchUrl": f"{base}/search?key={{{{key}}}}",
            }])
        elif parsed.path == "/stats":
            with _search_lock:
                if parse_qs(parsed.query).get("reset") == ["1"]:
                    _max_active_searches = _active_searches
                body = json.dumps({"activeSearches": _active_searches,
                                   "maxActiveSearches": _max_active_searches})
        elif parsed.path == "/search":
            with _search_lock:
                _active_searches += 1
                _max_active_searches = max(_max_active_searches, _active_searches)
            try:
                params = parse_qs(parsed.query)
                delay_ms = min(max(int(params.get("delayMs", ["0"])[0]), 0), 1000)
                if delay_ms:
                    time.sleep(delay_ms / 1000)
                key = params.get("key", [""])[0]
                body = (
                    "<html><div class='book'><a href='/book'>"
                    "<span class='name'>差分测试书</span></a>"
                    "<span class='author'>测试作者</span></div></html>"
                    if key in ("差分", "差分测试书") else "<html></html>"
                )
            finally:
                with _search_lock:
                    _active_searches -= 1
        else:
            body = PAGES.get(parsed.path)
        if body is None:
            self.send_error(404)
            return
        data = body.encode("utf-8")
        self.send_response(200)
        content_type = "application/json" if parsed.path == "/source.json" else "text/html"
        self.send_header("Content-Type", f"{content_type}; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, format, *args):
        pass


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, required=True)
    args = parser.parse_args()
    ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()
