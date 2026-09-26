#!/usr/bin/env python3
"""Compare local HTTP-TTS settings and byte responses against the original JAR.

Both readers use disposable accounts/storage; the only TTS upstream is a local
HTTP fixture returning a deterministic, valid silent WAV. No external TTS is used.
"""

import base64
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import http.cookiejar
import io
import json
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import wave


ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / ".tools/jdk-11.0.8/bin/java.exe"
ORIGINAL = ROOT / "reference/original/reader-pro-3.2.14.original.jar"
RESTORED = ROOT / "build/libs/reader-4.0.7.jar"
REPORT = ROOT / "reports/tts-lifecycle-diff-latest.json"


def silent_wav():
    output = io.BytesIO()
    with wave.open(output, "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(8000)
        audio.writeframes(b"\x00\x00" * 800)
    return output.getvalue()


AUDIO = silent_wav()
AUDIO_SHA256 = hashlib.sha256(AUDIO).hexdigest()


class AudioFixture(BaseHTTPRequestHandler):
    def do_GET(self):
        if urllib.parse.urlsplit(self.path).path != "/voice.wav":
            self.send_error(404)
            return
        self.send_response(200)
        self.send_header("Content-Type", "audio/wav")
        self.send_header("Content-Length", str(len(AUDIO)))
        self.end_headers()
        self.wfile.write(AUDIO)

    def log_message(self, format, *args):
        pass


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def client():
    return urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))


def request(opener, base, method, path, payload=None):
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    headers = {} if data is None else {"Content-Type": "application/json; charset=utf-8"}
    req = urllib.request.Request(base + path, data=data, method=method, headers=headers)
    try:
        response = opener.open(req, timeout=20)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        raw = response.read()
        content_type = response.headers.get("Content-Type", "")
        if "json" in content_type:
            body = json.loads(raw.decode("utf-8"))
            for item in body.get("data", []) if isinstance(body.get("data"), list) else []:
                if isinstance(item, dict) and "lastUpdateTime" in item:
                    if not isinstance(item["lastUpdateTime"], int):
                        raise AssertionError("lastUpdateTime must be numeric")
                    item["lastUpdateTime"] = "<timestamp>"
            result = {"status": response.status, "contentType": content_type, "body": body}
        else:
            result = {"status": response.status, "contentType": content_type,
                      "length": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
            if response.status == 200 and path.startswith("/reader3/book/tts"):
                if raw != AUDIO:
                    raise AssertionError("TTS byte response differs from the local WAV fixture")
        return result


def expect(result, success, probe, count=None):
    body = result.get("body", {})
    if result["status"] != 200 or body.get("isSuccess") is not success:
        raise AssertionError(f"{probe}: unexpected status or ReturnData: {result}")
    if count is not None and len(body.get("data", [])) != count:
        raise AssertionError(f"{probe}: expected {count} rows, got {body.get('data', [])}")


def run_one(jar, workdir, username, other_user, password, voice_url):
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [str(JAVA), "-jar", str(jar), f"--reader.app.workDir={workdir}",
         f"--reader.server.port={port}", "--reader.app.secure=true",
         "--reader.app.licenseCheckEnabled=false", "--spring.profiles.active=prod"],
        cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    anonymous, owner, other = client(), client(), client()
    probes = []

    def add(name, user, method, path, payload=None):
        result = request(user, base, method, path, payload)
        probes.append({"probe": name, **result})
        return result

    try:
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"Reader exited during startup: {process.returncode}")
            try:
                if request(anonymous, base, "GET", "/reader3/getSystemInfo")["status"] == 200:
                    break
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(0.4)
        else:
            raise TimeoutError("Reader startup timeout")

        add("anonymous-list", anonymous, "GET", "/reader3/httpTTS/list")
        add("anonymous-audio", anonymous, "GET", "/reader3/book/tts?text=fixture&type=api&voice=fixture&rate=1")
        for session, name in ((owner, username), (other, other_user)):
            for login in (False, True):
                response = request(session, base, "POST", "/reader3/login",
                                   {"username": name, "password": password, "isLogin": login})
                expect(response, True, "login")

        expect(add("empty-list", owner, "GET", "/reader3/httpTTS/list"), True, "empty-list", 0)
        voice = {"id": 1700000000000, "name": "fixture", "url": voice_url,
                 "contentType": "audio/wav", "concurrentRate": "0"}
        expect(add("invalid-name", owner, "POST", "/reader3/httpTTS/save",
                   {**voice, "name": ""}), False, "invalid-name")
        expect(add("invalid-url", owner, "POST", "/reader3/httpTTS/save",
                   {**voice, "url": ""}), False, "invalid-url")
        expect(add("save", owner, "POST", "/reader3/httpTTS/save", voice), True, "save")
        expect(add("one", owner, "GET", "/reader3/httpTTS/list"), True, "one", 1)
        expect(add("other-user-empty", other, "GET", "/reader3/httpTTS/list"), True,
               "other-user-empty", 0)

        missing = add("audio-missing-text", owner, "GET", "/reader3/book/tts?type=api&voice=fixture&rate=1")
        if missing["status"] != 404:
            raise AssertionError("missing TTS text must return 404")
        unknown = add("audio-unknown-voice", owner, "GET",
                      "/reader3/book/tts?text=fixture&type=api&voice=unknown&rate=1")
        if unknown["status"] != 404:
            raise AssertionError("unknown TTS voice must return 404")
        audio_path = "/reader3/book/tts?text=fixture&type=api&voice=fixture&rate=1"
        for label, method, path, body in (
            ("audio-get", "GET", audio_path, None),
            ("audio-post", "POST", "/reader3/book/tts",
             {"text": "fixture", "type": "api", "voice": "fixture", "rate": "1"}),
        ):
            result = add(label, owner, method, path, body)
            if result["status"] != 200 or result["sha256"] != AUDIO_SHA256:
                raise AssertionError(f"{label}: fixture WAV was not returned")

        encoded = add("audio-base64", owner, "GET", audio_path + "&base64=1")
        expect(encoded, True, "audio-base64")
        if base64.b64decode(encoded["body"]["data"], validate=True) != AUDIO:
            raise AssertionError("base64 TTS data differs from fixture WAV")
        if add("other-user-audio", other, "GET", audio_path)["status"] != 404:
            raise AssertionError("another user accessed the owner's TTS configuration")

        second = {**voice, "id": 1700000000001, "name": "fixture-second"}
        expect(add("save-multi", owner, "POST", "/reader3/httpTTS/saveMulti", [voice, second]),
               True, "save-multi")
        expect(add("after-save-multi", owner, "GET", "/reader3/httpTTS/list"),
               True, "after-save-multi")
        expect(add("delete-one", owner, "POST", "/reader3/httpTTS/delete", voice), True,
               "delete-one")
        expect(add("after-delete", owner, "GET", "/reader3/httpTTS/list"), True,
               "after-delete")
        expect(add("delete-multi", owner, "POST", "/reader3/httpTTS/deleteMulti",
                   [second]), True, "delete-multi")
        expect(add("after-delete-multi", owner, "GET", "/reader3/httpTTS/list"),
               True, "after-delete-multi", 0)

        storage_path = workdir / "storage/data" / username / "httpTTS.json"
        storage = json.loads(storage_path.read_text(encoding="utf-8"))
        for item in storage:
            if "lastUpdateTime" in item:
                item["lastUpdateTime"] = "<timestamp>"
        return {"probes": probes, "storage": storage}
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def main():
    for path in (JAVA, ORIGINAL, RESTORED):
        if not path.is_file():
            raise FileNotFoundError(path)
    fixture = ThreadingHTTPServer(("127.0.0.1", free_port()), AudioFixture)
    thread = threading.Thread(target=fixture.serve_forever, daemon=True)
    thread.start()
    username = "ttsprobe" + secrets.token_hex(5)
    other_user = "otherprobe" + secrets.token_hex(5)
    password = "Probe-" + secrets.token_hex(18)
    voice_url = f"http://127.0.0.1:{fixture.server_port}/voice.wav"
    try:
        with tempfile.TemporaryDirectory(prefix="reader-tts-diff-") as temp:
            root = Path(temp)
            original = run_one(ORIGINAL, root / "original", username, other_user,
                               password, voice_url)
            restored = run_one(RESTORED, root / "restored", username, other_user,
                               password, voice_url)
    finally:
        fixture.shutdown()
        fixture.server_close()
    comparisons = []
    for left, right in zip(original["probes"], restored["probes"]):
        accepted = False
        if left["probe"] == right["probe"] == "after-save-multi" and left != right:
            old_rows = left.get("body", {}).get("data", [])
            new_rows = right.get("body", {}).get("data", [])
            accepted = (left["status"] == right["status"] == 200
                        and left["contentType"] == right["contentType"]
                        and left["body"]["isSuccess"] is right["body"]["isSuccess"] is True
                        and left["body"]["errorMsg"] == right["body"]["errorMsg"] == ""
                        and len(old_rows) == 1 and len(new_rows) == 2
                        and [row["name"] for row in old_rows] == ["fixture-second"]
                        and [row["name"] for row in new_rows] == ["fixture", "fixture-second"]
                        and old_rows[0] == new_rows[1])
        comparisons.append({"probe": left["probe"], "equal": left == right,
                            "acceptedDivergence": accepted,
                            "original": left, "restored": right})
    report = {"originalJarSha256": hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),
              "restoredJarSha256": hashlib.sha256(RESTORED.read_bytes()).hexdigest(),
              "comparisons": comparisons, "storageEqual": original["storage"] == restored["storage"],
              "originalStorage": original["storage"], "restoredStorage": restored["storage"]}
    REPORT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in comparisons:
        outcome = "equal" if row["equal"] else "accepted old-JAR data loss" if row["acceptedDivergence"] else "DIFFERENT"
        print(f"{row['probe']}: {outcome}")
    print(f"Storage: {'equal' if report['storageEqual'] else 'DIFFERENT'}")
    print(f"Report: {REPORT}")
    if not all(row["equal"] or row["acceptedDivergence"] for row in comparisons) or not report["storageEqual"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
