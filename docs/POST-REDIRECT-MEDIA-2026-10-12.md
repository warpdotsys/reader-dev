# POST 重定向类型：保留托管失败与候选验证范围

更新日期：2026-10-12。只使用生成请求、生成认证值与Cookie；没有读取真实正文、导入账号或改动生产服务。此处不是将一个未复现的托管错误宣布为已修复。

## 8e0 自己的托管结果：三绿一红，不借旧绿灯

输出缓冲区提交 `8e0f8d7ce89212080decdd56ab7e214e17d16ab6`，测试合并快照 `9c569c79f981f6fcea7ea475d1ad095d89454273`：

- [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38187529051)、[Vue](https://github.com/warpdotsys/reader-dev/actions/runs/38187529033)、[Native六作业](https://github.com/warpdotsys/reader-dev/actions/runs/38187529107)已终态success。
- [完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38187529026)失败于第14步Camoufox合约；24项中1项失败、errors/skipped0。主失败为 `generatedPostRedirectsPreserveBodyCookiesAndOriginHeaders`，10.229秒，目标应收到 `application/json; charset=UTF-8`，实际为 `application/octet-stream`。
- 原断言没有输出子场景/actor/role，不能把它定为跨源307或初始请求失败。后续source-document与镜像UI报告因未生成也使上传步骤失败；它们不是第二个已定位业务根因。新EPUB镜像UI在该运行没有执行，不能把这个错误归因于输出缓冲区修正，也不能用Native成功替代缺失UI。

失败XML SHA `d8597a7e…0293`已下载原样保留，未重跑覆盖。该提交的Camoufox worker源仍为原 `d8b2674c…65f5`，本次前端改动没有变更它。自己的双clean报告证明新JAR `cb115989e91a2bef70d44356ae3ed8cb0e9e38796cd6670eef571e964910fc97`，285,672,069B/1570条目、完整字节相等；目前此处只下载并核验小repro报告，**没有完成共享JAR本机重新散列或整镜像接受**。不能借eda9的0675产物或四绿接受8e0。

## 源码已确认的缺口与尚未证明的根因

原native listener在初始同源请求应用规则类型，但跨源POST只保留Firefox给出的当前类型，不记住已发送的正文类型。因此当当前类型变成octet-stream时，原监听器无法恢复JSON/表单类型。新增模型回归使用原监听器实际JS，先得到32项中3项同类失败；不是浏览器实际重现证明。

Firefox为合成POST重新构造upload元数据是候选解释，不是本次托管根因结论。静态检查Java HTTP转发分支，未看到它主动把Content-Type替换成octet-stream；这也不是端到端排除Java转发。托管失败的准确子场景仍须新的定位信息验证。

## 候选源码修正：一个有边界的原生请求记录

候选不改POST方法、正文或重定向策略，不发送额外业务请求，不重试业务导航。仅在原生listener中：

1. 对本render合成的初始POST，要求原URL、main_frame、frame0、有效tab/requestId，捕获应用后的唯一正文类型；每个render只保留一个记录。
2. 同一requestId/tab/frame的已知307/308，根据浏览器提供的有效HTTP(S)目标URL继续保留类型，支持同一目标的重复原生header事件。
3. 只恢复Content-Type；Host/Cookie/Content-Length继续由浏览器控制，Authorization及规则trace仍按原origin隔离；不使用已知会冻结Host的Route.headers。
4. 301/302/303、GET/HEAD转换、错误/完成、未知/非HTTP/过长目标、不一致request/tab/frame或意外目标均不能借该记录恢复类型；不接管其他网站脚本的新POST。

原URL/目标/requestId/正文类型只留在当前addon内存，不写入诊断、URL参数、持久文件或响应。类型与目标长度受限，addon/private bootstrap及临时目录生命周期不改。请求身份关联依赖[Mozilla webRequest接口](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/webRequest)规定的同一请求跨重定向保持ID；[onBeforeRedirect](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/webRequest/onBeforeRedirect)仅提供信息，修改仍发生在既有的阻塞header listener。这些文档不是对锁定Firefox实际行为的替代证明。

新增模型覆盖JSON及默认表单类型恢复、多个307/308、原生重复header事件、错误/完成清理、ID/tab/frame错配、缺事件、转换GET及无效/错配目标。当前35项监听器JS通过、86项worker mock通过、完整Python725项/29.800秒通过（1项既有Windows/POSIX平台跳过）。这些不是实际浏览器/Java gate接受。JDK11 `compileTestKotlin` 1m30s成功，只证明编译；控制台旧javac提示编码不等于已发现UI乱码。

真实JUnit方法保留原4个JSON重定向场景及240秒总上限，新增4个互相独立namespace的默认表单场景；断言加入有限case/actor/role/method提示，不输出正文或凭据，原正文、Cookie、Host及origin认证断言不放宽。总suite方法数仍24；新的托管执行须另验。

## 实际锁定浏览器：候选无已观察回归，但本机未复现失败

本机A使用0.5.6/锁定Firefox152.0.4-beta.30与缓存runtime `401d895c…bc44`，明确不是最新整镜像。baseline源SHA `d8b2674c…65f5`，候选快照 `50497908…cf08`；六执行文件在开始时写入SHA，源快照只读挂载。UID/GID10001、空附加组、零capabilities、no-new-privileges、六namespace独立、仅lo及只读根均核验。旧恢复Jar只读挂载但未执行，未挂载真实storage或用户浏览器会话。

使用两个生成HTTP站点及只允许其地址/端口的测试转发器，不是Java BrowserEgressProxy，不测HTTPS、真实站点、原件/远程三方或Reader API。每个worker各做JSON/默认表单 × 同源/跨源 × 307/308，共16个实际render、32个目标请求，核验完整正文长度/SHA、媒体类型、实际Host、origin认证值、trace、Cookie与返回DOM/生成HttpOnly Cookie。

| 观察 | baseline | candidate |
| --- | --- | --- |
| 已实际完成的场景 | 8 | 8 |
| 完整有限断言通过 | 8 | 8 |
| 目标请求 | 16 | 16 |
| 复现托管类型错误 | **没有** | 没有观察到 |

因此只能说这一轮候选未观察到生成请求回归；不能宣称旧逻辑必失败、根因已证实或生产问题已修复。原托管失败仍保留为failure。

合计父预算2 CPU/2GiB、高水位1.5GiB、256PID、配置及观察swap0不变。实际内层44.968秒、systemd50.075秒/CPU52.903秒，峰1,611,337,728B/PID234，soft-high1116，hard/OOM/PID-max0。soft-high和有限PID余量仍是风险，不是容量修复；不把父采样当作瞬时Firefox或Java线程分类。16个自有worker父进程已退出，一个自有容器stopped/PID0/delete、inventory0，另外确认单元inactive/dead/MainPID0。

[有限公开摘要](evidence/post-redirect-media-diagnostic-2026-10-12.json)绑定原始135,413B回执SHA `7a7038b7…bc52`、失败XML及各执行源，独立重新检查后精确比较；公开文件SHA `e2f79622…049f`。不含URL、正文、Cookie值、用户名、requestId或异常原文，未反写任何旧轮次。

## 未闭合项与回退

候选自己的托管合约、准确新JAR/实际镜像、失败子场景定位、Java gate端到端、HTTPS/实站认证与原件/远程/current三方仍待验证。EPUB真实单书压力、fflate瞬时工作集、应用/浏览器任务分类、完整场景、OPDS、同章anchor及Z未定位4xx等原缺口仍保留。

回退在独立干净工作树定向移除单请求媒体类型记录/生命周期listener及post标记；恢复原origin策略，不更改Cookie、正文、Host或用户storage，不reset本工作区。保留失败与回归以暴露该缺口；回退不是“已解决”。当前不合并、发版、推registry或部署，完整goal保持active。
