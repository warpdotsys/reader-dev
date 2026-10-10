# 六例三方实际观察与历史 UTF-8 已知缺陷

## 结果和证据等级

**已从原 JAR 实际验证**：同一私有 only-loopback 网络中，原件／恢复版远程路径／同恢复 JAR 的内置 Camoufox 各实际完成 6 次搜索及 6 次目标请求。六份完整生成 `ReturnData`、JSON 字段与类型、状态和脚本书名三侧一致。前五例 GET／POST 请求符合原规则；目标 Cookie 分别为历史两侧全空，Camoufox `空 → session=alpha== → 空` 后保持空。

**已从固定历史引擎实际验证的有意差异**：第六例历史两侧各发送 44 B，原字节 SHA-256 `c61ca3e561f770d4fc8551da0bddb98afd0be801801502504a73221ed11ec5d9`；该生成 JSON 被截断，不能正确解析。内置 Camoufox 实际发送完整 60 B／`8d0383070028f4f457194a194e8e9133df2487261e341c1da121171d59eeeef1`，并保留中文、补充平面字符及脚本结果。合成目标故意仍返回 HTML，所以 Reader 搜索成功不证明旧请求体正确。

**不是六例完全等价或正式交付**：诊断模式实际非零退出 1，`observationsComplete=true`，`strictSixCaseParityAccepted=false`、`historicalUtf8BodyCorrect=false`，外层 `overallAccepted=false`。独立复核默认严格 validator 仍在历史请求字段处拒绝本报告，既没有改历史引擎，也没有截断正确请求追随旧缺陷。这个记录关闭“原返回没保留／三侧第六例从未实际收齐”的观察缺口，不关闭严格兼容绿灯、历史超时根因或真实认证三方门槛。

[公开证据元数据](evidence/webview-utf8-characterization-2026-10-07.json)仅包含生成结果摘要、实际资源和私有原报告哈希；原日志与完整生成报告只在授权临时目录及本机 ignored 隔离目录保留，不发布原日志或真实凭据。

## 工具及安全边界

只新增显式 `--characterize-historical-utf8` 诊断开关及负向测试，不修改业务源码、worker、UI、存储或镜像。默认模式继续要求历史和当前三侧第六例都发送正确的 60 B。诊断模式限定：

- 必须同时启用原件隔离、真实历史引擎、脚本、POST、UTF-8 六例及 host 协调 handoff；不能使用模拟 renderer 或省略交接。
- 只有两历史侧都符合已独立实测的精确 44 B 原文／摘要／头，以及前五例原规则、六份完整 JSON 和 Cookie 规则时才发完成标记；任何新差异均拒绝。
- 仍由 host 确认累计预算、停止仅本轮历史容器、重新验证 netns、8050 拒绝连接后才执行 Camoufox；当前侧仍必须正确 60 B，不容许沿用截断。
- 原报告不规范化或改写；只计算与实际完整 JSON 相对应的摘要。诊断即使收齐全部观察，也必定非零退出；外层连异常零退出也拒绝，不能产生 `overallAccepted=true`。

G 轮未使用浏览器 API wrapper 或额外 DEBUG 环境；固定历史镜像和 `index.js` 不改。原件 `b26fb476...`、恢复 JAR `221d41ef...` 沿用授权临时目录只读副本。运行时 `190e9712...` 是既存锁定 UID 10001 amd64 镜像，不执行其自带旧 JAR；这是指定恢复 JAR／worker 加旧锁定运行时的实测，不冒充当前最终完整镜像。历史容器自身 UID 0／全部 capabilities 删除，不能冒充最终非 root Reader 的接受证据。

## 实际资源和清理

服务 `reader-threeway-utf8-characterization-20261007-g.service`，Invocation ID `370c33b64a6544cfb1a63749ba3f47bf`；终态 MainPID 0／exit 1。两只容器及所有 Java／浏览器后代合计 2 CPU／2 GiB／零 swap／PID 512，单容器 PID 256；原预算未增大、重置或扣缓存。

91 个采样，末次 97.341 秒，内存峰值 1,674,854,400 B、采样 PIDs 峰值 237。内存 max／OOM／oom_kill 和 PID max 计数 0；`sock_throttled=4` 原样保留，不能说全部事件为零，也未据此推断业务故障或唯一原因。严格 harness 因非零诊断保持 `budgetAccepted=false`，另由独立审计核对原预算和原触限守卫；这两种字段不混用。

两只自有随机 ownership 容器已移除，宿主随后按确切本轮标签独立查询剩余 0；两个 JAR 散列不变，58 份用户已有报告散列不变。没有公网出口、端口发布、生产挂载、真实 Cookie／账号／正文、新 JAR 上传或生产部署。

## 可重复命令

Linux Docker/systemd root 控制端，预先准备固定镜像与只读输入，在授权临时根下选择全新输出目录；这是诊断命令，**预期 exit 1，不要加 `|| true` 将其归为接受成功**：

```sh
python3 -B scripts/run-three-way-webview-in-docker.py \
  --original /absolute/read-only/reader-pro-3.2.14.original.jar \
  --restored /absolute/read-only/reader-pro-v4.0.7.jar \
  --runtime-image sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf \
  --output /var/tmp/NEW-generated-only-run \
  --exercise-encoding --characterize-historical-utf8
```

去掉最后的诊断开关即恢复原严格六例模式；它仍会拒绝历史截断。两个原 JAR 的服务器输入位置及可用镜像必须先独立核对，不应在其他机器照抄旧产物身份。

本增量纯测试：Windows Python 271 项中 270 通过／1 POSIX 专用跳过、0 失败；WSL 定向 45＋20 项无跳过／失败。WSL 全量 271 项另有 2 项因缺少 Node 失败，不是全量通过，不跳过或安装额外系统包取绿灯。Windows 固定 Node 执行发布结构／身份／manifest 共 69 项通过。以上均不作为真实浏览器证据；本节真实三方来自 G 实际报告。本提交自身托管结果已独立验收，见下一节；前一提交绿灯不替代。

## 本诊断增量自身的常规托管验收

源码 `f88fdd819c029f4e794bcc5c8e059aece0fcd2b5`／受测 `39248993ee1104ed8faa43b099d044d744d246e8` 的 [Java](https://github.com/warpdotsys/reader-dev/actions/runs/37607041030)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37607040979)、[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37607040978)、[原生演练](https://github.com/warpdotsys/reader-dev/actions/runs/37607040997)均终态 success，原生六个实际作业全部成功。Linux 自带 Node 的全量 Python 271／8.742 秒，零失败／错误／跳过；它不抹去 WSL 缺 Node 的失败。

常规 Full 的五个可选专项未执行，不算实站／原件／长期接受；两套 20 项来自 Full 与共享 JAR 的真实合约任务，不声称 ARM64 镜像各跑了全部 20 项。两架构镜像的实际生成 UI／异步／资源及转移后重载接受分别来自上述原生作业。

下载小报告独立重跑当前门禁，两套真实 Camoufox 各 20／零跳过／失败／错误，第 20 项实际 3.078／2.967 秒，helper 各 3。publisher 精确 24 JSON，五份生成 UI／异步／预算的 15 次原守卫通过，最高 901,910,528 B／PID 199、内存 max／OOM／PID max 和实际 swap 0；常规配置允许 1 GiB swap，与 G／匿名专项的零 swap 不混。原生 JAR `99a83f8c...`、Full `18d8c275...` 不混作 G 的 221 或匿名专项的 3816 输入；未将巨型镜像归档下载到本机重新验字节，未额外目视本提交的新截图。[自己的报告散列和范围](evidence/webview-utf8-characterization-hosted-f88fdd81-2026-10-07.json)。

这不是实站成功：[另外的一次匿名起点专项](PUBLIC-METADATA-IMAGE-2026-10-06.md)消费此前已验收、产品源码相同的 `d94eb587` 原镜像，元数据业务仍失败。常规绿灯、G 的完整生成观察和这一实站红灯分别保留，不拼接为真实认证或生产验收。

## 已知问题与回退

旧引擎 UTF-8 截断是实际缺陷；a／b／d／e 的搜索超时及 f 的 readiness 前置失败仍未唯一归因，G 一次收齐不能追认这些失败为已解决。真实起点解析／认证三方、全部非 UTF-8 编码、当前最终镜像长期容量和生产验收仍未完成，PR 继续 draft，不发布正式标签。

回退只在干净检出撤销本诊断工具／测试增量或不传诊断开关；业务数据与引擎无需变更。保留原 JAR、完整失败与诊断证据，不 reset 用户工作区，不覆盖旧报告，不把诊断开关带进正常发布接受命令。
