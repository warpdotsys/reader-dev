"""Generated HTTPS regression against the actual worker bytes packaged in a JAR.

Only run in the dedicated offline, read-only native-image container. The test
adds a generated CA to a private distribution tmpfs, never the host trust store.
It exercises the production worker, not the Java API/production proxy chain.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import select
import shutil
import socket
import ssl
import subprocess
import sys
import tempfile
import threading
import time
import zipfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

EXTRA_CASES = ("same-post-307", "same-post-308", "cross-post-307",
               "cross-post-308", "script-cross-get", "resources-redirect")


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require(value, reason):
    if not value:
        raise RuntimeError(reason)


def hash_file(path):
    # The locked product image uses Python 3.10; file_digest is 3.11+ only.
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def identity_and_budget(resources):
    fields = dict(line.split(":", 1) for line in
                  Path("/proc/self/status").read_text().splitlines() if ":" in line)
    identity = {
        "uid": os.getuid(), "gid": os.getgid(), "groups": os.getgroups(),
        "interfaces": sorted(name for _, name in socket.if_nameindex()),
        "capsZero": all(int(fields[key].strip(), 16) == 0 for key in
                        ("CapEff", "CapPrm", "CapBnd", "CapInh", "CapAmb")),
        "noNewPrivileges": fields["NoNewPrivs"].strip() == "1",
        "namespaces": {key: os.stat("/proc/self/ns/" + key).st_ino
                       for key in ("user", "net", "mnt", "pid", "ipc", "uts")},
    }
    require(identity["uid"] == identity["gid"] == 10001 and
            set(identity["groups"]) <= {10001} and identity["interfaces"] == ["lo"] and
            identity["capsZero"] and identity["noNewPrivileges"],
            "TLS regression requires an offline unprivileged container")
    resources.verify_report(resources.collect_report(), require_no_swap=True)
    return identity


def probe(arguments):
    for name in ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS",
                 "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS"):
        os.environ[name] = "1"
    script_dir = Path(__file__).resolve().parent
    resources = load_module("tls_resource_observer", script_dir / "report-browser-cgroup.py")
    identity = identity_and_budget(resources)
    c = load_module("tls_bounded_headers", script_dir / "compare-webview-cookie.py")
    h = load_module("tls_header_fixture", script_dir / "reader_header_security_differential.py")
    jar_path = Path(arguments.jar)
    require(jar_path.is_file() and not jar_path.is_symlink(), "Expected a regular JAR")
    jar_sha = hash_file(jar_path)
    require(jar_sha == arguments.expected_jar_sha, "Packaged JAR differs from build output")
    with zipfile.ZipFile(jar_path) as archive:
        entries = [item for item in archive.infolist()
                   if item.filename == "BOOT-INF/classes/camoufox/worker.py"]
        require(len(entries) == 1 and 0 < entries[0].file_size <= 262144,
                "Expected exactly one bounded packaged worker")
        worker_source = archive.read(entries[0])
    worker_sha = hashlib.sha256(worker_source).hexdigest()
    require(worker_sha == arguments.expected_worker_sha,
            "Packaged worker differs from readable production source")
    from camoufox.multiversion import get_active_path
    active = Path(get_active_path()).resolve()
    distribution = Path(arguments.distribution).resolve()
    require(active.is_relative_to(Path("/home/reader/.cache/camoufox/browsers/official")) and
            distribution == active / "distribution" and os.path.ismount(distribution),
            "Certificate fixture must use the exact private distribution mount")
    mounts = [line.split() for line in Path("/proc/self/mountinfo").read_text().splitlines()]
    require(any(parts[4] == str(distribution) and parts[parts.index("-") + 1] == "tmpfs"
                for parts in mounts), "Certificate fixture is not a private tmpfs")
    require(any(parts[4] == "/home/reader/.cache/camoufox/fontconfig" and
                parts[parts.index("-") + 1] == "tmpfs" for parts in mounts),
            "Runtime fontconfig must use a private tmpfs, never modify the immutable browser")
    policy = distribution / "policies.json"
    require(not policy.exists() and not policy.is_symlink(), "Never overwrite an existing policy")
    seed = json.loads(Path(arguments.seed_policy).read_bytes())["policy"]
    require(isinstance(seed, dict) and isinstance(seed.get("policies"), dict) and
            "Certificates" not in seed["policies"], "Unexpected base browser policy")
    with tempfile.TemporaryDirectory(prefix="reader-packaged-tls-worker-") as worker_directory:
        worker_path = Path(worker_directory) / "worker.py"
        worker_path.write_bytes(worker_source)
        worker_path.chmod(0o400)
        original_stdout = sys.stdout
        try:
            worker = load_module("actual_packaged_tls_worker", worker_path)
        finally:
            sys.stdout = original_stdout
        return run_cases(arguments, identity, jar_sha, worker_sha, worker, policy, seed, h, c, resources)


def run_cases(arguments, identity, jar_sha, worker_sha, worker, policy, seed, h, c, resources):
    extra_cases=EXTRA_CASES
    h.CASES=tuple(h.CASES)+extra_cases
    tunnel_lock=threading.RLock()
    tunnels=[]
    proxy_outgoing_ports=set()
    results=[]
    servers=[]
    threads=[]

    class RedirectHandler(h.HeaderHandler):
        def handle_probe(self):
            self.server.fixture.connection_state.source_port = self.client_address[1]
            self.server.fixture.target_connections.append(self.client_address[1])
            parts=self.path.split('/')
            if len(parts)!=4 or parts[2] not in extra_cases:
                return super().handle_probe()
            case,role=parts[2:]
            fixture,actor=self.server.fixture,self.server.actor
            lengths=self.headers.get_all('Content-Length',[])
            assert not self.headers.get('Transfer-Encoding') and len(lengths)<=1
            length=int(lengths[0]) if lengths else 0
            assert 0<=length<=65536
            self.connection.settimeout(5)
            body=self.rfile.read(length)
            assert len(body)==length
            fixture.record(case,actor,role,self.command,self.headers,body)
            if role=='start' and case.endswith(('307','308')):
                assert actor=='primary' and self.command=='POST'
                endpoint=fixture.primary if case.startswith('same-') else fixture.secondary
                self.send_response(int(case[-3:]))
                self.send_header('Location',endpoint+'/probe/'+case+'/end')
                self.send_header('Set-Cookie','generated_session=GENERATED_SESSION_ONLY; Path=/probe/; HttpOnly; Secure; SameSite=Lax')
                self.send_header('Content-Length','0')
                self.end_headers()
            elif case=='script-cross-get' and role=='start':
                self.send(200,('<!doctype html><meta charset="UTF-8"><script>window.location.href='+json.dumps(fixture.secondary+'/probe/'+case+'/end')+';</script>').encode())
            elif case=='resources-redirect':
                if role=='start':
                    self.send(200,fixture.html(case)+('<script src="'+fixture.primary+'/probe/'+case+'/same.js"></script><script src="'+fixture.primary+'/probe/'+case+'/cross.js"></script>').encode())
                elif role=='same.js':self.send(307,location=fixture.primary+'/probe/'+case+'/same-end.js')
                elif role=='cross.js':self.send(303,location=fixture.secondary+'/probe/'+case+'/cross-end.js')
                elif role=='same-end.js':self.send(200,b"document.documentElement.dataset.generatedSameResource='executed';",'application/javascript')
                elif role=='cross-end.js':self.send(200,b"document.documentElement.dataset.generatedCrossResource='executed';",'application/javascript')
                else:self.send(404)
            else:
                assert role=='end'
                self.send(200,fixture.html(case))

    class NegativeHandler(BaseHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_GET(self):
            # A wrong-host/untrusted cert must fail BEFORE sending any HTTP fields.
            self.server.actual_http_count+=1
            self.send_response(500)
            self.send_header('Content-Length','0')
            self.end_headers()

    class Proxy(BaseHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_CONNECT(self):
            try:host,port=self.path.rsplit(':',1);endpoint=(host,int(port))
            except ValueError:self.send_error(403);return
            allowed={server.server_address[:2] for server in servers}
            if endpoint not in allowed:self.send_error(403);return
            with tunnel_lock:
                assert len(tunnels)<32
                row={'method':'CONNECT','host':host,'port':int(port),'clientToServerBytes':0,'serverToClientBytes':0}
                tunnels.append(row)
            outgoing=socket.create_connection(endpoint,timeout=5)
            with tunnel_lock:
                row['targetSourcePort'] = outgoing.getsockname()[1]
                proxy_outgoing_ports.add(row['targetSourcePort'])
            self.send_response(200);self.end_headers()
            try:
                deadline=time.monotonic()+12
                while time.monotonic()<deadline:
                    ready,_,_=select.select([self.connection,outgoing],[],[],.1)
                    for source in ready:
                        data=source.recv(16384)
                        if not data:return
                        key='clientToServerBytes' if source is self.connection else 'serverToClientBytes'
                        row[key]+=len(data)
                        assert row[key]<=4*1024*1024
                        (outgoing if source is self.connection else self.connection).sendall(data)
            except (ConnectionError,TimeoutError):pass
            finally:outgoing.close()

    class TlsFixture(h.HeaderFixture):
        def record(self, case, actor, role, method, headers, body):
            with self.lock:
                super().record(case, actor, role, method, headers, body)
                self.rows[-1]['targetSourcePort'] = self.connection_state.source_port

    fixture=TlsFixture.__new__(TlsFixture)
    fixture.connection_state=threading.local()
    fixture.lock=threading.RLock();fixture.rows=[];fixture.rejections=0
    fixture.bounded_headers=c.bounded_target_headers
    fixture.servers=[];fixture.threads=[];fixture.target_connections=[]
    try:
        with tempfile.TemporaryDirectory(prefix='reader-generated-tls-suite-') as temporary:
            directory=Path(temporary)
            def openssl(*args):
                result=subprocess.run(['/usr/bin/openssl',*args],cwd=directory,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,timeout=10)
                assert result.returncode==0 and len(result.stderr)<=8192
            openssl('req','-x509','-newkey','rsa:2048','-nodes','-keyout','ca.key','-out','ca.pem',
                '-days','2','-subj','/CN=Reader Generated Isolation CA','-addext','basicConstraints=critical,CA:TRUE','-addext','keyUsage=critical,keyCertSign,cRLSign')
            for name,san in (('valid','IP:127.0.0.1,IP:127.0.0.2'),('wrong-host','DNS:wrong.generated.invalid')):
                openssl('req','-new','-newkey','rsa:2048','-nodes','-keyout',name+'.key','-out',name+'.csr','-subj','/CN=Reader Generated TLS')
                extension=directory/(name+'.ext')
                extension.write_text('basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName='+san+'\n')
                openssl('x509','-req','-in',name+'.csr','-CA','ca.pem','-CAkey','ca.key','-CAcreateserial','-out',name+'.pem','-days','2','-extfile',str(extension))
            # A valid second CA is deliberately NOT installed. This distinguishes
            # an untrusted issuer from a malformed self-signed leaf/CA constraint.
            openssl('req','-x509','-newkey','rsa:2048','-nodes','-keyout','untrusted-ca.key','-out','untrusted-ca.pem','-days','2',
                '-subj','/CN=Reader Generated Untrusted CA','-addext','basicConstraints=critical,CA:TRUE','-addext','keyUsage=critical,keyCertSign,cRLSign')
            openssl('req','-new','-newkey','rsa:2048','-nodes','-keyout','untrusted.key','-out','untrusted.csr','-subj','/CN=Reader Generated Untrusted Leaf')
            openssl('x509','-req','-in','untrusted.csr','-CA','untrusted-ca.pem','-CAkey','untrusted-ca.key','-CAcreateserial',
                '-out','untrusted.pem','-days','2','-extfile',str(directory/'valid.ext'))
            config = json.loads(json.dumps(seed))
            config['policies']['Certificates'] = {'Install': [str(directory/'ca.pem')]}
            with policy.open('x', encoding='utf-8') as stream:
                json.dump(config, stream)
            policy.chmod(0o600)
            for host,actor in (('127.0.0.1','primary'),('127.0.0.2','secondary'),('127.0.0.1','wrong-host'),('127.0.0.1','untrusted')):
                certificate='valid' if actor in ('primary','secondary') else actor
                server=ThreadingHTTPServer((host,0),RedirectHandler if certificate=='valid' else NegativeHandler)
                server.actor=actor;server.fixture=fixture;server.actual_http_count=0
                context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
                context.load_cert_chain(str(directory/(certificate+'.pem')),str(directory/(certificate+'.key')))
                server.socket=context.wrap_socket(server.socket,server_side=True)
                servers.append(server)
                thread=threading.Thread(target=server.serve_forever,daemon=True)
                thread.start();threads.append(thread)
            fixture.primary='https://127.0.0.1:'+str(servers[0].server_port)
            fixture.secondary='https://127.0.0.2:'+str(servers[1].server_port)
            proxy=ThreadingHTTPServer(('127.0.0.1',0),Proxy)
            proxy_thread=threading.Thread(target=proxy.serve_forever,daemon=True)
            proxy_thread.start()
            try:
                for case in (*h.CASES,'wrong-host','untrusted'):
                    fixture.reset();fixture.target_connections.clear()
                    with tunnel_lock:tunnels.clear();proxy_outgoing_ports.clear()
                    negative=case in ('wrong-host','untrusted')
                    result=None;failure=None;codes=[];os_error=None
                    if negative:
                        server=servers[2 if case=='wrong-host' else 3]
                        url='https://127.0.0.1:'+str(server.server_port)+'/generated-negative'
                        options={'webJs':None,'headers':{}}
                    else:
                        definition=fixture.definition(case)
                        url,raw_options=definition['searchUrl'].split(', ',1)
                        options=json.loads(raw_options)
                        if case=='resources-redirect':
                            options['webJs']="(() => { if(document.documentElement.dataset.generatedSameResource !== 'executed' || document.documentElement.dataset.generatedCrossResource !== 'executed') throw new Error('GENERATED_RESOURCE_NOT_EXECUTED'); return document.documentElement.outerHTML.replace('认证头原始书-resources-redirect','认证头差分书-resources-redirect'); })()"
                    is_post=case=='cross-post' or case.endswith(('307','308'))
                    raw_body='q=生成中文&value=保持原始字节' if case.endswith(('307','308')) else h.POST_BODY
                    rule_headers={name:value for name,value in options['headers'].items() if name.lower()!='user-agent'}
                    if case.endswith(('307','308')):rule_headers['Content-Type']='application/json; charset=UTF-8'
                    payload={'url':url,'headers':rule_headers,'userAgent':h.USER_AGENT,'timeoutMs':8000,'browserVersion':os.environ['READER_APP_CAMOUFOXBROWSERVERSION'],
                        'proxy':'http://127.0.0.1:'+str(proxy.server_port),'javaScript':options['webJs'],'post':is_post,'body':raw_body if is_post else None}
                    try:result=worker.render(payload)
                    except Exception as error:
                        failure=type(error).__name__
                        if isinstance(error, OSError):
                            # Only generated diagnostic categories, not arbitrary paths/URLs.
                            filename = str(error.filename or "")
                            category = ("fontconfig" if filename.startswith("/home/reader/.cache/camoufox/fontconfig")
                                        else "profile" if filename.startswith("/home/reader/.camoufox")
                                        else "temporary" if filename.startswith("/tmp/") else "other")
                            os_error = {"errno": error.errno, "pathCategory": category}
                        codes=re.findall(r'\b(?:SSL_ERROR_|SEC_ERROR_|MOZILLA_PKIX_ERROR_)[A-Z0-9_]{1,80}\b',str(error))
                        assert len(codes)<=16
                    rows=fixture.snapshot()
                    with tunnel_lock:observed_tunnels=json.loads(json.dumps(tunnels))
                    results.append({'case':case,'failureType':failure,'osErrorDiagnostic':os_error,'tlsErrorCodes':sorted(set(codes)),'workerResult':result,
                        'everyActualTargetConnectionUsedProxy':bool(fixture.target_connections) and all(port in proxy_outgoing_ports for port in fixture.target_connections),
                        'observedTargetSourcePorts':list(fixture.target_connections),'negativeHttpRequests':server.actual_http_count if negative else None,
                        'tunnels':observed_tunnels,**rows})
                    print(json.dumps({'partialCaseCompleted':case,'failureType':failure,'targetCount':len(rows['targetRequests'])}),file=sys.stderr,flush=True)
            finally:
                proxy.shutdown();proxy.server_close();proxy_thread.join(timeout=5)
                assert not proxy_thread.is_alive()
    finally:
        if policy.exists(): policy.unlink()
        for server in servers:server.shutdown();server.server_close()
        for thread in threads:thread.join(timeout=5);assert not thread.is_alive()
    budget = resources.collect_report("packaged worker generated HTTPS; no Reader API or real accounts")
    resources.verify_report(budget, require_no_swap=True)
    return {
        'schemaVersion': 1, 'scope': 'packaged worker HTTPS subset; not Java CONNECT/API or production acceptance',
        'architecture': arguments.architecture, 'revision': arguments.revision,
        'jarSha256': jar_sha, 'workerSha256': worker_sha, 'identity': identity, 'resources': budget,
        'generatedOnly': True, 'realCredentialsImported': False, 'privateBookBodyRead': False,
        'readerJarStarted': False, 'hostTrustStoreChanged': False, 'ignoreHttpsErrorsUsed': False,
        'fixtureOnlyPrivateDistributionPolicy': True, 'workerLaunchOverridden': False,
        'fixtureOnlyPrivateFontconfigTmpfs': True,
        'httpsTested': True, 'fullGoalComplete': False, 'results': results,
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True)
    parser.add_argument("--expected-jar-sha", required=True)
    parser.add_argument("--expected-worker-sha", required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--architecture", choices=("amd64", "arm64"), required=True)
    parser.add_argument("--distribution", required=True)
    parser.add_argument("--seed-policy", required=True)
    arguments = parser.parse_args(argv)
    require(re.fullmatch(r"[0-9a-f]{64}", arguments.expected_jar_sha) and
            re.fullmatch(r"[0-9a-f]{64}", arguments.expected_worker_sha) and
            re.fullmatch(r"[0-9a-f]{40}", arguments.revision), "Invalid build identity")
    result = probe(arguments)
    json.dump(result, sys.stdout, ensure_ascii=False, separators=(",", ":"))
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
