# 准确 f5c9 与授权单书：Worker 诊断仍不接受整轮

更新日期：2026-10-11。仅使用用户授权的《黎明之剑》一个 EPUB，或自行生成的 Worker 测试页；不读取其他本地正文、原书架进度、真实登录凭据或个人浏览器会话。不上传 EPUB、正文或真实阅读截图。

## 身份与证据分类

- **已从 JAR 验证**：原件 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`；恢复下载 `f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b`，285,671,807B。下载来自7739的Native运行38115254938；35587自己的双clean/Full/Native也独立证明相同f5字节，[托管记录](GUTENBERG-SNAPSHOT-THREE-WAY-2026-10-11.md#35587-快照工具自己的托管结果已独立接受)。不能据此把本次探针源码归到35587。
- **已从授权输入验证**：EPUB 37,809,796B，SHA `bd8c68194bb26a3c3bc5edc7b6baa7023c584441f1c2835f71fab3a3bf3ced31`。只读解析其1611项spine生成隔离元数据，不复制原用户书架。
- **已从维护源码推断**：一次全量物化EPUB、缓存生命周期、fflate内部扩容，以及另记的同record切书问题。源码推断不是压力失败唯一原因。
- **尚未验证**：最新完整Native镜像本机、Reader实际Worker完整生命周期、整轮合并预算成功、压力根因修复、生产容量。

本机使用旧锁定runtime镜像 `401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44` 的只读rootfs，绑定准确f5；不是本机运行35587的最新完整镜像。Camoufox/Playwright/browser分别锁定0.5.6/1.62.0/152.0.4-beta.30。

用户授权的Ubuntu官方uidmap已安装，版本 `1:4.17.4-2ubuntu3`、来源 `archive.ubuntu.com/ubuntu resolute/main`。subuid/subgid各自SHA仍为 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`，不修改原映射范围、系统代理或个人会话。

## 同轮有限结果，不拼接成一次成功

每轮父预算均包含客户端与Reader：2CPU/2GiB、memory.high1.5GiB、256PID、配置及观察swap0，UID/GID10001、零capabilities、no-new-privileges、仅lo、输入只读。它比普通托管“服务端预算、客户端另计”的门槛包含更多，不用普通CI替代。

| 轮次 | API与UI有限观察 | 本轮资源证据 | 整轮结论 |
| --- | --- | --- | --- |
| H | 原件/f5的1611目录与0/1/805/1610四章API对照一致；四个text/HTML取样，进入raw806明确15秒Worker超时 | 66.828秒，峰1,624,424,448B/PID256，PID触限1、soft high68044，hard-OOM0 | 拒绝；探针无实际Reader Worker记录 |
| I | 同API对照与四取样；raw806明确Worker超时；window构造器安装标记仍无接点 | 106.123秒，峰1,624,248,320B/PID256，PID触限1、soft high229602，hard-OOM0 | 拒绝；不能把空记录解释成Worker没启动 |
| J | 同API对照；生成真实页面module Worker自检123B输入→77B输出，created/posted/message/terminated四事件；仅text805取样 | 87.002秒，峰1,611,653,120B/PID245，PID/hard/OOM0、soft high3087 | 预算有限通过，但Reader文档marker丢失使诊断阻断；不是业务Worker超时证明 |
| K | 同API对照；六取样text805、text806、刷新806、HTML806、raw806、raw805；生成自检通过 | 外层超时、收尾inventory也超时，没有完整资源JSON；恢复inner记录460.853秒不是完整外层耗时 | 拒绝；raw805刷新与回text未完成，不能借H/J峰值填资源回执 |

H/I首次采样到PID触限的phase都是 `uiRefresh806Text`，早于切raw；0.5秒采样仅同步压力线索，不是因果证明。相应首次样本浏览器线程162/165、Java线程69/70，PID预算统计任务/线程，不等于256个独立浏览器进程。H/I稳态检查僵尸0，也不能据此证明整个生命周期永无僵尸。

K只恢复生成stdout JSONL，不读取私有日志/正文。恢复时已观察自有容器stopped/PID0，后续inventory为空；首次恢复在清理后因生成server行schema断言失败，保留该失败，再按正确schema恢复有限记录。[停止恢复回执](evidence/authorized-epub-f5c9-k-stopped-recovery-2026-10-11.json)，SHA `c7abdebce3b339dcf4f797e7965fc44c2b61e4f90205cf363ad446188191d59b`。

K异常处理中的checkpoint再抛TargetClosedError，首个业务异常被遮蔽，不能把此异常当成Reader唯一根因。79个点采样/10检查点不是最终资源报告：最后收尾样本僵尸7、父PID触限计数2、high259550；没有完整aggregate峰值或验收。实际注入文档计数只有fixture/login各1，未截获实际Reader Worker。

K raw805为3906字符/64段，SHA `8840e39870e34d782ad6d5f6130e8f2c6d76cb25a93b20b44eb08449edf33c0f`，不能与旧C/F/G的3952字符取样混认一致。不同画面/加载时机可能影响取样，但原因与完整可见内容一致性未证明；计数、SHA不包含原文。

[四轮独立有限核验](evidence/authorized-epub-f5c9-worker-diagnostic-2026-10-11.json)，SHA `366e59ad6a8f5b78fc0a0d6baebee576ea92f75c4e697cf92f6568a841d58495`。`rootCauseProven`、`productFixAccepted`、`fullGoalComplete`全部false。

## 探针边界与安全预算

H的全局评估、I的window评估都未截获Reader Worker。J/K仅向测试响应的HTML head插入真实页面脚本，保留产品main-world-eval设置、证书/隔离/预算和15秒Worker/60秒下载上限；生成module自检只能证明该生成测试页的包装链路。

当前可维护 `scripts/epub_worker_lifecycle_probe.py` SHA `308ca905f27ce811cafb6bf302feb57e2c192bf0bd7ab84a0f70910888830326`。它委托原构造器、消息参数与transfer列表、返回值/原异常和terminate，仅导出≤64事件的数值/布尔/时间/字节数量，不导出URL、名称、正文、Cookie或异常原文。新增DOM发布失败标记，观测失败不替代原生操作结果，但报告仍拒绝接受不完整观测。

这是测试响应变更及有限包装，不是“无扰动读取DOM”：构造器身份、消息派发时序、最多8192项计数和meta写入有开销。当前源码已比J/K实际执行版本增加publishFailed字段；J/K执行SHA `07f08f5bda185387810700352817f4616df727a243a21f8bf0990fcc6d65099d` 与旧schema原样保留，不追改为新工具版本。

本机17条纯生成探针回归通过，随后完整Python717项/30.945秒通过，1项既有Windows/POSIX平台跳过；无真实书/浏览器运行，不能代替实际Reader接点或后续提交的托管CI。

只读审计锁定fflate0.8.3：`Inflate.push`会在应用ondata预算检查之前临时扩容、复制输出。因此4KiB输入切片增至64KiB不是安全等价优化，暂不修改。当前中央目录预检及实际输出限制仍为compressed64MiB/expanded256MiB/entry32MiB/text4MiB/8192项；Worker输出buffer采用transfer，不把它误报成成功路径全书字节再次复制。当前raw→text保留当前书EpubDoc至卸载是会话缓存，不是已证实泄漏。内部临时工作集、下载累积与全量物化仍是主要风险。

## 清理、已知问题与回退

H/I/J/K各三份准确输入副本396,395,490B，共12份1,585,581,960B已按确切UID/长度/SHA/nlink只unlink；自有实例均停止、MainPID0且容器清理。来源、只读备份、f5下载、授权EPUB及58用户脏报告重新散列未变。临时副本可重建，没有删除用户数据或递归删除目录。[H清理](evidence/authorized-epub-f5c9-h-input-cleanup-2026-10-11.json)、[I清理](evidence/authorized-epub-f5c9-i-input-cleanup-2026-10-11.json)、[J清理](evidence/authorized-epub-f5c9-j-input-cleanup-2026-10-11.json)、[K清理](evidence/authorized-epub-f5c9-k-input-cleanup-2026-10-11.json)。

本轮不合并、打tag、发布、推送registry或部署，goal保持active。压力失败、原件完整HTTPS/WS/WSS/真实Cookie三方、OPDS未实现、最新整镜像本机与生产容量仍未闭合。回退只撤回测试探针/对应回归，不改原件、规则、安全或上限；保留失败原记录。

另发现的同record切书复用与进度归属候选，见[切书修复及自己的验证范围](READER-ROUTE-IDENTITY-2026-10-11.md)；它不是上述Worker超时的已证原因。
