# 生成 API 阶段诊断与同批三方实际结果

更新：2026-10-08。基于源提交 `b17109db4e6d73b9dddeef453618e44cacd160ff` 上尚未提交的六份精确脚本字节。只使用生成书源、生成账号和生成页面；没有读取真实正文或会话，不改生产。这不是本次新 Native JAR 的三方验收。

## 已从真实执行验证

固定原 JAR `b26fb476...`、历史恢复 JAR `221d41ef...` 及固定 runtime image `sha256:190e9712...`；旧 renderer 为 `hectorqin/remote-webview@sha256:b61d8e86...`。原 JAR＋旧远程、恢复 JAR＋旧远程、同一恢复 JAR＋内置 Camoufox 在同一次仅 loopback 的隔离环境顺序执行。旧远程先停止，再启动 Camoufox；不是把不同时期的两方样本拼接成第三方。

完整生成报告已取回，并独立重新执行原严格比较器：

- 每侧五次搜索的 HTTP 状态、完整 ReturnData 和 JSON 类型／默认值一致；旧脚本结果与目标 POST 字段保留。原件和恢复远程目标 Cookie 均为空，Camoufox 的预期序列仍为 `空 → session=alpha== → 空 → 空 → 空`。这是生成 Cookie，不是用户凭据。
- 每侧新增一次真实 `/reader3/getBookInfo`，恰好一次目标 GET；书源保存读回、固定等待脚本／诊断规则散列、八项页面布尔及完整 22 字段均检查。没有重放 POST、脚本或详情请求。
- 三个构造器默认系统时钟原值均保留，逐侧落在自己的实际 API 请求窗口；其余完整 JSON 精确相同。原始 ReturnData 并非逐字相同，不能省略这一差别。这个显式契约来自[原 JAR 的只读 javap 证据](GENERATED-METADATA-THREE-WAY-2026-10-07.md)，没有修改默认时间业务逻辑。
- 89 份实际资源采样，探针 94.819 秒；所有测试容器及其 Java／浏览器后代共同限制为 2 CPU／2 GiB／零 swap／512 tasks，未在渲染交接时重置累计计数。内存峰值 2,120,466,432 B（约 2022.2 MiB，距上限仅约 25.8 MiB），采样最大 tasks 238；OOM／memory.max／pids.max 事件均为零。临近上限是风险，不是宽裕容量、精确 PID 峰值或长期无泄漏证明。
- 主控 `reader-threeway-phase-b17109db-a.service` 终态 MainPID 0／inactive／success／退出 0。精确标签 `readerwebview0ba152ff19f2` 的自有容器独立查询剩余 0；两份 JAR 散列与六份输入散列未变。终态查询的 InvocationID 已为空，不把它冒充仍可读取的初始执行身份。旧 renderer 的受控停止退出码 137 同样保留，OOMKilled=false、累计 OOM 0；不隐去该值，也不据此宣称所有进程正常退出。

两次 SSH 取回中断只影响结果传输，没有重跑已完成的测试或改动原报告。恢复连接后只上传一个固定路径的清理审计 helper，再取回九份本轮生成文件。独立回执 SHA-256 `f2d2ed2eb32806d3d9f4cee2fdeef7883da32cff195a1f723c8e4b2f23917330`；[机器证据](evidence/generated-api-phase-three-way-2026-10-08.json)包含实际输入边界、请求窗口、预算、终态及九份原始文件散列。

## 新增可维护诊断，不伪造修复

此前 c 只有原件侧 TimeoutError，没有具体 API 阶段；不能回填为登录、详情或某个确定根因。本增量只给生成探针增加 18 个固定阶段白名单、最后已返回阶段与开始／完成次数，不保存参数、URL、账号、Cookie、密码或异常原文。回调只执行一次，不新增重试，不改变 HTTP／启动超时、请求窗口、输出或旧严格门禁。失败的 API 不能标作已完成；调用间的结果校验失败标作 betweenCalls。

七个新增测试在 Windows 和 WSL 各全执行通过。两项进入真实 `run_jar` 控制流但模拟进程与 API，分别确认 sourceRead 失败仍保留 5 开始／4 完成，以及详情失败保留五次搜索、详情尝试一次／未返回、15 开始／14 完成；它们不是 Java 或浏览器实跑。Windows Python 全套 330 项：329 通过、1 POSIX 专属跳过，20.724 秒。现有 GitHub 托管 workflow 的全量 discover 会执行新测试，其自身提交结果必须另验，不借用 b171 的旧绿灯。

```sh
python3 -B -m unittest discover -s src/test/python -p '*_test.py'
python3 -B -m unittest discover -s src/test/python -p 'generated_api_phase_test.py'
```

实际三方复现必须遵循上篇的 Linux 独立网络命名空间、已核验只读输入及全新输出目录命令，不能在 Windows 或用户 Reader 上裸启动原件。获准的 Ubuntu 官方 uidmap `1:4.17.4-2ubuntu3` 已安装；这一环境修复不等于本机最终非 root UID 10001 镜像验收。

## 仍未验证与回滚

早期 a 的字面时钟拒绝、b 权限失败、c 无阶段 timeout 原始记录均保留；新成功不证明旧 timeout 的唯一根因或把旧红灯改绿。严格 UTF-8 旧 44 B／正确 60 B 差异、历史 newContext/readiness 问题、起点匿名详情失败及真实认证三方仍未关闭。本轮没有执行 UTF-8 第六例，不以生成详情契约放宽它。

本轮使用历史 JAR／镜像，不是新 Native `b8cf8c1b...` 或普通 Full `5931436c...` 的实际三方；最终当前源码长期、真实用户关键流和生产验收仍需独立进行。58 份用户报告散列未变，未动日常 Reader。没有合并 draft PR、创建稳定标签、推送 registry 或部署。

回滚只撤销生成 API 阶段包装及新增测试，恢复原诊断形状；不涉及业务实现、数据库／用户数据、前端或字节码，不删除已有失败证据或只读 JAR。整个 goal 保持 active，不以这个局部门禁宣布完成。
