# 当前请求头失败与准确原件元数据对照

日期2026-10-11。Java/Kotlin legacy维护线；仅生成账号、书源、页面，无真实Cookie或本地正文。GitHub默认分支仍legacy，PR56仍OPEN draft，不发版、不推registry、不部署生产。

## 已从当前自己的托管产物验证

a5cd1e74 / 实际测试快照2147efa3自己的Java、Vue3和Native六作业均终态成功，自己的准确9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84 JAR，285,672,062B / 1570entries，Java两次干净重建一致。25实际生成UI / 零跳过、301前端、603 Linux Python / 零跳过、201 JVM / 35原环境跳过。断网编辑截图已目视，只有一条未确认保存；不能扩展成所有页面或真实账号验收。

Native32份publisher JSON与四套实际Reader HTTPS8项逐字段重验，两个固定证书提示含原有异常类型前缀。此处只接受Native子集：[回执](evidence/tts-notification-native-subset-a5cd1e74-2026-10-11.json) 明确overallCandidateAccepted=false、fullImageWorkflowAccepted=false。未用Native成功替代失败的Full任务。

## 已验证失败；原因尚未验证

[Full任务38065687414](https://github.com/warpdotsys/reader-dev/actions/runs/38065687414) 的24个实际Camoufox契约中1失败、零跳过：generatedRuleHeadersStayOnTheirOriginAcrossRedirectsAndScripts期待生成同源Authorization值，但实际为空。之后的整镜像构建、业务HTTPS和默认整镜像UI没有执行，不应称作已通过。该失败没有证明跨站凭据泄漏，原XML也未包含具体case/actor/role，不能臆定为哪个跳转或直接归类偶发问题。

[独立失败回执](evidence/header-contract-full-failure-a5cd1e74-2026-10-11.json)；[未改写的生成原XML](evidence/header-contract-full-failure-a5cd1e74-2026-10-11.xml)，SHA f7a80968881d90e744477edbd4f072c55637b987c761acb430bff17be2bf39ad。严格发布守卫确实拒绝该XML。新增记录回归防止将同源Native成功或伪造零计数当作该失败通过；不是再执行了一次浏览器。

可读测试源码仅给断言补充有限的case/actor/role/method，不记录URL、正文、Cookie或真实凭据，保持Host、Authorization、trace原值检查，不重试、不跳过、不放宽TLS/网络/资源。焦点Kotlin编译已通过target1.8；首次手动编译缺少正常测试friend模块配置被拒绝，补齐friend路径后使用新的独立输出成功，没有将生产internal改成public。新诊断源码的真实托管运行另需自己的报告。

本诊断候选本机604项Python / 28.252秒 / 零失败 / 1原Windows平台跳过；新增的是已保留实际失败XML的拒绝回归，不是新的Camoufox运行，也不把Linux/Windows的原环境跳过改成业务通过。

## 已从准确原件验证的有限元数据对照

公开归档同名JAR与原件不同，预检失败保留在[归档报告](PUBLIC-ARCHIVE-IDENTITY-FAILURE-2026-10-11.md)。后续准确9b9d JAR已从自己的共享构建artifact下载到本机并独立重算SHA；下载仅2CPU / 512MiB自有Windows任务预算，峰值150,736,896B、164.266秒；没有下载新整镜像或上传原件。

新的本机隔离轮次使用b26准确原件与9b9d恢复JAR，UID10001、cap0、no-new-privileges、只读输入、仅lo网络、2CPU / 2GiB / 256PID / memory.high1.5GiB / 零swap。使用固定历史WebView参考、旧缓存只读runtime，明确不是本机最新整镜像。

- 原件与恢复JAR使用同一历史WebView各完成5项基础搜索/脚本/POST；完整基础ReturnData相等。
- 各实际执行一次getBookInfo，延迟生成DOM中的书名/作者/封面、默认字段类型及诊断标志一致。除三个默认时间字段外，全JSON精确相等；每个时间值严格在自己的实际API请求窗口内。原始完整响应及时间值保留，完整原始ReturnData字面相等为false，不抹掉差异。
- 历史服务独立停止、PID0后才允许Camoufox阶段。但新Reader首个searchBook调用TimeoutError：目标确实收到1次GET，客户端未收到完整Reader响应；搜索完成数0、元数据调用0。不能称作Camoufox元数据错误，也不能称作三方通过，根因尚未确认。
- 102.888秒，内存峰值1,611,567,104B / PID239；soft high3538、hard max/OOM/PID上限事件0、swap0。仅有high事件不证明超时由内存造成。最后两个自有容器独立观察停止PID0并删除，剩余0，WSL当次无D状态进程。

[完整生成元数据、失败阶段和资源回执](evidence/three-way-local-metadata-pair-failure-a5cd1e74-2026-10-11.json)，SHA 97892c29d3cb0fd6fb0cb564fe0199d35abde7249bdd1fcc015be40fa35083e8。首次独立读报告时纠正了“零目标请求”和哈希大小写格式两项假设，没有改写原始报告或把三方失败变成通过。

结束后只清理该轮两个已停止、SHA及路径/单链接/所属UID核对过的临时输入JAR，共358,585,949B；原件、只读备份和准确CI下载均再次重算保留，可重建这些副本。没有递归删除其他cache或目录。清理回执SHA 6a0bf980a8dbaec5a8dad831ce7b6661d82021cb6be44de14a0dc16468e37d18。官方uidmap当前1:4.17.4-2ubuntu3，原有subuid/subgid范围及SHA未改。

## 已知问题与回退

当前同源头失败和本机首搜索超时仍阻挡整体验收。原件完整HTTPS、WS/WSS、真实站点认证/授权书籍旅程、OPDS及生产部署待办继续保留；旧WebView跨站认证泄漏与UTF8截断已知差异不为取绿灯而修改。通知修复仅编辑写入失败，不覆盖所有页面。

本候选只改测试诊断与记录回归，生产源码不变；可单独revert候选，保留原失败证据和当前发布守卫。不得reset覆盖用户工作树。没有使用字节码补丁、空实现、替换原件哈希、真实账号或私有正文。
