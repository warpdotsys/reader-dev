# Python 3.10 哈希兼容增量的独立托管验收

日期：2026-10-11。源码提交 8b250346d5bc4e3bca142da0eaf75ae6bc6151c6，实际 PR 合并快照 ccbdad907fb803ca4f75a38d182cb86009372dc9。下面是本提交自己的终态作业、下载报告与独立回执，不借父提交的绿灯。不代表合并、正式发版、registry 发布或生产部署。

## 已成功重建及实际执行

| GitHub 托管流程 | 本提交实际结果 | 独立回执 |
| --- | --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38111391705) | 627 Linux Python，零跳过；201 JVM / 35 既有环境跳过；TTS 编辑 11、日志 9、代理 4 项全执行；两次干净构建字节相同 | [Java](evidence/hash-compat-hosted-8b250346-independent-java-acceptance.json) |
| [Vue 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/38111391715) | 301 前端测试、25 实际生成 UI 场景零失败/跳过、11 张合成图的身份回执；不声称本轮已逐图目视 | [UI](evidence/hash-compat-hosted-8b250346-independent-ui-java-acceptance-v2.json) |
| [完整单镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38111391712) | Camoufox 24 契约零失败/跳过；实际打包 Reader HTTPS、默认完整 UI 和源辅助接口门槛接受；峰值 832,548,864 B / PID 207 | [Full](evidence/hash-compat-hosted-8b250346-independent-full-acceptance.json) |
| [原生双架构与导入](https://github.com/warpdotsys/reader-dev/actions/runs/38111391710) | 六作业终态成功；AMD64/ARM64 原生构建和重导入实际运行；publisher 的 32 份报告逐字段接受；最大峰值 877,780,992 B / PID 198 | [Native](evidence/hash-compat-hosted-8b250346-independent-native-acceptance.json) |

Runner 为 GitHub 托管 ubuntu-24.04 / ubuntu-24.04-arm，不使用自托管 runner。Full 的七个可选归档、三方或长测作业是 skipped，不计入本次已执行结果。

双干净 JAR 均为 9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84，285,672,062 B / 1570 条目；完整 ZIP 注释、条目顺序及四类差异计数核对相同。a5cd 下载副本与本轮重建字节相同，但其下载来源仍是 a5cd/2147，不改写历史来源。

AMD64 镜像 ID：sha256:aa3b64d75108514eb9d1a1ee49dc97f80ba42e447aa7de008b6c5f634f47f8fc。

ARM64 镜像 ID：sha256:5051f3de3bc39ab81ce7d7f6352509a51ba14869d6556c786d52e2ce08bab3eb。

回执 SHA-256：Java fe0f0708122b3107d830620770d4380cea69ba4addb01a89c9ba2292026550f6；UI fe50204294cc7dd941619be6f3128b5e56a117d0a3c192af524362c5a65607ad；Full 9e11402f19482b3d3c13e0fca18609755c77e46c55dd0574244e9f79f188fcf8；Native c467cd98c43e11365f0861889974b3cfc0401d62a6e813bba3df836e29740903。

## 范围、已知问题与回退

维护脚本使用完整原字节、1 MiB 分块 SHA-256，兼容 Python 3.10；新增五项回归。本增量没有改变产品 JAR。普通 Full 允许 1 GiB swap、观察为 0，不改称配置禁止 swap；测试 UI 客户端位于服务端预算之外。

只下载本轮有限测试/身份报告，未下载大型完整镜像；镜像归档摘要保留托管报告值，不冒充本机重新计算完整归档 SHA。当前最新整镜像的本机启动仍未验收。原同源认证头丢失、生成三方首搜索超时及真实书 C/D 合并预算失败均保留；本次未复现不等于根因修复。

[真实授权 EPUB 增量](AUTHORIZED-EPUB-9B9D-2026-10-11.md)单独区分原件 API、合并预算失败和正常外部客户端成功；不把合成 CI 旅程冒充真实书，也不把宿主 Chromium 客户端视为内置后端引擎证明。真实站点认证、准确原件完整 HTTPS/WS/WSS、OPDS 和生产迁移仍未完整验收。

默认分支仍为 legacy，PR56 仍 OPEN/draft/base legacy；58 份受保护用户报告未变。不发布或部署失败轮次。回退只撤回自有测试工具改动，保留原件、失败证据和发布守卫，不 reset 工作区或删除真实 storage。
