"""Bounded credential-free CONNECT bridge: only one public host, no host TCP listener."""
import ipaddress
import json
import os
from pathlib import Path
import select
import socket
import socketserver
import ssl
import threading
import time

HOST = 'www.gutenberg.org'
LIMIT = 4 * 1024 * 1024


def allowed_connect(line):
    return line in (b'CONNECT www.gutenberg.org:443 HTTP/1.0',
                    b'CONNECT www.gutenberg.org:443 HTTP/1.1')


def public_addresses():
    addresses = sorted({row[4][0] for row in socket.getaddrinfo(HOST, 443, type=socket.SOCK_STREAM)})
    if not addresses or any(not ipaddress.ip_address(value).is_global for value in addresses):
        raise ValueError('Only verified public DNS answers are permitted')
    ipv4 = [value for value in addresses if ipaddress.ip_address(value).version == 4]
    if not ipv4:
        raise ValueError('Public IPv4 address required for this bounded run')
    return ipv4


def client_hello_host(raw):
    """Require a complete TLS ClientHello with exactly one expected DNS SNI."""
    if len(raw) < 9 or raw[0] != 22 or raw[1] != 3 or len(raw) != 5 + int.from_bytes(raw[3:5], 'big'):
        raise ValueError('Bounded TLS handshake record required')
    body = raw[5:]
    if body[0] != 1 or int.from_bytes(body[1:4], 'big') != len(body) - 4:
        raise ValueError('Complete ClientHello required')
    data, offset = body[4:], 34
    offset += 1 + data[offset]
    offset += 2 + int.from_bytes(data[offset:offset + 2], 'big')
    offset += 1 + data[offset]
    length = int.from_bytes(data[offset:offset + 2], 'big')
    offset += 2
    if offset + length != len(data):
        raise ValueError('ClientHello extension size differs')
    found = []
    end = offset + length
    while offset < end:
        kind, size = int.from_bytes(data[offset:offset + 2], 'big'), int.from_bytes(data[offset + 2:offset + 4], 'big')
        offset += 4
        value = data[offset:offset + size]
        offset += size
        if offset > end:
            raise ValueError('Incomplete TLS extension')
        if kind == 0:
            if len(value) < 5 or int.from_bytes(value[:2], 'big') != len(value) - 2 or value[2] != 0 or int.from_bytes(value[3:5], 'big') != len(value) - 5:
                raise ValueError('Exactly one DNS SNI required')
            found.append(value[5:].decode('ascii'))
    if found != [HOST]:
        raise ValueError('TLS SNI is outside the authorized host')
    return HOST


def exact_read(connection, size):
    raw = b''
    while len(raw) < size:
        data = connection.recv(size - len(raw))
        if not data:
            raise ValueError('Truncated TLS ClientHello')
        raw += data
    return raw


def pump(left, right, row):
    deadline = time.monotonic() + 40
    while time.monotonic() < deadline:
        ready, _, _ = select.select([left, right], [], [], 0.2)
        for source in ready:
            data = source.recv(16384)
            if not data:
                return
            key = 'clientToServerBytes' if source is left else 'serverToClientBytes'
            row[key] += len(data)
            if row[key] > LIMIT:
                raise ValueError('Tunnel byte bound exceeded')
            (right if source is left else left).sendall(data)
    raise TimeoutError('Tunnel lifetime exceeded')


class Broker:
    def __init__(self, directory):
        self.directory = Path(directory)
        self.socket_path = self.directory / 'egress.sock'
        if self.socket_path.exists():
            raise ValueError('Preserve existing socket')
        self.addresses = public_addresses()
        self.rows, self.denied, self.failures = [], 0, []
        self.lock = threading.Lock()
        self.slots = threading.BoundedSemaphore(8)
        owner = self

        class Handler(socketserver.StreamRequestHandler):
            def handle(self):
                if not owner.slots.acquire(blocking=False):
                    self.wfile.write(b'HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\nConnection: close\r\n\r\n')
                    return
                try:
                    self.connection.settimeout(10)
                    line = self.rfile.readline(2049).rstrip(b'\r\n')
                    total, headers = len(line), []
                    while True:
                        header = self.rfile.readline(2049)
                        total += len(header)
                        if total > 8192 or not header:
                            raise ValueError('Malformed or oversized proxy request')
                        if header == b'\r\n':
                            break
                        headers.append(header)
                    forbidden = any(row.lower().startswith((b'authorization:', b'proxy-authorization:', b'cookie:')) for row in headers)
                    with owner.lock:
                        allowed = allowed_connect(line) and not forbidden and len(owner.rows) < 96
                        if not allowed:
                            owner.denied += 1
                    if not allowed:
                        self.wfile.write(b'HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n')
                        return
                    row = dict(host=HOST, port=443, method='CONNECT', clientToServerBytes=0, serverToClientBytes=0)
                    with owner.lock:
                        owner.rows.append(row)
                    with socket.create_connection((owner.addresses[0], 443), timeout=10) as outgoing:
                        outgoing.settimeout(10)
                        self.wfile.write(b'HTTP/1.1 200 Connection Established\r\n\r\n')
                        self.wfile.flush()
                        pump(self.connection, outgoing, row)
                except (ConnectionError, OSError, TimeoutError) as failure:
                    # Browsers cancel subresources normally; retain type only.
                    with owner.lock:
                        owner.failures.append(type(failure).__name__)
                except Exception as failure:
                    with owner.lock:
                        owner.failures.append(type(failure).__name__)
                finally:
                    owner.slots.release()

        class Server(socketserver.ThreadingMixIn, socketserver.UnixStreamServer):
            daemon_threads = False
            block_on_close = True

        self.server = Server(str(self.socket_path), Handler)
        os.chown(self.socket_path, 0, 10001)
        os.chmod(self.socket_path, 0o660)
        self.thread = threading.Thread(target=self.server.serve_forever)
        self.thread.start()

    def facts(self):
        with self.lock:
            return dict(allowedHost=HOST, allowedPort=443, upstreamPinnedPublicAddresses=self.addresses,
                        hostTcpListenerCreated=False, deniedRequests=self.denied,
                        tunnels=[dict(row) for row in self.rows], transportFailureTypes=list(self.failures))

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=3)
        if self.thread.is_alive():
            raise RuntimeError('Owned broker thread did not stop')
        self.socket_path.unlink()


class Relay:
    def __init__(self, socket_path, public_ip):
        owner = self
        owner.socket_path = str(socket_path)

        class Handler(socketserver.BaseRequestHandler):
            def handle(self):
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as outgoing:
                    outgoing.settimeout(10)
                    outgoing.connect(owner.socket_path)
                    try:
                        pump(self.request, outgoing, dict(clientToServerBytes=0, serverToClientBytes=0))
                    except (ConnectionError, OSError, ValueError, TimeoutError):
                        pass

        class Server(socketserver.ThreadingMixIn, socketserver.TCPServer):
            daemon_threads = True

        self.server = Server(('127.0.0.1', 0), Handler)
        self.port = self.server.server_address[1]
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

        class TransparentHandler(socketserver.BaseRequestHandler):
            def handle(self):
                try:
                    self.request.settimeout(10)
                    prefix = exact_read(self.request, 5)
                    size = int.from_bytes(prefix[3:5], 'big')
                    if not 64 <= size <= 16384:
                        raise ValueError('Oversized or missing TLS handshake')
                    hello = prefix + exact_read(self.request, size)
                    client_hello_host(hello)
                    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as outgoing:
                        outgoing.settimeout(10)
                        outgoing.connect(owner.socket_path)
                        outgoing.sendall(b'CONNECT www.gutenberg.org:443 HTTP/1.1\r\nHost: www.gutenberg.org:443\r\n\r\n')
                        head = b''
                        while not head.endswith(b'\r\n\r\n') and len(head) <= 8192:
                            head += exact_read(outgoing, 1)
                        if not head.startswith(b'HTTP/1.1 200 ') or len(head) > 8192:
                            raise ValueError('Restricted proxy rejected transparent bridge')
                        outgoing.sendall(hello)
                        pump(self.request, outgoing, dict(clientToServerBytes=len(hello), serverToClientBytes=0))
                except (ConnectionError, OSError, ValueError, TimeoutError, IndexError, UnicodeError):
                    pass

        if not ipaddress.ip_address(public_ip).is_global:
            raise ValueError('Transparent listener must be the host-verified public address in private netns')
        self.transparent_server = Server((public_ip, 443), TransparentHandler)
        self.transparent_thread = threading.Thread(target=self.transparent_server.serve_forever, daemon=True)
        self.transparent_thread.start()

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)
        self.transparent_server.shutdown()
        self.transparent_server.server_close()
        self.transparent_thread.join(timeout=2)

    def negative_checks(self):
        checks = {}
        for line in ('CONNECT example.com:443 HTTP/1.1', 'CONNECT 127.0.0.1:443 HTTP/1.1',
                     'CONNECT www.gutenberg.org:80 HTTP/1.1', 'GET http://www.gutenberg.org/ HTTP/1.1'):
            with socket.create_connection(('127.0.0.1', self.port), timeout=5) as connection:
                connection.sendall(line.encode() + b'\r\nHost: www.gutenberg.org\r\n\r\n')
                reply = connection.recv(4096)
                checks[line.split(' ', 2)[0] + '-' + line.split(' ', 2)[1]] = reply.startswith(b'HTTP/1.1 403 ')
        if not all(checks.values()):
            raise ValueError('Whitelist negative check failed before Reader launch')
        return checks
