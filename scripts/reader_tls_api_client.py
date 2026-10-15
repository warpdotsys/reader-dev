"""Actual generated Reader API calls. No worker substitute or real credentials."""
import hashlib
import http.cookiejar
import ipaddress
import json
import os
from pathlib import Path
import re
import secrets
import socket
import subprocess
import sys
import time
import urllib.request
import zipfile

if sys.flags.optimize:
    raise RuntimeError('Never disable the generated API fixture assertions')

def hash_file(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1048576), b''):
            digest.update(block)
    return digest.hexdigest()

def address(raw):
    host, port = raw.split(':')
    if len(host) == 8:
        value = ipaddress.ip_address(bytes.fromhex(host)[::-1])
    else:
        value = ipaddress.ip_address(b''.join(bytes.fromhex(host[i:i+8])[::-1] for i in range(0,32,8)))
        value = value.ipv4_mapped or value
    return str(value), int(port,16)

class ReaderApi:
    def __init__(self, helper, jar_path, jar_sha, worker_sha, browser_version):
        assert re.fullmatch('[0-9a-f]{64}', jar_sha) and re.fullmatch('[0-9a-f]{64}', worker_sha)
        assert re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+(?:-[A-Za-z0-9.]+)?', browser_version)
        self.helper = helper
        self.jar_path = Path(jar_path)
        self.jar_sha = jar_sha
        self.worker_sha = worker_sha
        self.browser_version = browser_version
        self.process = None
        self.openers = {}
        self.auth = []
        self.stopped = False
        self.pid_start = None
        self.log = None
        self.started = False

    def start(self):
        assert os.getuid() == os.getgid() == 10001 and set(os.getgroups()) <= {10001}
        assert sorted(name for _, name in socket.if_nameindex()) == ['lo']
        mounts = [line.split() for line in Path('/proc/self/mountinfo').read_text().splitlines()]
        assert any(row[4]=='/storage' and row[row.index('-')+1]=='tmpfs' and 'rw' in row[5].split(',') for row in mounts)
        assert self.process is None, 'Never start a second JVM in one fixture'
        jar = self.jar_path
        assert jar.is_file() and not jar.is_symlink() and 0 < jar.stat().st_size <= 384*1024**2
        assert hash_file(jar) == self.jar_sha
        with zipfile.ZipFile(jar) as archive:
            matches = [entry for entry in archive.infolist() if entry.filename == 'BOOT-INF/classes/camoufox/worker.py']
            assert len(matches) == 1 and 0 < matches[0].file_size < 262144
            assert hashlib.sha256(archive.read(matches[0])).hexdigest() == self.worker_sha
        reservation = socket.socket()
        reservation.bind(('127.0.0.1',0))
        port = reservation.getsockname()[1]
        reservation.close()
        self.base = 'http://127.0.0.1:'+str(port)
        directory = Path('/storage') / ('generated-reader-tls-'+secrets.token_hex(6))
        directory.mkdir(mode=0o700)
        self.log_path = Path('/tmp') / ('generated-reader-business-tls-'+secrets.token_hex(6)+'.log')
        self.log = self.log_path.open('xb')
        environment = os.environ.copy()
        environment['READER_BROWSER_ALLOW_PRIVATE_NETWORKS'] = 'true'
        environment['JAVA_TOOL_OPTIONS'] = '-Xms256m -Xmx768m -Dfile.encoding=UTF-8'
        command = ['/opt/java/openjdk/bin/java','-jar',str(jar),
            '--reader.app.workDir='+str(directory),'--reader.server.port='+str(port),
            '--reader.server.bindAddress=127.0.0.1','--reader.app.webviewRenderer=camoufox',
            '--reader.app.camoufoxPythonExecutable=/usr/bin/python3',
            '--reader.app.camoufoxBrowserVersion='+self.browser_version,
            '--reader.app.secure=true','--reader.app.licenseCheckEnabled=false',
            '--spring.profiles.active=prod']
        self.process = subprocess.Popen(command,cwd=directory,env=environment,stdout=self.log,
            stderr=subprocess.STDOUT,stdin=subprocess.DEVNULL)
        self.pid_start = (Path('/proc')/str(self.process.pid)/'stat').read_text().rsplit(')',1)[1].split()[19]
        assert os.stat('/proc/self/ns/net').st_ino == os.stat('/proc/'+str(self.process.pid)+'/ns/net').st_ino
        deadline = time.monotonic()+55
        startup = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        while time.monotonic()<deadline:
            assert self.process.poll() is None
            try:
                status,value = self.helper.request(startup,self.base,'/reader3/getSystemInfo')
                if status==200 and value.get('isSuccess') is True:
                    break
            except (OSError,ValueError):
                pass
            time.sleep(.2)
        else:
            raise TimeoutError('Generated Reader startup')
        for actor in ('a','b'):
            jar_cookie = http.cookiejar.CookieJar()
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}),
                urllib.request.HTTPCookieProcessor(jar_cookie))
            username = 'tls'+actor+secrets.token_hex(4)
            password = 'Generated-ONLY-'+secrets.token_hex(12)
            for is_login in (False,True):
                status,value = self.helper.request(opener,self.base,'/reader3/login',
                    {'username':username,'password':password,'isLogin':is_login})
                assert status==200 and value.get('isSuccess') is True
                # Never copy generated login credentials/tokens into the report.
                self.auth.append({'actor':actor,'operation':'login' if is_login else 'register',
                    'status':status,'isSuccess':True,'returnDataFieldNames':sorted(value),
                    'cookieCount':len(jar_cookie)})
                value = None
            self.openers[actor] = opener
            status,value = self.helper.request(opener,self.base,'/reader3/getBookshelf')
            assert status==200 and value.get('isSuccess') is True and value.get('data')==[]
        self.started = True

    def owner(self,peer,proxy_port):
        assert self.process is not None and self.process.poll() is None
        process = Path('/proc')/str(self.process.pid)
        assert (process/'stat').read_text().rsplit(')',1)[1].split()[19] == self.pid_start
        descriptors = list((process/'fd').iterdir())
        assert len(descriptors)<4096
        inodes = set()
        for descriptor in descriptors:
            try:
                link = os.readlink(descriptor)
            except FileNotFoundError:
                continue
            if link.startswith('socket:['):
                inodes.add(link[8:-1])
        for filename in ('tcp','tcp6'):
            with (Path('/proc/net')/filename).open('rb') as stream:
                raw = stream.read(1048577)
            assert len(raw)<1048576
            for line in raw.decode('ascii').splitlines()[1:]:
                row = line.split()
                if len(row)>9 and row[9] in inodes and address(row[1])==peer and address(row[2])==('127.0.0.1',proxy_port):
                    return {'clientIsOwnedCurrentReaderJvm':True,'javaPid':self.process.pid,
                        'javaPidStartTick':self.pid_start,'javaSocketInode':row[9],
                        'clientHost':peer[0],'clientPort':peer[1],'proxyPort':proxy_port}
        raise RuntimeError('Generated upstream connection has no matching JVM-owned socket')

    def render(self,case,source,proxy_url,actor):
        assert self.started
        opener = self.openers[actor]
        if actor=='b':
            status,value = self.helper.request(opener,self.base,'/reader3/getBookSources')
            assert status==200 and value.get('isSuccess') is True and value.get('data')==[]
        source = json.loads(json.dumps(source))
        source['header'] = json.dumps({'proxy':proxy_url},ensure_ascii=False)
        status,value = self.helper.request(opener,self.base,'/reader3/saveBookSource',source)
        assert status==200 and value.get('isSuccess') is True
        status,saved = self.helper.request(opener,self.base,'/reader3/getBookSource',
            {'bookSourceUrl':source['bookSourceUrl']})
        assert status==200 and saved.get('isSuccess') is True
        assert saved['data']['header']==source['header'] and saved['data']['searchUrl']==source['searchUrl']
        assert saved['data']['ruleSearch']==source['ruleSearch'] or all(
            saved['data']['ruleSearch'].get(k)==v for k,v in source['ruleSearch'].items())
        status,value = self.helper.request(opener,self.base,'/reader3/searchBook',
            {'key':'GENERATED_ONLY','page':1,'bookSourceUrl':source['bookSourceUrl']})
        assert self.log_path.stat().st_size<4194304
        return {'actor':actor,'status':status,'returnData':value,'sourceRoundtripMatched':True,
            'configuredProxy':proxy_url,'actualJarSha256':self.jar_sha,
            'bookSourcesWereEmptyForSecondNamespace':True if actor=='b' else None}

    def stop(self):
        if self.process is not None and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        self.stopped = self.process is not None and self.process.poll() is not None
        if self.log is not None:
            self.log.close()
        return {'javaExitedBeforeOuterCleanup':self.stopped,
            'javaExitCode':self.process.returncode if self.process is not None else None,
            'privateLogBytes':self.log_path.stat().st_size if self.log is not None else 0,
            'realCredentialsImported':False}
