# 固定书源 SSE 生命周期与双源分页差分

验证日期：2026-09-23（Asia/Shanghai）。

原始 JAR 与恢复版分别在本机随机端口、独立临时工作目录运行；固定测试站点只监听 `127.0.0.1`。每端使用脚本生成的账号，先测一个书源，再加入第二个书源。不读取生产数据，也不调用真实第三方网站。脚本在每次测试结束后停止 Java 与测试站点进程并删除临时目录。

```powershell
pwsh -NoProfile -File .\scripts\build.ps1
python -B .\scripts\compare-sse-pagination.py
```

原始 JAR SHA-256：`B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`；本次干净重建的恢复版 SHA-256：`CD81FC945928E9E558BCEED6970AD152E500BF0CA92128ECE5C34E4FEC3B3B89`。逐项结果见 `reports/sse-pagination-diff-latest.json`。

## 已从 JAR 验证并与恢复版比对

九个探针中，匿名拒绝、无书源、缺少关键字、未知书籍、双源第一页、游标耗尽这六项响应一致。两个正常 SSE 接口均收到数据帧和完整的 `event: end` 终帧；错误路径均收到单个完整的 `event: error` 帧。脚本要求严格 UTF-8、`text/event-stream`、帧尾空行、终止帧、预期错误文案、每页结果数、游标和结果摘要。另用两段模拟截断流验证：缺失终帧或终帧尚未写完时，解析器必须报错，不能把提前 EOF 记为成功。

双源分页使用 `searchSize=1`、`concurrentCount=1`：第一页返回 1 条、`lastIndex=0`、`isEnd=false`；第二页从游标 0 续取，返回另 1 条、`lastIndex=1`。同一个固定书源加入第二源后，第一页结果摘要保持不变。正常搜索与换源搜索都由真实测试站点规则解析产生结果，而非伪造空实现。

## 已审查的三项差异

单源搜索、单源换源、双源第二页均已扫描到最后一个书源。原始 JAR 在三处终帧都返回 `isEnd=false`；恢复版返回 `isEnd=true`。除这个布尔值外，状态、响应头、帧数、结果数、结果摘要及游标一致。原版的 `false` 与随后请求返回“没有更多了”矛盾，因此保留恢复版的游标修复。差分脚本只接受这三处精确差异；其它差异会失败。

修改测试站点以响应完整书名查询后，原有固定书源阅读差分脚本也已复跑通过，其报告 `reports/web-source-reading-diff-latest.json` 同步更新。

## 尚未验证

- `concurrentCount>1` 时的乱序完成、客户端主动断开、服务器中途异常与长连接稳定性。
- 生产容器的带登录态 SSE；测试没有向生产服务发送认证请求。
- `bookSourceDebugSSE`、`cacheBookSSE`、POST 变体及大量书源分页。
