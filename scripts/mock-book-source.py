"""Deterministic loopback-only fixture for remote book-source differential tests."""

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
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


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlsplit(self.path)
        if parsed.path == "/health":
            body = "ready"
        elif parsed.path == "/search":
            key = parse_qs(parsed.query).get("key", [""])[0]
            body = (
                "<html><div class='book'><a href='/book'>"
                "<span class='name'>差分测试书</span></a>"
                "<span class='author'>测试作者</span></div></html>"
                if key == "差分" else "<html></html>"
            )
        else:
            body = PAGES.get(parsed.path)
        if body is None:
            self.send_error(404)
            return
        data = body.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
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
