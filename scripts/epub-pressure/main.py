"""Full default packaged UI, synthetic large EPUB, same-budget browser and Reader."""
import argparse
from contextlib import redirect_stderr, redirect_stdout
import json
import os
from pathlib import Path
import re
import signal
import socket
import time

from fixture import PRESSURE, create, digest
from generated_account import prepare_generated_account
from storage_budget import StorageBudget
from ui import ui


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', required=True)
    parser.add_argument('--jar-sha', required=True)
    parser.add_argument('--architecture', choices=('amd64', 'arm64'), required=True)
    args = parser.parse_args()
    assert not __import__('sys').flags.optimize
    assert re.fullmatch('[0-9a-f]{40}', args.revision) and re.fullmatch('[0-9a-f]{64}', args.jar_sha)
    assert os.getuid() == os.getgid() == 10001 and set(os.getgroups()) <= {10001}
    status = dict(line.split(':', 1) for line in Path('/proc/self/status').read_text().splitlines() if ':' in line)
    assert all(int(status[k].strip(), 16) == 0 for k in ('CapEff', 'CapPrm', 'CapBnd', 'CapInh', 'CapAmb'))
    assert status['NoNewPrivs'].strip() == '1' and sorted(n for _, n in socket.if_nameindex()) == ['lo']
    assert Path('/proc/self/cgroup').read_text().strip() == '0::/'
    assert os.environ['READER_BUILD_REVISION'] == args.revision
    assert os.environ['JAVA_TOOL_OPTIONS'] == '-Xms256m -Xmx768m -Dfile.encoding=UTF-8'
    assert os.environ['READER_APP_WEBUI'] == 'vue3' and os.environ['READER_APP_WEBVIEWRENDERER'] == 'camoufox'
    assert digest(Path('/app/reader.jar')) == args.jar_sha
    output = Path('/verification-output/observation.json')
    assert not output.exists() and not output.is_symlink()
    observation = dict(schemaVersion=1, scope='default-image-generated-large-epub-pressure',
                       revision=args.revision, jarSha256=args.jar_sha, architecture=args.architecture,
                       passed=False, readerEightScenesAccepted=False, cases={},
                       defaultJvmUnchanged=True, downloadDeadlineSeconds=60, workerDeadlineSeconds=15,
                       uiObservationSeconds=30, fixtureGeneratedInsideAggregateBudget=True,
                       realCredentialsImported=False, privateBookInputRead=False,
                       originalJarAccountCompatibilityTested=False,
                       browserClientIncludedInAggregateBudget=True, hostGlobalCachesChanged=False)

    def interrupted(_sig, _frame):
        raise InterruptedError('Owned generated test interrupted')

    signal.signal(signal.SIGTERM, interrupted)
    started, exit_code = time.monotonic(), 2
    budget = StorageBudget(observation)
    budget.start()
    try:
        asset = Path('/verification-data/fixture/generated-pressure.epub')
        observation['fixture'] = create(asset, PRESSURE)
        with Path('/tmp/ui-generated.log').open('x') as log, redirect_stdout(log), redirect_stderr(log):
            workdir = Path('/verification-data/generated-account')
            credentials = prepare_generated_account(workdir, observation)
            ui(Path('/opt/java/openjdk/bin/java'), Path('/app/reader.jar'), asset, workdir, observation, credentials)
            del credentials
        expected = {'text805', 'text806', 'text806Refresh', 'html806', 'raw806', 'raw805', 'raw805Refresh', 'text805Return'}
        assert set(observation['cases']) == expected and observation['passed'] is True
        assert observation['ownedJavaStopped'] and observation['passiveSamplerStopped']
        assert observation['contextCloseFailureCode'] is None and observation['passiveSamplerErrors'] == []
        comparison = observation['rawRefreshComparison']
        assert all(comparison[k] is True for k in ('innerTextEqual', 'textContentEqual', 'nonWhitespaceInnerTextEqual', 'headingEqual'))
        assert comparison['beforeHeadingEqualsPreviousRawChapter'] is False
        assert all(comparison[k]['headingMatchesRequestedChapterTitle'] is True for k in ('before', 'after'))
        assert comparison['before']['chapterQueryIndex'] == comparison['after']['chapterQueryIndex'] == 805
        assert observation['exactPackagedEntrypointExecuted'] and observation['webUiOrRendererOverriddenByTest'] is False
        assert observation['fixture']['sha256'] == digest(asset)
        observation['readerEightScenesAccepted'], exit_code = True, 0
    except Exception as error:
        observation['passed'] = False
        observation['exceptionType'] = type(error).__name__
        frames, cursor = [], error.__traceback__
        while cursor:
            frames.append(dict(file=Path(cursor.tb_frame.f_code.co_filename).name,
                               function=cursor.tb_frame.f_code.co_name, line=cursor.tb_lineno))
            cursor = cursor.tb_next
        observation['boundedFailureFrames'] = frames[-8:]
    finally:
        budget.stop()
        storage = observation['generatedDiskStorageBudget']
        if not storage['monitorStopped'] or storage['metadataReadErrors'] or storage['budgetExceeded']:
            observation['passed'], observation['readerEightScenesAccepted'], exit_code = False, False, 2
        observation['seconds'] = round(time.monotonic() - started, 3)
        for value in observation['cases'].values():
            for key in ('bodySha256', 'textContentShaInternal', 'nonWhitespaceShaInternal', 'headingShaInternal'):
                value.pop(key, None)
        with output.open('x', encoding='utf-8') as stream:
            json.dump(observation, stream, indent=2)
        print(json.dumps(dict(phase='generated-pressure-terminal', accepted=observation['readerEightScenesAccepted'],
                             completedScenes=len(observation['cases']), seconds=observation['seconds'])), flush=True)
    return exit_code


if __name__ == '__main__':
    raise SystemExit(main())
