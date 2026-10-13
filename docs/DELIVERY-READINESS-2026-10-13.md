# 正式发布就绪核查

主线／发布配置审计基线为`c703abaad98bf492a00d17d436e4b5e61a31dd44`，检查时间2026-10-13。随后用户批准统一版本v6.1.0，当前源码候选为`f6e61bf3be91ade4e66c4c22fe76bf2a1fd6ed41`；本候选自己的四条托管流程已独立接受，[精准收据](evidence/candidate-f6e61bf3-hosted-acceptance-2026-10-13.json)与[候选发布说明](releases/v6.1.0.md)已同步。不创建标签、合并PR、修改Secrets、登录registry或部署生产。c703的历史证据不移借为新版本验收。用户未提交的报告仍只核对哈希，不读取、暂存或覆盖。

## 已从当前仓库和GitHub状态验证

- 仓库默认分支legacy。配置审计时PR56为OPEN／draft、base=legacy、head=c703、mergeStateStatus=CLEAN；随后代码候选更新为f6e61bf3。可合并状态不等于已完成审查或已合并，[当前PR](https://github.com/warpdotsys/reader-dev/pull/56)仍未合入主线。
- c703历史四流程38383739279／38383739183／38383739189／38383739203保持原身份；当前f6自己的四条流程见下文及[精准本候选收据](evidence/candidate-f6e61bf3-hosted-acceptance-2026-10-13.json)。不将跳过作业或重复测试叠加成新覆盖。
- 审计时工程清单为`4.0.7`，旧前端锁文件根包仍为`4.0.5`。用户随后选定`v6.1.0`；当前后端、备用CLI、两套前端及其锁文件根包已经同步为`6.1.0`，依赖节点版本不改，新增只读同步守卫。仓库最新正式Release为`v6.0.45`，非draft／非prerelease，targetCommitish=master，发布日期2026-08-24。[该正式版](https://github.com/warpdotsys/reader-dev/releases/tag/v6.0.45)。发布工作流检查仓库全局最新正式稳定版本，不以当前分支重置排序；4.0.7及文档旧例4.1.0仍会被拒绝。6.1.0尚未打标签或发布，不猜测旧版的语言、源码一致性或数据兼容性。
- 仓库级变量名称`GHCR_IMAGE`、`DOCKERHUB_IMAGE`存在，仓库级Secret名称`DOCKERHUB_USERNAME`、`DOCKERHUB_PASSWORD`存在。`production`环境存在，四个`PROD_SSH_HOST/USER/KEY/KNOWN_HOSTS`环境Secret名称齐备，部署job明确选择该环境。因此仓库级Secret清单中没有生产名称并不构成缺失证据。仅确认配置名称，不读取Secret值，不将名称齐备当作认证或端到端部署成功。
- 正式流水线只接受稳定SemVer、三处同步版本、带“已知问题”章节的同名Markdown、legacy祖先提交，并要求版本高于当时最新正式Release。它复用完整Vue旅程及双原生Native门禁；正式publisher先校验并导入已经受测的双镜像，再登录registry、推送同镜像及多架构manifest，不重新构建。
- 完整镜像Dockerfile携带固定浏览器／Python／JRE及依赖，USER10001:10001，默认Camoufox及Vue3；Compose只发布回环18088，保留Vue2回退、cap_drop=ALL及no-new-privileges。普通JAR的旧默认与完整镜像默认不同。测试预算不是生产Compose限额证明，Compose当前没有CPU／内存硬限配置。
- `production`环境当前API的deployment_branch_policy为null、protection_rules为空：未看到环境级分支／标签限制、等待或人工审批保护。后续branch-policy列表API的404与这个“未配置策略”的直接结果分别记录，不当作凭据失效。环境名称齐备仍不代表部署权限已最小化；本轮不擅自更改这些保护或创建新SSH授权。

## 已验证与仍未闭合的边界

准确548a/87b归档已完成唯一获准单书八场景、原版生成账号存储读取、公开Gutenberg默认UI、双架构有限长测、生成原件三方和真实HTTPS公开书三方。c703共享JAR与该归档内16f相同，但完整镜像身份不同；不因此宣称c703或未来改版镜像做了完全相同的整镜像验收。

当前明确兼容变化包括历史UTF8请求缺陷与当前正确编码、历史空Cookie／当前重放清除、默认时钟非字面相等、历史忽略TLS证书错误／当前严格，以及隔离代理接入差异。真实HTTPS三方只证明固定匿名公开书；不是已清起点登录态、验证码后的认证、任意站点或原生产远程实例同版证明。

新增公开站点脚本／生成Cookie预检两轮均失败，第二次实际HTTP504，第一轮状态码未捕获；零Reader启动／零业务调用，不能算实站脚本或Cookie通过。两轮自有容器、socket及父级隔离组已独立清理。用户新批准的Chrome起点登录态仍只限本机；浏览器工具因无法确认当前网址中止，未读取或导出Cookie，不改变服务器唯一公开站点授权。

首个6.1.0候选f1bcfaab实际下载JAR与双clean的66d1摘要相符，但MANIFEST缺少Implementation-Version；[实物验收失败](evidence/candidate-f1bcfaab-jar-version-rejection-2026-10-13.json)保留。随后补入确定性应用版本的f6e61bf3已完成自己的Java38411329432／Vue38411329680／Full38411330336／Native38411330281，独立接受全部六个Native作业及32份发布器JSON。实际下载1f114993 JAR实读Implementation-Version=6.1.0、原ReaderApplicationKt入口，完整字节与其两次clean一致；本机离线配置检查不是本机重建或启动。Full生成账号旅程11.074秒及两张中文截图已验看，客户端在服务端预算外；真实账号、准确新镜像单书／原件三方仍不能移借首候选或548a结果。

服务器固定Gutenberg页本轮严格HTTPS预检200，页面摘要与历史捕获一致；准确新镜像的软件源上传遇SSH超时／连接关闭，保留不完整输入，尚未加载或启动Reader。预检不是真实脚本／Cookie业务通过，不改生产或扩大出网范围。

最终发布仍需闭合：

1. 剩余真实书源脚本／Cookie及真实站点认证的明确测试范围和证据；不得复用已清凭据，不扩大唯一域名服务器出网授权。
2. 最终候选精准源码、JAR、镜像与验收范围一致，保留历史失败，不用普通CI绿灯代替真实用户路径。
3. 将候选审查并合入legacy；用户已选定6.1.0并同步项目／根包版本与发布说明，本版本自己的托管验收现已独立接受，但其余真实业务及最终审查未闭合。发布时仍须高于当时最新正式版，不能靠删除“版本必须增加”的门禁发布4.x。
4. 通过托管runner的正式链路完成双registry一致manifest、GitHub正式Release、服务器按digest拉取、回滚快照及公网版本／构建提交校验。现在只有演练与配置名称证据，并未发生正式发布或生产切换。

## 回滚与保全

不改现有Go/Rust工程、原JAR、用户storage或未提交报告；不reset／checkout覆盖。正式部署代码在停止旧容器前拉取并核digest，之后进行一致状态备份和失败状态救援；这些源码守卫并不等同当前生产故障注入验收。原B26、恢复工程、准确归档、失败及独立清理报告保留。任何新发版必须在说明中明确哪些风险已解决、哪些仍仅有限验证，不能把旧预发布说明复制成当前完整证明。
