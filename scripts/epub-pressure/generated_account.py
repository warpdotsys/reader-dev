"""Actual image-generated account; not original-JAR or real-account evidence."""
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import time
import urllib.parse
import urllib.request


def prepare_generated_account(workdir, observation):
    if workdir.exists() or workdir.is_symlink():
        raise ValueError('Refusing any existing Reader storage')
    workdir.mkdir(mode=0o700)
    username, password = 'generatedui' + secrets.token_hex(5), 'Generated-' + secrets.token_hex(16)
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    base = 'http://127.0.0.1:' + str(port)
    env = os.environ.copy()
    env.update(READER_APP_WORKDIR=str(workdir), READER_SERVER_PORT=str(port),
               READER_SERVER_BINDADDRESS='127.0.0.1', READER_APP_SECURE='true',
               READER_APP_LICENSECHECKENABLED='false', READER_APP_SHELFUPDATEINTEVAL='0',
               READER_APP_REMOTEBOOKSOURCEUPDATEINTERVAL='0', READER_APP_AUTOBACKUPUSERDATA='false',
               READER_APP_AUTOCLEARINACTIVEUSER='0')
    assert env['JAVA_TOOL_OPTIONS'] == '-Xms256m -Xmx768m -Dfile.encoding=UTF-8'
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))

    def request(path, payload=None):
        raw = None if payload is None else json.dumps(payload).encode()
        value = urllib.request.Request(base + path, data=raw,
                                       headers={'Content-Type': 'application/json'} if raw else {})
        with opener.open(value, timeout=15) as response:
            reply = response.read(131073)
            assert response.status == 200 and len(reply) <= 131072
            return json.loads(reply)

    with (workdir / 'registration-generated.log').open('xb') as log:
        child = subprocess.Popen(['/usr/local/bin/reader-entrypoint'], cwd='/app', env=env,
                                 stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 75
            while time.monotonic() < deadline:
                assert child.poll() is None
                try:
                    if request('/reader3/getSystemInfo')['isSuccess'] is True:
                        break
                except (OSError, ValueError, KeyError):
                    pass
                time.sleep(.25)
            else:
                raise TimeoutError('Generated account Reader startup failed')
            registered = request('/reader3/login', dict(username=username, password=password, isLogin=False))
            assert registered['isSuccess'] is True and registered['errorMsg'] == ''
            assert registered['data']['username'] == username
            token = registered['data']['accessToken']
            assert token.startswith(username + ':')
            wrong = request('/reader3/login', dict(username=username, password='Wrong-' + secrets.token_hex(16), isLogin=True))
            assert wrong['isSuccess'] is False and wrong['errorMsg'] == '密码错误' and wrong.get('data') is None
            assert set(wrong) <= {'isSuccess', 'errorMsg', 'data'}
            observation['generatedWrongPasswordReplyFields'] = sorted(wrong)
            observation['generatedWrongPasswordDataPresence'] = 'null' if 'data' in wrong else 'missing'
            revoked = request('/reader3/logout?' + urllib.parse.urlencode({'accessToken': token}), {})
            assert revoked['isSuccess'] is True and revoked['errorMsg'] == '' and revoked['data'] == 'NEED_LOGIN'
            users_file = workdir / 'storage/data/users.json'
            assert users_file.is_file() and not users_file.is_symlink() and users_file.stat().st_size <= 131072
            users = json.loads(users_file.read_bytes())
            assert set(users) == {username} and users[username]['username'] == username
            assert users[username]['salt'] and users[username]['password'] != password
            observation['generatedRegistrationAccepted'] = True
            observation['manualUserRecordOrPasswordHashSeeded'] = False
        finally:
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
            observation['generatedRegistrationProcessStopped'] = child.poll() is not None
    assert observation['generatedRegistrationProcessStopped']
    return dict(username=username, password=password)
