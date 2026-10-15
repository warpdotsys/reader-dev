#!/usr/bin/env python3
"""Compare this task's generated data across a deliberate local service restart."""
import argparse
import hashlib
import http.cookiejar
import importlib.util
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('local_generated_acceptance', Path(__file__).with_name('verify.py'))
verify = importlib.util.module_from_spec(spec); sys.modules[spec.name] = verify; spec.loader.exec_module(verify)


def identity():
    paths = {'licensePublicKey': verify.MOUNT / 'data/license-data/legacy-rsa-public.spki',
             'installedLicense': verify.MOUNT / 'data/storage/data/license.key',
             'generatedEpub': verify.DATA / 'generated-small.epub'}
    value = {}
    for name, path in paths.items():
        verify.regular(path)
        value[name] = hashlib.sha256(path.read_bytes()).hexdigest()
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('before', 'after'))
    args = parser.parse_args()
    output = verify.WORK / ('restart-' + args.action + '.json')
    if output.exists():
        raise ValueError('restart receipt exists; do not overwrite')
    state = verify.read_json(verify.WORK / 'service-state.json')
    if state.get('status') != 'running':
        raise ValueError('owned service is not running')
    report = {'schemaVersion': 1, 'stage': args.action, 'generatedOnly': True,
              'hashes': identity(), 'serviceState': state}
    if args.action == 'after':
        before = verify.read_json(verify.WORK / 'restart-before.json')
        if report['hashes'] != before['hashes'] or state.get('generatedNetworkTesting') is not False:
            raise ValueError('persistence or default network policy mismatch')
        if state['runRoot'] == before['serviceState']['runRoot']:
            raise ValueError('not a new service instance')
        opener = verify.local_opener(http.cookiejar.CookieJar())
        logged = verify.success(verify.request(opener, verify.BASE + '/reader3/login', 'POST',
            dict(username=verify.ACCOUNT, password=verify.PASSWORD, isLogin=True)))
        if logged.get('username') != verify.ACCOUNT:
            raise ValueError('generated account not retained')
        shelf = verify.success(verify.request(opener, verify.BASE + '/reader3/getBookshelf'))
        if not isinstance(shelf, list) or len(shelf) != 1 or shelf[0].get('name') != '纯生成 EPUB 压力回归':
            raise ValueError('generated shelf not retained')
        license_value = verify.success(verify.request(opener, verify.BASE + '/reader3/getLicense'))['license']
        if license_value.get('verified') is not True or license_value.get('type') != 'pro':
            raise ValueError('activated license not retained')
        report.update(accountLoginAccepted=True, generatedShelfRetained=True, activatedLicenseRetained=True,
                      defaultPrivateNetworkDenyRestored=True)
    with output.open('x', encoding='utf-8') as stream:
        json.dump(report, stream, indent=2, sort_keys=True)
    print(json.dumps({'stage': args.action, 'passed': True, 'generatedOnly': True}))


if __name__ == '__main__':
    main()
