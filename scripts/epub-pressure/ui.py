"""Real frozen eight UI scenarios plus passive OS counters; no browser code injection."""
import hashlib
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import threading
import time
import urllib.parse
import urllib.request
from worker_network import WorkerNetworkObservation
from file_network import FileNetworkObservation
from pressure import process_snapshot
observer_page = None
generated_chapter_titles = {}

def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''): h.update(block)
    return h.hexdigest()

def finite_error_code(error):
    if isinstance(error, TimeoutError) or type(error).__name__ == 'TimeoutError': return 'timeout'
    if type(error).__name__ == 'TargetClosedError': return 'closed'
    if isinstance(error, (AssertionError, ValueError, TypeError)): return 'validation'
    return 'other'

def run_diagnostic_preserving_primary(operation):
    try: operation(); return None
    except Exception as error: return finite_error_code(error)

def checkpoint(observation, phase):
    observation['activePhase'] = phase
    allowed = {'uiReaderStartup','uiWrongPassword','uiExistingGeneratedAccountLogin','uiImportPreview','uiSaveBook',
        'uiImportDialogCleanup','uiLogout','uiFreshContextRelogin','uiUnrememberedNewTab','uiRead805Text',
        'uiNext806Text','uiRefresh806Text','uiHtml806','uiRaw806','uiRaw805Previous','uiRaw805Refresh',
        'uiReturnText805','allEightCasesFinished','uiExceptionBeforeOwnedCleanup'}
    cases = sorted(observation.get('cases',{}))
    assert phase in allowed and set(cases) <= {'text805','text806','text806Refresh','html806','raw806','raw805','raw805Refresh','text805Return'}
    flags = {key:value for key,value in observation.items() if key.endswith('Accepted') and type(value) is bool}
    assert len(flags) <= 64 and all(key.isascii() and len(key) <= 96 for key in flags)
    # Bypass Python's private diagnostic-log redirect with finite metadata only.
    os.write(1,(json.dumps({'progress':{'phase':phase,'completedCaseNames':cases,'acceptedBooleanFlags':flags}})+'\n').encode())
    with Path('/verification-output/progress.jsonl').open('a',encoding='utf-8') as stream:
        stream.write(json.dumps({'phase':phase,'completedCaseNames':cases,'acceptedBooleanFlags':flags})+'\n')
    snapshot=process_snapshot(phase)
    observation.setdefault('boundedPhaseProcessObservations', []).append(snapshot)
    os.write(1,(json.dumps({'ownedPhaseCounters':snapshot})+'\n').encode())

def wait_readiness(base, child, observation):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    deadline = time.monotonic() + 75
    while time.monotonic() < deadline:
        assert child.poll() is None
        try:
            with opener.open(base + '/reader3/getSystemInfo', timeout=2) as reply:
                data = reply.read(131073)
                assert reply.status == 200 and len(data) <= 131072
                if json.loads(data)['isSuccess'] is True:
                    observation['readerReady'] = True
                    return
        except (OSError, ValueError, KeyError): pass
        time.sleep(.25)
    raise TimeoutError('Owned UI Reader not ready')

def metrics(page, raw=False):
    # The synthetic fixture text stays in this isolated test process's memory.
    # Some runtime profiles do not expose WebCrypto; never change that profile.
    value = page.evaluate("""raw => {
      const root = raw ? document.querySelector('iframe.epub-frame')?.contentDocument?.body
                       : document.querySelector('.reader-content');
      if (!root) return null;
      const text = root.innerText || '';
      const textContent = root.textContent || '';
      const heading = root.querySelector('h1,h2,h3')?.textContent || '';
      const frame = raw ? document.querySelector('iframe.epub-frame') : null;
      return {text, textContent, heading, textContentCharacters:textContent.length,
        chapterQueryIndex:Number(new URL(location.href).searchParams.get('chapter')),
        frameBusy:raw ? frame?.getAttribute('aria-busy') === 'true' : null,
        fontStatus:raw ? root.ownerDocument.fonts.status : null, paragraphs:root.querySelectorAll('p').length, characters:text.length,
        replacementCharacters:Array.from(text).filter(v=>v.charCodeAt(0)===65533).length,
        scripts:root.querySelectorAll('script').length,
        resourceUrlOnly:text.trim().startsWith('/book-assets/') || text.trim().startsWith('__API_ROOT__/book-assets/'),
        iframeHeight:raw ? document.querySelector('iframe.epub-frame').clientHeight : null};
    }""", raw)
    assert value is not None
    text = value.pop("text")
    value["bodySha256"] = hashlib.sha256(text.encode("utf-8")).hexdigest()
    value["textContentShaInternal"] = hashlib.sha256(value.pop('textContent').encode()).hexdigest()
    value["nonWhitespaceShaInternal"] = hashlib.sha256(''.join(text.split()).encode()).hexdigest()
    heading=value.pop('heading')
    value['headingShaInternal']=hashlib.sha256(heading.encode()).hexdigest()
    expected=generated_chapter_titles.get(value['chapterQueryIndex'])
    value['requestedChapterHeadingComparisonAvailable']=expected is not None
    value['headingMatchesRequestedChapterTitle']=expected is not None and ''.join(heading.split())==''.join(expected.split())
    value["nonWhitespaceCharacters"] = len(''.join(text.split()))
    return value

def wait_content(page, raw=False, previous=None):
    if raw: page.wait_for_function(Path("/verification/epub-frame-readiness.js").read_text())
    page.wait_for_function("""raw => {
      const root = raw ? document.querySelector('iframe.epub-frame')?.contentDocument?.body
                       : document.querySelector('.reader-content');
      return root && (root.innerText||'').length > 1000;
    }""", arg=raw)
    value = metrics(page, raw)
    if previous:
        deadline = time.monotonic() + 30
        while value["bodySha256"] == previous:
            assert time.monotonic() < deadline
            page.wait_for_timeout(50)
            value = metrics(page, raw)
    assert value["characters"] > 1000 and value["paragraphs"] > 20
    assert value["replacementCharacters"] == value["scripts"] == 0 and value["resourceUrlOnly"] is False
    if raw:
        assert value["iframeHeight"] >= 320
    return value

def ui(java, jar, asset, workdir, observation, credentials):
    global observer_page
    observation['workerInstrumentationInstalled'] = False
    observation['testOnlyPageHtmlInstrumented'] = False
    assert workdir.is_dir() and not workdir.is_symlink()
    username, password = credentials["username"], credentials["password"]
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    base = "http://127.0.0.1:" + str(port)
    # Exercise the exact packaged production entrypoint, not a substituted JAR/CLI.
    assert str(jar) == '/app/reader.jar'
    command = ['/usr/local/bin/reader-entrypoint']
    child_env = os.environ.copy()
    child_env.update({'READER_APP_WORKDIR':str(workdir), 'READER_SERVER_PORT':str(port),
        'READER_SERVER_BINDADDRESS':'127.0.0.1', 'READER_APP_SECURE':'true',
        'READER_APP_LICENSECHECKENABLED':'false', 'READER_APP_SHELFUPDATEINTEVAL':'0',
        'READER_APP_REMOTEBOOKSOURCEUPDATEINTERVAL':'0', 'READER_APP_AUTOBACKUPUSERDATA':'false',
        'READER_APP_AUTOCLEARINACTIVEUSER':'0'})
    assert child_env['READER_APP_WEBUI'] == 'vue3' and child_env['READER_APP_WEBVIEWRENDERER'] == 'camoufox'
    observation['exactPackagedEntrypointExecuted'] = True
    observation['webUiOrRendererOverriddenByTest'] = False
    with (workdir / "reader-private.log").open("xb") as log:
        child = subprocess.Popen(command, cwd="/app", env=child_env, stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        stop_sampler = threading.Event()
        sampler_errors = []
        observation['boundedPressureSamples'] = []
        def collect_pressure():
            try:
                while not stop_sampler.is_set():
                    assert len(observation['boundedPressureSamples']) < 150
                    observation['boundedPressureSamples'].append(process_snapshot(observation.get('activePhase', 'uiStartup')))
                    stop_sampler.wait(2.0)
            except Exception as error:
                sampler_errors.append(type(error).__name__)
        sampler = threading.Thread(target=collect_pressure, daemon=True)
        sampler.start()
        try:
            checkpoint(observation, 'uiReaderStartup')
            wait_readiness(base, child, observation)
            import importlib.util
            spec = importlib.util.spec_from_file_location('default_ui', '/verification/verify-reader-default-ui.py')
            verifier = importlib.util.module_from_spec(spec); spec.loader.exec_module(verifier)
            observation['defaultPackagedUiStaticObservation'] = verifier.observe(base, jar)
            observation['defaultPackagedUiStaticAccepted'] = True
            for key in ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS"):
                os.environ[key] = "1"
            from camoufox import Camoufox, DefaultAddons
            with Camoufox(headless=True, geoip=False, block_webrtc=True,
                    exclude_addons=[DefaultAddons.UBO], i_know_what_im_doing=True) as browser:
                context = browser.new_context(service_workers="block")
                try:
                    observation["browserAndContextStarted"] = True
                    page = context.new_page()
                    passive_worker = WorkerNetworkObservation(base, observation)
                    passive_worker.attach(page)
                    file_observer=FileNetworkObservation(base,observation)
                    file_observer.attach(page)
                    observer_page = page
                    page.set_default_timeout(30000)
                    observation["metricsTextHeldOnlyInPrivateProcessMemory"] = True
                    observation["browserCapabilities"] = page.evaluate("()=>({crypto:typeof crypto==='object', subtle:typeof crypto==='object' && typeof crypto.subtle==='object'})")
                    def login_existing(remember, expect_empty):
                        page.goto(base + '/login')
                        page.locator('input[autocomplete=username]').fill(username)
                        page.locator('input[autocomplete=current-password]').fill(password)
                        if not remember: page.locator('.remember-box').uncheck()
                        with page.expect_response(lambda response: urllib.parse.urlsplit(response.url).path == '/reader3/login') as logged:
                            with page.expect_response(lambda response: urllib.parse.urlsplit(response.url).path == '/reader3/getBookshelf') as shelf:
                                page.locator('.submit-btn').click()
                        reply, shelf_reply = logged.value, shelf.value
                        data, shelf_data = reply.json(), shelf_reply.json()
                        assert reply.status == shelf_reply.status == 200
                        assert reply.request.post_data_json['isLogin'] is True
                        assert data['isSuccess'] is True and data['errorMsg'] == '' and data['data']['username'] == username
                        current_token = data['data']['accessToken']
                        assert current_token.startswith(username + ':')
                        assert shelf_data['isSuccess'] is True and shelf_data['errorMsg'] == ''
                        assert shelf_reply.headers.get('x-reader-namespace') == username
                        assert shelf_data['data'] == [] if expect_empty else len(shelf_data['data']) == 1 and shelf_data['data'][0]['bookUrl'] == url
                        page.locator('.bookshelf-page').wait_for()
                        assert page.locator('.user-chip').inner_text() == username
                        state = page.evaluate("""() => ({localPresent:!!localStorage.getItem('reader_access_token'),
                            sessionPresent:!!sessionStorage.getItem('reader_access_token')})""")
                        assert state['sessionPresent'] is True and state['localPresent'] is remember
                        return current_token
                    checkpoint(observation, 'uiWrongPassword')
                    page.goto(base + '/login')
                    page.locator('input[autocomplete=username]').fill(username)
                    page.locator('input[autocomplete=current-password]').fill('Wrong-' + secrets.token_hex(16))
                    with page.expect_response(lambda response: urllib.parse.urlsplit(response.url).path == '/reader3/login') as failed:
                        page.locator('.submit-btn').click()
                    reply, data = failed.value, failed.value.json()
                    assert reply.status == 200 and reply.request.post_data_json['isLogin'] is True
                    assert data['isSuccess'] is False and data['errorMsg'] == '密码错误' and data.get('data') is None
                    assert set(data) <= {'isSuccess', 'errorMsg', 'data'}
                    observation['restoredWrongPasswordReplyFields'] = sorted(data)
                    observation['restoredWrongPasswordDataPresence'] = 'null' if 'data' in data else 'missing'
                    observation['wrongPasswordJsonFieldPresenceMatchesGenerated'] = observation['restoredWrongPasswordReplyFields'] == observation['generatedWrongPasswordReplyFields']
                    page.get_by_text('密码错误', exact=True).wait_for()
                    assert page.locator('.bookshelf-page').count() == 0
                    observation['wrongPasswordContractAndUiAccepted'] = True
                    checkpoint(observation, 'uiExistingGeneratedAccountLogin')
                    first_token = login_existing(True, True)
                    observation['existingGeneratedAccountLoginAccepted'] = True
                    observation['rememberedTokenStoredInBothAccepted'] = True
                    checkpoint(observation, "uiImportPreview")
                    page.locator(".import-btn").click()
                    dialog = page.locator("[aria-label='导入本地书籍']")
                    dialog.wait_for()
                    with page.expect_response(lambda response: urllib.parse.urlsplit(response.url).path.endswith("/reader3/importBookPreview")) as preview:
                        with page.expect_file_chooser() as chooser:
                            page.locator(".dropzone").click()
                        chooser.value.set_files(str(asset))
                    reply = preview.value
                    value = reply.json()
                    assert reply.status == 200 and value["isSuccess"] is True and value["errorMsg"] == ""
                    page.locator(".preview-panel").wait_for()
                    observation["importPreviewViaUiSucceeded"] = True
                    checkpoint(observation, "uiSaveBook")
                    with page.expect_response(lambda response: urllib.parse.urlsplit(response.url).path.endswith("/reader3/saveBook")) as saved:
                        dialog.locator(".accent-btn").click()
                    reply = saved.value
                    value = reply.json()
                    assert reply.status == 200 and value["isSuccess"] is True and value["errorMsg"] == ""
                    book = value["data"]
                    assert isinstance(book, dict) and book["origin"] == "loc_book"
                    url = book["bookUrl"]
                    assert isinstance(url, str) and url.startswith("storage/data/" + username + "/")
                    observation["importSaveViaUiSucceeded"] = True
                    page.locator(".bookshelf-page").wait_for()
                    checkpoint(observation, 'uiImportDialogCleanup')
                    # Real rendered interactions; no Vue state injection or request interception.
                    page.wait_for_function("""() => {
                        const dlg = document.querySelector('[aria-label="导入本地书籍"]');
                        return !dlg || !dlg.querySelector('.dlg-close').disabled;
                    }""")
                    manual_close = dialog.is_visible()
                    if manual_close:
                        dialog.locator('.dlg-close').click()
                    dialog.wait_for(state='hidden')
                    page.locator('.import-btn').click()
                    dialog.wait_for()
                    page.wait_for_timeout(1100)
                    assert dialog.is_visible() and dialog.locator('.file-list .file-row').count() == 0
                    observation['importDialogManualCloseBeforeTimerObserved'] = manual_close
                    observation['importDialogReopensEmptyAccepted'] = True
                    observation['reopenedImportSurvivesOldAutoCloseAccepted'] = True
                    dialog.locator('.dlg-close').click(); dialog.wait_for(state='hidden')
                    checkpoint(observation, 'uiLogout')
                    with page.expect_response(lambda response: urllib.parse.urlsplit(response.url).path == '/reader3/logout') as revoked:
                        page.locator('.logout-btn').click()
                    reply, data = revoked.value, revoked.value.json()
                    assert reply.status == 200 and data['isSuccess'] is True and data['errorMsg'] == '' and data['data'] == 'NEED_LOGIN'
                    page.locator('.login-page').wait_for()
                    cleared = page.evaluate("""() => ({localAbsent:!localStorage.getItem('reader_access_token'),
                        sessionAbsent:!sessionStorage.getItem('reader_access_token')})""")
                    assert cleared['localAbsent'] is cleared['sessionAbsent'] is True
                    page.reload(); page.locator('.login-page').wait_for()
                    request = urllib.request.Request(base + '/reader3/getBookshelf?' + urllib.parse.urlencode({
                        'accessToken':first_token, 'readerAuth':'access-token'}))
                    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
                    with opener.open(request, timeout=15) as reply:
                        raw = reply.read(131073); assert reply.status == 200 and len(raw) <= 131072
                        data = json.loads(raw)
                    assert data['isSuccess'] is False and data['data'] == 'NEED_LOGIN' and data['errorMsg']
                    observation['logoutThreeFieldContractAccepted'] = True
                    observation['logoutBrowserTokensClearedAccepted'] = True
                    observation['logoutReloadRemainsAnonymousAccepted'] = True
                    observation['revokedTokenRejectedByRealApi'] = True
                    context.close()
                    checkpoint(observation, 'uiFreshContextRelogin')
                    context = browser.new_context(service_workers='block')
                    page = context.new_page(); page.set_default_timeout(30000)
                    observer_page = page
                    passive_worker = WorkerNetworkObservation(base, observation); passive_worker.attach(page)
                    file_observer.attach(page)
                    second_token = login_existing(False, False)
                    assert second_token != first_token
                    observation['freshContextReloginAccepted'] = True
                    observation['existingShelfNamespaceConfirmedAccepted'] = True
                    observation['unrememberedTokenSessionOnlyAccepted'] = True
                    page.reload(); page.locator('.bookshelf-page').wait_for()
                    observation['unrememberedRefreshKeepsSessionAccepted'] = True
                    checkpoint(observation, 'uiUnrememberedNewTab')
                    new_tab = context.new_page()
                    try:
                        new_tab.goto(base + '/'); new_tab.locator('.login-page').wait_for()
                        assert new_tab.locator('.bookshelf-page').count() == 0
                    finally: new_tab.close()
                    observation['newTabDoesNotAdoptCookieOnlyIdentityAccepted'] = True
                    del first_token, second_token, password, credentials, raw, data
                    def observe_chapter_titles(response):
                        if urllib.parse.urlsplit(response.url).path!='/reader3/getChapterList':return
                        data=response.json()
                        assert response.status==200 and data['isSuccess'] is True and isinstance(data['data'],list)
                        assert 1<=len(data['data'])<=8192
                        for index,item in enumerate(data['data']):
                            assert isinstance(item['title'],str) and len(item['title'])<=4096
                            generated_chapter_titles[index]=item['title']
                    page.on('response',observe_chapter_titles)
                    reader = base + "/reader/" + urllib.parse.quote(url, safe="")
                    checkpoint(observation, "uiRead805Text")
                    page.goto(reader + "?chapter=805")
                    first = wait_content(page)
                    observation["cases"]["text805"] = first
                    checkpoint(observation, "uiNext806Text")
                    page.locator(".chapter-nav button").last.click()
                    page.wait_for_function("()=>new URL(location.href).searchParams.get('chapter')==='806'")
                    second = wait_content(page, previous=first["bodySha256"])
                    observation["cases"]["text806"] = second
                    checkpoint(observation, "uiRefresh806Text")
                    page.reload()
                    refreshed = wait_content(page)
                    assert refreshed["bodySha256"] == second["bodySha256"]
                    observation["cases"]["text806Refresh"] = refreshed
                    checkpoint(observation, "uiHtml806")
                    page.locator("button[title^='EPUB 排版模式']").click()
                    page.locator(".reader-content.epub-html").wait_for()
                    observation["cases"]["html806"] = wait_content(page)
                    checkpoint(observation, "uiRaw806")
                    page.locator("button[title^='EPUB 排版模式']").click()
                    page.locator("iframe.epub-frame").wait_for()
                    assert page.locator("iframe.epub-frame").get_attribute("sandbox") == "allow-same-origin"
                    raw = wait_content(page, raw=True)
                    observation["cases"]["raw806"] = raw
                    checkpoint(observation, "uiRaw805Previous")
                    page.locator(".chapter-nav button").first.click()
                    page.wait_for_function("()=>new URL(location.href).searchParams.get('chapter')==='805'")
                    raw_first = wait_content(page, raw=True, previous=raw["bodySha256"])
                    observation["cases"]["raw805"] = raw_first
                    checkpoint(observation, "uiRaw805Refresh")
                    page.reload()
                    page.locator("iframe.epub-frame").wait_for()
                    refreshed_raw = wait_content(page, raw=True)
                    observation['rawRefreshComparison']={
                        'innerTextEqual':refreshed_raw['bodySha256']==raw_first['bodySha256'],
                        'textContentEqual':refreshed_raw['textContentShaInternal']==raw_first['textContentShaInternal'],
                        'nonWhitespaceInnerTextEqual':refreshed_raw['nonWhitespaceShaInternal']==raw_first['nonWhitespaceShaInternal'],
                        'headingEqual':refreshed_raw['headingShaInternal']==raw_first['headingShaInternal'],
                        'beforeHeadingEqualsPreviousRawChapter':raw_first['headingShaInternal']==raw['headingShaInternal'],
                        'before':{key:raw_first[key] for key in ('characters','paragraphs','textContentCharacters','nonWhitespaceCharacters','chapterQueryIndex','frameBusy','fontStatus','requestedChapterHeadingComparisonAvailable','headingMatchesRequestedChapterTitle')},
                        'after':{key:refreshed_raw[key] for key in ('characters','paragraphs','textContentCharacters','nonWhitespaceCharacters','chapterQueryIndex','frameBusy','fontStatus','requestedChapterHeadingComparisonAvailable','headingMatchesRequestedChapterTitle')},
                        'generatedStrictBodyAssertionUnchanged':True,'sharedCurrentDocumentGateUsed':True}
                    assert refreshed_raw["bodySha256"] == raw_first["bodySha256"]
                    observation["cases"]["raw805Refresh"] = refreshed_raw
                    checkpoint(observation, "uiReturnText805")
                    page.locator("button[title^='EPUB 排版模式']").click()
                    last = wait_content(page)
                    assert last["bodySha256"] == first["bodySha256"]
                    observation["cases"]["text805Return"] = last
                    checkpoint(observation, "allEightCasesFinished")
                    generated_chapter_titles.clear()
                    observation["passed"] = True
                    observation["activePhase"] = "complete"
                except Exception as primary:
                    observation["primaryUiFailureCode"] = finite_error_code(primary)
                    run_diagnostic_preserving_primary(lambda: checkpoint(observation, "uiExceptionBeforeOwnedCleanup"))
                    observation["visibleStateReadFailureCode"] = run_diagnostic_preserving_primary(lambda: observation.update(boundedVisibleState=page.evaluate("""()=> {
                        const text=document.querySelector('.epub-error .state-text')?.textContent||'';
                        return {rawIframePresent:!!document.querySelector('iframe.epub-frame'),
                          rawErrorPresent:!!text, rawErrorDownloadTimeout:text.includes('EPUB 下载超时'),
                          rawErrorWorkerTimeout:text.includes('EPUB 解压超时'),
                          rawErrorWorkerStarting:text.includes('阶段：等待 Worker 启动'),
                          rawErrorWorkerReceiving:text.includes('阶段：等待接收文件'),
                          rawErrorWorkerExpanding:text.includes('阶段：解压文件'),
                          rawErrorWorkerTransferring:text.includes('阶段：等待解压结果传递'),
                          rawErrorWorkerFailure:text.includes('EPUB 解压 Worker'),
                          genericChapterErrorPresent:!!document.querySelector('.retry-btn'),
                          loadingStateCount:document.querySelectorAll('.loading-text').length};
                    }""")))
                    raise
                finally:
                    observer_page = None
                    observation["contextCloseFailureCode"] = run_diagnostic_preserving_primary(context.close)
        finally:
            stop_sampler.set()
            sampler.join(timeout=2)
            observation['passiveSamplerStopped'] = not sampler.is_alive()
            observation['passiveSamplerErrors'] = sampler_errors
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
            observation["ownedJavaStopped"] = child.poll() is not None
