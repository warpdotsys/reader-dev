import importlib.util
from pathlib import Path
import os, tempfile, unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('rollback', ROOT/'scripts/local-reader/verify-rollback.py')
R=importlib.util.module_from_spec(spec); spec.loader.exec_module(R)

class RollbackSafetyTests(unittest.TestCase):
 def setUp(self): self.temp=tempfile.TemporaryDirectory(); self.root=Path(self.temp.name)/'namespace'; self.root.mkdir()
 def tearDown(self): self.temp.cleanup()
 def test_inventory_and_copy_are_bounded_and_hash_preserving(self):
  nested=self.root/'reading'; nested.mkdir(); (nested/'book.json').write_bytes(b'generated'); target=Path(self.temp.name)/'snapshot'
  uid=self.root.stat().st_uid; manifest=R.copy_tree(self.root,target,chown=lambda *_:None,expected_uid=uid); self.assertEqual(1,len(manifest['files'])); self.assertEqual(manifest,R.tree(target,uid))
 def test_symlink_and_hardlink_are_rejected(self):
  target=self.root/'plain'; target.write_bytes(b'x'); (self.root/'link').symlink_to(target)
  with self.assertRaises(ValueError): R.tree(self.root,self.root.stat().st_uid)
  (self.root/'link').unlink(); os.link(target,self.root/'hard')
  with self.assertRaises(ValueError): R.tree(self.root,self.root.stat().st_uid)
 def test_book_url_cannot_escape_generated_namespace(self):
  username='rollback610abc123'
  self.assertEqual('storage/data/'+username+'/book/one.epub',R.generated_book_url('storage/data/'+username+'/book/one.epub',username))
  for unsafe in ('storage/data/'+username+'/../other.epub','storage/data/'+username+'/%2e%2e/other.epub','storage/data/other/book.epub'):
   with self.assertRaises(ValueError): R.generated_book_url(unsafe,username)
 def test_copy_rejects_symlinked_target_parent(self):
  (self.root/'book').write_bytes(b'generated')
  outside=Path(self.temp.name)/'outside'; outside.mkdir(); redirect=Path(self.temp.name)/'redirect'; redirect.symlink_to(outside,target_is_directory=True)
  with self.assertRaises(ValueError): R.copy_tree(self.root,redirect/'snapshot',chown=lambda *_:None,expected_uid=self.root.stat().st_uid)
 def test_seed_saves_preview_before_using_saved_url_for_toc(self):
  work=Path(self.temp.name)/'rollback'; secrets_dir=work/'secrets'; secrets_dir.mkdir(parents=True)
  pending=secrets_dir/'pending.json'; credentials=secrets_dir/'credentials.json'; receipt=secrets_dir/'receipt.json'
  generated_url='storage/data/rollback610abc123/book.epub'; calls=[]
  class Response:
   status=200
   def read(self,*_): return b'{"isSuccess":true,"data":[{"book":{"name":"Generated","author":"Test","bookUrl":"temporary://preview"}}]}'
   def __enter__(self): return self
   def __exit__(self,*_): return False
  class Client:
   def open(self,*_,**__): return Response()
  class Fixture:
   SMALL=object()
   @staticmethod
   def create(*_): return {'sha256':'fixture'}
  def fake_write(path,value):
   path.write_text('{}',encoding='utf-8'); os.chmod(path,0o600)
  def fake_request(_client,path,body=None):
   calls.append((path,body))
   if path=='/reader3/saveBook': return {'bookUrl':generated_url}
   if path.startswith('/reader3/getChapterList?url='): return [{},{},{}]
   return {}
  with patch.object(R,'service_state'), patch.object(R,'layout'), patch.object(R,'ensure_private_directory'), \
       patch.object(R,'pending_path',lambda:pending), patch.object(R,'credentials_path',lambda:credentials), \
       patch.object(R,'receipt_path',lambda:receipt), patch.object(R,'NAMESPACES',self.root), \
       patch.object(R,'ROOT',work), patch.object(R,'opener',lambda:Client()), patch.object(R,'fixture',lambda:Fixture), \
       patch.object(R,'multipart',lambda _:(b'fixture',{})), patch.object(R,'write_secret_json',fake_write), \
       patch.object(R,'safe_regular',lambda *_:None), patch.object(R,'request',fake_request), patch.object(R.secrets,'token_hex',lambda _:'abc123'), \
       patch.object(R.secrets,'token_urlsafe',lambda _:'password'):
   R.seed()
  paths=[item[0] for item in calls]
  self.assertLess(paths.index('/reader3/saveBook'),next(index for index,path in enumerate(paths) if path.startswith('/reader3/getChapterList?url=')))
  self.assertIn('/reader3/getChapterList?url=storage%2Fdata%2Frollback610abc123%2Fbook.epub',paths)
 def test_inactive_unit_is_required_for_stopped_phase(self):
  inactive=lambda *args,**kwargs:type('R',(),{'returncode':3,'stdout':b'inactive\n','stderr':b''})()
  active=lambda *args,**kwargs:type('R',(),{'returncode':0,'stdout':b'active\n','stderr':b''})()
  self.assertEqual('inactive',R.unit_state(inactive)); self.assertTrue(R.unit_active(active))
 def test_collected_unit_is_accepted_only_with_exact_dead_identity(self):
  def collected(command,**_):
   if command[1]=='is-active': return type('R',(),{'returncode':4,'stdout':b'inactive\n','stderr':b''})()
   return type('R',(),{'returncode':0,'stdout':b'LoadState=not-found\nActiveState=inactive\nSubState=dead\nMainPID=0\n','stderr':b''})()
  self.assertEqual('inactive',R.unit_state(collected))
  def nonzero_pid(command,**_):
   if command[1]=='is-active': return type('R',(),{'returncode':4,'stdout':b'inactive\n','stderr':b''})()
   return type('R',(),{'returncode':0,'stdout':b'LoadState=not-found\nActiveState=inactive\nSubState=dead\nMainPID=9\n','stderr':b''})()
  with self.assertRaises(ValueError): R.unit_state(nonzero_pid)
 def test_unknown_unit_state_is_rejected(self):
  unknown=lambda *args,**kwargs:type('R',(),{'returncode':3,'stdout':b'activating\n','stderr':b''})()
  with self.assertRaises(ValueError): R.unit_state(unknown)
 def test_default_cli_never_calls_phase(self):
  called=[]; old=R.PHASES['seed']; R.PHASES['seed']=lambda:called.append(True)
  try: self.assertEqual(0,R.main(['seed'])); self.assertEqual([],called)
  finally: R.PHASES['seed']=old
if __name__=='__main__': unittest.main()
