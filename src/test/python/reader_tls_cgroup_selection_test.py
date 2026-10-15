"""Generated controller-tree fixtures only, not executed container acceptance."""
import importlib.util
from pathlib import Path
import tempfile
import types
import unittest

ROOT=Path(__file__).parents[3]
spec=importlib.util.spec_from_file_location('generated_current_group_selection',ROOT/'scripts/smoke-camoufox-tls.py')
smoke=importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


class ReaderTlsCgroupSelectionTest(unittest.TestCase):
    def select(self,membership,group=None):
        with tempfile.TemporaryDirectory(prefix='reader-generated-cgroup-selection-') as temporary:
            directory=Path(temporary)
            base=directory/'controller-tree'
            base.mkdir()
            member=directory/'generated-membership'
            member.write_text(membership,encoding='ascii')
            if group is not None:
                (base/group).mkdir(parents=True)
            resources=types.SimpleNamespace(ROOT=None)
            smoke.select_current_cgroup(resources,member,base)
            self.assertEqual(resources.ROOT,base.resolve()/(group or ''))

    def test_docker_private_cgroup_namespace_root(self):
        self.select('0::/\n')

    def test_rootless_current_group_not_host_root(self):
        self.select('0::/system.slice/GENERATED_ONLY/client\n','system.slice/GENERATED_ONLY/client')

    def test_no_unified_membership_rejected(self):
        with self.assertRaises(RuntimeError):
            self.select('1:cpu:/GENERATED_ONLY\n')

    def test_multiple_unified_memberships_rejected(self):
        with self.assertRaises(RuntimeError):
            self.select('0::/\n0::/GENERATED_ONLY\n')

    def test_parent_traversal_rejected(self):
        with self.assertRaises(RuntimeError):
            self.select('0::/../GENERATED_ONLY\n')


if __name__=='__main__':
    unittest.main()
