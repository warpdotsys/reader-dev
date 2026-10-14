"""Passive same-origin file timing only: no body, query, cookies, path or error text.

Use request/response/finished/failed events and Request.timing/Request.sizes
from https://playwright.dev/python/docs/api/class-request . Never route,
intercept, replay, inject JS, or change the production 60-second deadline.
"""
import math
import time
from urllib.parse import urlsplit


class FileNetworkObservation:
    def __init__(self,base,observation):
        parsed=urlsplit(base)
        assert parsed.scheme=='http' and parsed.hostname=='127.0.0.1' and parsed.port
        self.origin=(parsed.scheme,parsed.netloc)
        self.start=time.monotonic();self.observation=observation
        self.ids={};self.events=[];self.errors=[];self.total=0
        observation['passiveFileTransferEvents']=self.events
        observation['passiveFileTransferErrorTypes']=self.errors
        observation['fileTransferBodiesHeadersQueriesCaptured']=False

    def kind(self,request):
        p=urlsplit(request.url)
        if (p.scheme,p.netloc)!=self.origin:return None
        return {'/reader3/file/list':'selected-path-check','/reader3/file/download':'selected-file-download'}.get(p.path)

    def observe(self,request,event,response=None):
        try:
            kind=self.kind(request)
            if kind is None:return
            self.total+=1;self.observation['passiveFileTransferEventCount']=self.total
            if len(self.events)>=32:
                self.observation['passiveFileTransferEventsTruncated']=True
                return
            identity=id(request)
            if identity not in self.ids:
                assert len(self.ids)<16
                self.ids[identity]=len(self.ids)+1
            row=dict(requestId=self.ids[identity],kind=kind,event=event,
                phase=self.observation.get('activePhase'),milliseconds=round((time.monotonic()-self.start)*1000))
            if response is not None:row['httpStatus']=response.status
            if event in ('finished','failed'):
                values=request.timing
                keys=('domainLookupStart','domainLookupEnd','connectStart','secureConnectionStart','connectEnd','requestStart','responseStart','responseEnd')
                row['relativeTimingMs']={k:values[k] for k in keys if k in values and type(values[k]) in (int,float) and math.isfinite(values[k])}
            if event=='finished':
                sizes=request.sizes()
                row['byteCounts']={k:sizes[k] for k in ('requestBodySize','requestHeadersSize','responseBodySize','responseHeadersSize') if k in sizes and type(sizes[k]) is int and sizes[k]>=0}
            self.events.append(row)
        except Exception as error:
            # Diagnostics never replace the primary UI result or disclose an
            # exception message (which could contain URLs/credentials/names).
            if len(self.errors)<4:self.errors.append(type(error).__name__)

    def attach(self,page):
        page.on('request',lambda r:self.observe(r,'request'))
        page.on('response',lambda r:self.observe(r.request,'response',r))
        page.on('requestfinished',lambda r:self.observe(r,'finished'))
        page.on('requestfailed',lambda r:self.observe(r,'failed'))
