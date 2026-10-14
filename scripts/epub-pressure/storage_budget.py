"""Metadata-only monitor of the exact generated disk volume; never read body, names into logs, or other storage."""
import os
from pathlib import Path
import signal
import stat
import threading
ROOT=Path('/verification-data')
LIMIT=536870912

class StorageBudget:
    def __init__(self,observation):
        info=ROOT.stat()
        assert ROOT.resolve()==ROOT and not ROOT.is_symlink() and info.st_uid==info.st_gid==10001
        self.device=info.st_dev;self.stop_event=threading.Event();self.observation=observation
        self.summary={'budgetBytes':LIMIT,'maximumBytesObserved':0,'maximumFilesObserved':0,
            'samples':0,'budgetExceeded':False,'metadataReadErrors':[],'fileBodiesRead':False,'namesPublished':False}
        observation['generatedDiskStorageBudget']=self.summary
        self.thread=threading.Thread(target=self.collect,name='own-storage-metadata-budget',daemon=True)

    def sample(self):
        total=files=0
        for directory,dirs,names in os.walk(ROOT,followlinks=False):
            parent=Path(directory);info=parent.lstat()
            assert stat.S_ISDIR(info.st_mode) and info.st_dev==self.device and info.st_uid==10001
            assert parent.resolve().is_relative_to(ROOT)
            for name in dirs:
                child=(parent/name).lstat();assert stat.S_ISDIR(child.st_mode) and not stat.S_ISLNK(child.st_mode)
            for name in names:
                path=parent/name
                try:info=path.lstat()
                except FileNotFoundError:continue
                assert stat.S_ISREG(info.st_mode) and info.st_dev==self.device and info.st_uid==10001 and info.st_nlink==1
                total+=info.st_size;files+=1
                assert files<=8192
        self.summary['samples']+=1
        self.summary['maximumBytesObserved']=max(total,self.summary['maximumBytesObserved'])
        self.summary['maximumFilesObserved']=max(files,self.summary['maximumFilesObserved'])
        if total>LIMIT:
            self.summary['budgetExceeded']=True
            os.kill(os.getpid(),signal.SIGTERM)

    def collect(self):
        try:
            while not self.stop_event.is_set():
                self.sample();self.stop_event.wait(1)
        except Exception as error:
            if len(self.summary['metadataReadErrors'])<4:self.summary['metadataReadErrors'].append(type(error).__name__)
            os.kill(os.getpid(),signal.SIGTERM)

    def start(self):self.thread.start()
    def stop(self):
        self.stop_event.set();self.thread.join(timeout=3)
        self.summary['monitorStopped']=not self.thread.is_alive()
        if not self.thread.is_alive():self.sample()
