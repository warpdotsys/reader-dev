# 当前恢复候选的双架构持续运行验收

## 已实际验证的产物与结果

源码 `8d0d03c0c84882e90d16cde4b25dbf03d9ae9954`，受测合并快照 `3fe3777a5988b29e7e7e524d83d11eb733b3044d`。只消费成功 Native `37730974808` 的原字节产物，不重建镜像、不发版或部署。两架构的共同 JAR SHA-256 为 `591ffbce8aba9b3fd36d83c4581f0fdf64848d6a0af0b0860edf82f1e943dcec`。

| 托管运行 | 连续时长 | 轮次／成功搜索 | 峰值内存 | PID 峰值 |
| --- | --- | --- | --- | --- |
| [AMD64 37733156352](https://github.com/warpdotsys/reader-dev/actions/runs/37733156352) | 1805.322 秒 | 119／484 | 975,773,696 B，约 930.6 MiB | 217 |
| [ARM64 37733160269](https://github.com/warpdotsys/reader-dev/actions/runs/37733160269) | 1809.012 秒 | 125／508 | 1,000,312,832 B，约 954.0 MiB | 210 |

两次作业都在 GitHub 托管 runner 上实际完成。每次只有一项实际 soak 作业成功，其余六个互斥模式跳过，不能将跳过算作实测。测试以镜像 UID 10001、无外网、四个生成账号执行；逐轮四账号并发、交替 GET／POST，目标 Cookie 与请求形态错误均为 0。原 2 CPU／2 GiB／256 PID 预算未提高，swap 上限和实用均为 0，内存 max／OOM／OOM-kill 和 PID 触限均为 0。

三种故障与恢复也实际执行：导航超时 AMD64／ARM64 分别 7.542／7.251 秒，脚本 watchdog 25.026／25.025 秒，仅杀本轮自有 worker 的异常退出 2.176／2.052 秒。每次失败 ReturnData 和随后健康请求均被验证；冷／暖／每次恢复／每轮／最终静止样本中 worker、browser、driver 均为 0。

只下载两份小型制品，分别 5,715／5,788 B，共 14 个原始观察文件、244 行连续轨迹。加载前／后 image ID、运行 JAR、公开版本对象、源 Native 身份都逐字段核对；新独立比较器逐轮接受了这两份真实报告。大镜像与 JAR 没有再次下载到 Windows，没有压缩或读取书库。

完整身份、资源、故障、逐文件 SHA 与证据等级见[机器记录](evidence/browser-soak-8d0d03c0-2026-10-08.json)。正常短测和本次同字节构建见[JAR 复建记录](JAR-REPRODUCIBILITY-GATE-2026-10-08.md)。

## 已知限制与风险

- 本次是生成小页面的 30 分钟有限持续样本，不是无限时长、复杂真实站点、完整生产负载或吞吐承诺。
- 首个连续样本至最终的静止内存分别增长 57,647,104／22,114,304 B，约 55.0／21.1 MiB；冷请求后至最终增长约 84.6／51.6 MiB。没有将其归因为 JVM 堆、缓存或泄漏，也不能宣称无泄漏。
- 四账号突发的最长观测请求为 23.132／21.213 秒。完整 API 延迟包含调度与浏览器阶段，不能将配置中的单阶段超时当作全程延迟上限。
- 最终 `memory.stat` 仅是当时的重叠类别，不可相加当作峰值 RSS 分解或事后推断泄漏原因。
- 原始两次运行的 `CONTAINER_STATE.json` 是删除前健康状态；未采集删除后宿主清单。保留“进程静止时已回收”与“宿主删除已独立证明”的区别。
- 起点匿名详情仍为空；真实认证、当前最终镜像的完整三方与全部格式／界面边界未完成。旧 UTF-8 POST 行为差异继续在[特征记录](WEBVIEW-UTF8-CHARACTERIZATION-2026-10-07.md)中保留，不能用本次 ASCII 生成 POST 消除它。
- 原 JAR、日常 Reader、真实会话、私有正文和生产服务未修改；PR 仍为候选，不能把本表当作合并或上线证明。

## 可维护的逐轮与清理验收增量

旧 shell 门禁只检查摘要。新增 `scripts/verify-bundled-browser-soak.py`：检查全部连续轮次、GET／POST 交替、时序、末五样本、成功请求和目标计数、三类故障恢复、全部静止进程／累积峰值、原零 swap 预算，以及源 Native／加载前后／运行中的 JAR 与镜像身份。重复 JSON 键、非有限数、缺失字段、真假类型混用、超大输入与异常文件严格拒绝，不输出原始响应或路径。尾轮实际时长必须达到要求；两个不同位置的真实时钟读数保留原值，只允许最终落盘／关闭的 0.01 秒有限开销，不能靠请求结束后的空等补足时长。

新 harness 还按本次自有完整容器 ID 删除并独立查询宿主清单，保留 `CONTAINER_REMOVAL.json`；只有删除成功、清单读取成功且自有容器剩余 0 才能通过最终验收。新 `POST_REMOVAL_VERIFIED_SOAK.json` 与删除前回执分开保存，不覆盖历史观察。

20 项定向测试在 Windows／WSL 全执行通过，Windows 全套 382 项中只有一项平台跳过，发布相关 Node 82 项全通过，shell 语法通过。早期缺少接入的实际断言失败保留；本轮 72 个报告文件哈希核对未变，不覆盖用户未提交数据。上述两次 30 分钟运行使用旧 harness；新比较器是在下载后独立核对，**不伪称当时执行了新增清理门禁**。新清理路径仍须自己的托管实际运行。

可对 AMD64 原始小报告独立复验：

```powershell
python scripts/verify-bundled-browser-soak.py build/native-soak-8d0d03c0-20261008-a/amd64 --architecture amd64 --expected-revision 3fe3777a5988b29e7e7e524d83d11eb733b3044d --expected-source-revision 8d0d03c0c84882e90d16cde4b25dbf03d9ae9954 --expected-native-run 37730974808 --expected-jar 591ffbce8aba9b3fd36d83c4581f0fdf64848d6a0af0b0860edf82f1e943dcec --expected-image sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44 --seconds 1800 --require-container-state
```

该旧报告没有删除后观察；不能为它加 `--require-removal` 后编造通过。输入只读，比较器不会启动 Reader 或重放请求。

## 回滚

本增量仅改变验收工具，不改业务实现、依赖、原始 JAR、镜像层或生产配置。停止消费新工具即可退回旧比较流程；旧报告和失败记录继续保留。若新增门禁失败，应保留原产物／报告并定位，不提高资源上限、不放宽 Cookie／JSON 契约，不创建正式发布标签。
