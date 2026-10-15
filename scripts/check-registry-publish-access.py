"""Read-only Docker Hub grant inspection. Never print a credential or request a registry write."""
import argparse
import base64
import json
import os
from pathlib import Path
import re
import time
from urllib.error import HTTPError
from urllib.parse import urlencode
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener

AUTH_ENDPOINT='https://auth.docker.io/token'
IMAGE=re.compile(r'[a-z0-9][a-z0-9._-]*/[a-z0-9][a-z0-9._-]*\Z')
MAX_RESPONSE=65536

class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self,*args):
        raise ValueError('Registry credential redirects are forbidden')

def reported_actions(token,image,now=None):
    """Inspect server-reported JWT claims, not a substitute for an actual push or signature verification."""
    if not isinstance(token,str) or not 1<=len(token)<=MAX_RESPONSE:
        raise ValueError('Invalid bounded token')
    pieces=token.split('.')
    if len(pieces)!=3 or not all(pieces):raise ValueError('Opaque token cannot be inspected safely')
    raw=base64.b64decode(pieces[1]+'='*(-len(pieces[1])%4),altchars=b'-_',validate=True)
    value=json.loads(raw)
    now=time.time() if now is None else now
    if not isinstance(value,dict) or value.get('iss')!='auth.docker.io' or value.get('aud')!='registry.docker.io':
        raise ValueError('Unexpected token issuer or audience')
    if type(value.get('exp')) not in (int,float) or not now<value['exp']<=now+86400:
        raise ValueError('Token expiration is invalid')
    rows=value.get('access',[])
    if not isinstance(rows,list) or len(rows)>16:raise ValueError('Unexpected grant list')
    actions=set()
    for row in rows:
        if not isinstance(row,dict):raise ValueError('Unexpected grant entry')
        if row.get('type')=='repository' and row.get('name')==image:
            granted=row.get('actions')
            if not isinstance(granted,list) or any(not isinstance(item,str) for item in granted):
                raise ValueError('Unexpected actions')
            actions.update(granted)
    return {'pull': 'pull' in actions, 'push': 'push' in actions}

def inspect(username,password,image,opener=None):
    if not IMAGE.fullmatch(image):raise ValueError('Invalid configured image')
    request=Request(AUTH_ENDPOINT+'?'+urlencode(dict(service='registry.docker.io',
        scope='repository:'+image+':pull,push',client_id='reader-readonly-publish-preflight')),
        headers={'Authorization':'Basic '+base64.b64encode((username+':'+password).encode()).decode(),
                 'Accept':'application/json','User-Agent':'Reader-registry-access-preflight/1'})
    opener=opener or build_opener(ProxyHandler({}),NoRedirect())
    try:
        with opener.open(request,timeout=20) as response:
            if response.status!=200:raise ValueError('Unexpected authentication status')
            raw=response.read(MAX_RESPONSE+1)
            if len(raw)>MAX_RESPONSE:raise ValueError('Authentication response exceeds byte bound')
            value=json.loads(raw)
    except HTTPError as error:
        status=error.code;error.close()
        return dict(authenticationHttpStatus=status,grantInspectable=False,pull=None,push=None)
    token=value.get('token',value.get('access_token'))
    grants=reported_actions(token,image)
    token=None
    return dict(authenticationHttpStatus=200,grantInspectable=True,**grants)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if args.output.exists():raise ValueError('Refuse to overwrite a prior observation')
    username=os.environ.get('DOCKERHUB_USERNAME','')
    password=os.environ.get('DOCKERHUB_PASSWORD','')
    configured=os.environ.get('DOCKERHUB_IMAGE','')
    owner=os.environ.get('GITHUB_REPOSITORY_OWNER','').lower()
    if not username or not password or not IMAGE.fullmatch(configured):raise ValueError('Missing or invalid registry configuration')
    if not re.fullmatch('[a-z0-9][a-z0-9_-]{0,63}',username.lower()):raise ValueError('Invalid registry account identifier')
    own_image=username.lower()+'/'+configured.split('/',1)[1]
    result=dict(schemaVersion=1,scope='read-only authentication-server reported grants, not an actual push',
        configuredImage=configured,credentialUsernameEqualsRepositoryOwner=username.lower()==owner,
        configuredNamespaceEqualsCredentialUsername=configured.split('/',1)[0]==username.lower(),
        credentialOrTokenPersisted=False,registryWriteRequested=False,independentJwtSignatureVerification=False,
        productionAccessed=False,observations={})
    try:
        result['observations']['configured']=inspect(username,password,configured)
        result['observations']['credentialNamespace']=result['observations']['configured'] if own_image==configured else inspect(username,password,own_image)
        if username.lower()==owner:result['publicOwnerImageCandidate']=owner+'/'+configured.split('/',1)[1]
        result['completed']=True
    except Exception as error:
        result['completed']=False
        result['errorClass']=type(error).__name__
    finally:
        password=None;username=None
    with args.output.open('x',encoding='utf-8') as output:json.dump(result,output,indent=2)
    print(json.dumps(result,ensure_ascii=True))
    return 0 if result['completed'] else 1

if __name__=='__main__':raise SystemExit(main())
