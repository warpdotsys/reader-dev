"""Read-only counters for exactly the owned UID and cgroup; no command lines or bodies."""
import os
from pathlib import Path
import threading
import time

CGROUP = '/'  # Docker's private cgroup namespace; the host controller observes its parent slice.
LOCK = threading.Lock()
PREVIOUS = {}
CPU_OBSERVED = {}
TICKS = os.sysconf('SC_CLK_TCK')
PAGE_BYTES = os.sysconf('SC_PAGE_SIZE')
assert 1 <= TICKS <= 10000 and PAGE_BYTES >= 4096

def category(name):
    if name == 'java': return 'java'
    if name.startswith('python'): return 'python'
    if name == 'node': return 'browser-driver'
    if name in {'firefox', 'camoufox-bin', 'MainThread'}: return 'browser-main'
    if name in {'Web Content', 'Isolated Web Co', 'Privileged Cont', 'WebExtensions'}: return 'browser-content'
    if name in {'Socket Process', 'RDD Process', 'Utility Process'}: return 'browser-service'
    return 'other'

def executable_category(status_name, executable_name):
    if executable_name == 'node': return 'browser-driver'
    if executable_name == 'java': return 'java'
    if executable_name and executable_name.startswith('python'): return 'python'
    if executable_name in ('camoufox-bin','camoufox','firefox','firefox-bin'):
        value = category(status_name)
        return value if value.startswith('browser-') else 'browser-service'
    # MainThread is ambiguous without the executable; never assign it to Firefox.
    return 'other' if status_name == 'MainThread' else category(status_name)

def stat(path):
    raw = path.read_text()
    close = raw.rfind(')')
    assert close >= 3
    name = raw[raw.index('(') + 1:close]
    fields = raw[close + 2:].split()
    assert len(fields) >= 22
    return name, int(fields[11]) + int(fields[12]), int(fields[19]), int(fields[21]) * PAGE_BYTES

def worker_thread_kind(name, is_main):
    value = name.lower()
    if is_main: return 'process-main-thread'
    if value.startswith(('dom worker', 'domworker')): return 'dom-worker'
    if value.startswith(('js helper', 'jshelper', 'gc helper', 'gchelper')): return 'js-gc-helper'
    if 'compositor' in value or 'renderer' in value: return 'render'
    return 'other'

def status_memory_bytes(fields, key):
    value = fields.get(key)
    if value is None: return None
    pieces = value.split()
    assert len(pieces) == 2 and pieces[1] == 'kB' and pieces[0].isascii() and pieces[0].isdecimal()
    return int(pieces[0]) * 1024

def process_snapshot(phase):
    with LOCK:
        assert Path('/proc/self/cgroup').read_text().strip().split('::', 1)[1] == CGROUP
        groups = {}
        thread_groups = {}
        errors = []
        for proc in Path('/proc').iterdir():
            if not proc.name.isascii() or not proc.name.isdecimal(): continue
            try:
                fields = dict(line.split(':', 1) for line in (proc / 'status').read_text().splitlines() if ':' in line)
                if int(fields['Uid'].split()[0]) != 10001: continue
                assert (proc / 'cgroup').read_text().strip().split('::', 1)[1] == CGROUP
                name, cpu, birth, rss = stat(proc / 'stat')
                try: executable = Path(os.readlink(proc / 'exe')).name
                except PermissionError: executable = None
                group = executable_category(name, executable)
                row = groups.setdefault(group, {'processes': 0, 'threads': 0, 'rssBytesSum': 0, 'cpuTicksObserved': 0})
                row['processes'] += 1; row['threads'] += int(fields['Threads'].strip())
                row.setdefault('executableResolvedProcesses', 0)
                row.setdefault('executableUnavailableProcesses', 0)
                row['executableResolvedProcesses' if executable is not None else 'executableUnavailableProcesses'] += 1
                row['rssBytesSum'] += max(0, rss); row['cpuTicksObserved'] += cpu
                keys = {'statusRssAnonBytesSum':'RssAnon', 'statusRssFileBytesSum':'RssFile',
                    'statusRssShmemBytesSum':'RssShmem', 'statusVmSwapBytesSum':'VmSwap'}
                measured = {out:status_memory_bytes(fields, key) for out,key in keys.items()}
                row.setdefault('memoryStatusMeasuredProcesses', 0)
                row.setdefault('memoryStatusMissingProcesses', 0)
                for out in keys: row.setdefault(out, 0)
                if all(value is not None for value in measured.values()):
                    row['memoryStatusMeasuredProcesses'] += 1
                    for out,value in measured.items(): row[out] += value
                else: row['memoryStatusMissingProcesses'] += 1
                identity = (int(proc.name), birth)
                old = PREVIOUS.get(identity, 0)
                assert cpu >= old and len(PREVIOUS) <= 4096
                CPU_OBSERVED[group] = CPU_OBSERVED.get(group, 0) + cpu - old
                PREVIOUS[identity] = cpu
                if not group.startswith('browser-') or group == 'browser-driver': continue
                for task in (proc / 'task').iterdir():
                    try:
                        task_name, ticks, _, _ = stat(task / 'stat')
                        kind = worker_thread_kind(task_name, task.name == proc.name)
                        count = thread_groups.setdefault(kind, {'threads': 0, 'cpuTicksObserved': 0})
                        count['threads'] += 1; count['cpuTicksObserved'] += ticks
                    except (FileNotFoundError, ProcessLookupError): continue
            except (FileNotFoundError, ProcessLookupError): continue
            except Exception as error:
                if len(errors) < 4: errors.append(type(error).__name__)
        parent = Path('/sys/fs/cgroup') / CGROUP.lstrip('/')
        # This root is the one isolated container; host provenance keeps the actual aggregate counters.
        memory = dict(line.split() for line in (parent / 'memory.stat').read_text().splitlines())
        events = dict(line.split() for line in (parent / 'memory.events').read_text().splitlines())
        cpu_stat = dict(line.split() for line in (parent / 'cpu.stat').read_text().splitlines())
        assert len(groups) <= 7 and len(thread_groups) <= 5
        return {'phase': phase, 'epochMs': time.time_ns() // 1000000, 'monotonicMs': round(time.monotonic() * 1000),
            'ticksPerSecond': TICKS, 'processGroups': groups, 'browserThreadGroups': thread_groups,
            'cpuTicksLifetimeObservedLowerBound': dict(CPU_OBSERVED), 'readFailureTypes': errors,
            'rssSumsIncludeSharedPagesNotUniqueMemory': True,
            'memoryStatusTotalsAreAvailableProcessSumsNotUniquePrivateMemory': True,
            'processMapsCommandLinesOrMemoryBodiesRead': False,
            'classificationUsesOwnedExecutableBasenameNotMainThreadNameAlone': True,
            'aggregate': {'memoryCurrent': int((parent / 'memory.current').read_text()),
                'pidsCurrent': int((parent / 'pids.current').read_text()),
                'memoryHighEvents': int(events['high']),
                'memoryStat': {key: int(memory[key]) for key in ('anon', 'file', 'shmem', 'pgscan_direct', 'pgsteal_direct', 'workingset_refault_file')},
                'cpuStat': {key: int(cpu_stat[key]) for key in ('usage_usec', 'user_usec', 'system_usec', 'nr_periods', 'nr_throttled', 'throttled_usec')}}}
