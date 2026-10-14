"""Generated cgroup-file doubles test policy rejection, not real Linux execution."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).parents[3]
SPEC = importlib.util.spec_from_file_location('pressure_runner_guard', ROOT / 'scripts/epub-pressure/runner.py')
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


class GeneratedPressureRunnerBudgetTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='reader-generated-cgroup-double-')
        self.addCleanup(self.temporary.cleanup)
        self.group = Path(self.temporary.name)
        for name, value in {'cpu.max': '200000 100000', 'memory.max': '2147483648',
                            'memory.high': '1610612736', 'memory.peak': '1500000000',
                            'memory.swap.max': '0', 'pids.max': '512',
                            'memory.events': 'low 0\nhigh 123\nmax 0\noom 0\noom_kill 0\n',
                            'pids.events': 'max 0\n'}.items():
            (self.group / name).write_text(value, encoding='ascii')

    def test_policy_double_preserves_exact_limits_and_unmodified_events(self):
        value = RUNNER.budget(self.group)
        self.assertEqual(200000, value['cpuQuota'])
        self.assertEqual(100000, value['cpuPeriod'])
        self.assertEqual(123, value['memoryEvents']['high'])
        self.assertEqual(0, value['swapMaxBytes'])
        self.assertEqual(512, value['pidsMax'])

    def test_wider_cpu_or_zero_period_rejected(self):
        for value in ('300000 100000', '0 0', 'max 100000'):
            (self.group / 'cpu.max').write_text(value, encoding='ascii')
            with self.assertRaises((ValueError, AssertionError)):
                RUNNER.budget(self.group)

    def test_wider_memory_high_swap_and_pids_rejected(self):
        for name, value in (('memory.max', '3221225472'), ('memory.high', '2147483648'),
                             ('memory.swap.max', '1048576'), ('pids.max', '1024')):
            path = self.group / name
            old = path.read_text(encoding='ascii')
            path.write_text(value, encoding='ascii')
            with self.subTest(file=name), self.assertRaises(AssertionError):
                RUNNER.budget(self.group)
            path.write_text(old, encoding='ascii')

    def test_oom_and_pid_events_are_retained_not_zeroed_by_observer(self):
        (self.group / 'memory.events').write_text('high 200\nmax 7\noom 2\noom_kill 1\n', encoding='ascii')
        (self.group / 'pids.events').write_text('max 3\n', encoding='ascii')
        value = RUNNER.budget(self.group)
        self.assertEqual(7, value['memoryEvents']['max'])
        self.assertEqual(2, value['memoryEvents']['oom'])
        self.assertEqual(1, value['memoryEvents']['oom_kill'])
        self.assertEqual(3, value['pidsEvents']['max'])

    def test_missing_actual_counter_file_cannot_be_defaulted_to_zero(self):
        (self.group / 'memory.events').unlink()
        with self.assertRaises(FileNotFoundError):
            RUNNER.budget(self.group)


if __name__ == '__main__':
    unittest.main()
