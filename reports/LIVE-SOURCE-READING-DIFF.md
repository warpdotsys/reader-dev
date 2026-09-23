# 线上现有真实书源差分

验证日期：2026-09-23（Asia/Shanghai）。经只读 SSH，从已部署服务的 `storage/data/<账户目录>/bookSource.json` 中仅提取一个无需登录配置的书源规则至测试进程内存；未导出整份书源库。原始 JAR 和当前恢复构建分别在本机临时目录启动，以合成账号调用同一真实书源。测试没有修改生产书源、书架或用户数据，也没有把章节正文或书源规则写入报告。

运行（需预先配置对服务器的只读 SSH 访问）：

```powershell
python -B .\scripts\compare-live-source-reading.py --source-namespace YOUR_NAMESPACE --source-index 98 --query 斗破苍穹 --sse --production-smoke --quiet --report .\reports\live-source-reading-diff-latest.json
```

## 已从原始 JAR 验证并成功重建

“快眼看书（优+）”书源搜索返回 100 条；对同一本书，两版详情成功、目录各 1914 章、首章正文各 2852 个 Unicode 字符。搜索、详情、目录的 HTTP 状态、`isSuccess`、`errorMsg`、结果数量及首条对象的字段名/默认值类别一致；全部搜索书目、详情关键字段及全部章节的名称/URL/索引摘要亦一致。正文 UTF-8 SHA-256 在两版均为 `27D041EDEDD93E919F4D3DE969A0E13061E40001C84DA51BDCE12068348B45E1`。正文只计算长度和哈希，不保存或展示文本。

两种 SSE 接口都验证到真正的 `event: end` 终帧（各 2 帧），没有把连接建立或提前 EOF 算作完成。多源搜索 SSE 返回 41 条，换源 SSE 返回 1 条；两版每个流的数据数量、字段形态和书目摘要一致。

## 有意修复的 SSE 终帧差异

仅配置一个书源时，原始 JAR 的两种 SSE 终帧均是 `lastIndex=0, isEnd=false`；这与服务端下一次请求的“没有更多了”判断相矛盾。恢复版将终帧修正为 `lastIndex=0, isEnd=true`，其它帧字段及结果一致。零基索引判断另有单元测试；黑盒脚本只接受这两个接口中恰好这一布尔值的差异，出现其它差异会失败。本机恢复构建 SHA-256：`2CE4C1ABA8B40471A036C266F7E4ECFC586EB91C83878CB246194B21C25DA8EF`；此哈希是发布前本机构建，不是 GitHub 发布制品哈希。

## 生产 API 的发布前后复核

发布前对当时已部署的 `https://read.medwarp.cn` 发起匿名、只读的搜索与详情请求：两项均为 HTTP 200、`isSuccess=true`，搜索 100 条且全体书目摘要与本地一致，详情含目录地址；搜索响应当时多出内部 `_userNameSpace` 字段。

`v4.0.7-restored.8` 随后由 GitHub 托管 runner 部署；服务器标记和当前 JAR 的 SHA-256 均为 `D2AA722284965C7B05544C98C49B509B2E52F37CC0487FB860FFC75D7B53D780`，容器为 `running/healthy`。再次匿名访问同一真实书源：搜索与详情均 HTTP 200，搜索 100 条且全体书目摘要与本地一致，搜索 JSON 不再含 `_userNameSpace`，首条结果字段形态与本地恢复版一致。生产容器本体的登录态目录/正文未调用；受限隔离副本的结果见 `reports/STAGED-PRODUCTION-DATA-READING.md`。

## 真实书源自身的失败样本

- “阅读助手”搜索 10 条且详情成功，但所试两本书在原始 JAR 和恢复版均报 `TocEmptyException: 目录为空`。
- “天天看书”两版搜索均在书源的 Rhino 脚本规则处报相同异常。
- “八零小说”“有度中文”“鲸云轻说”对所试查询两版均返回空搜索结果；“熊猫文学”两版均遇到过多重定向。

这些现象属于同源同输入下共同失败，不能据此判定恢复版回归，也不能据此断言对应书源整体不可用。真实站点结果可能随时间、地区和反爬策略变化；本报告仅记录本轮快照。

## 尚未验证

- 生产容器本体的登录态目录、正文、书架以及缓存链路；现有 accessToken 自动登录路径会写入 `users.json` 的最后登录时间，故不将其包装成纯只读验证。隔离副本已完成其中一条书源的认证与阅读链路。
- 真实书源的其他书目、翻页、JS 规则全集、封面图片、SSE 多源并发及长期稳定性。固定双源的顺序分页差分另见 `reports/SSE-PAGINATION-DIFF.md`，不能代替多源并发测试。
