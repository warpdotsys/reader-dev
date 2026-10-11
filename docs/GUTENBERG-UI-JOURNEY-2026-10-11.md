# 固定网络书的真实界面旅程

## 当前边界

第二轮固定匿名网络书的真实界面旅程已独立通过，主代理实际查看两张无正文图片；首轮业务成功但退出截图空白的拒绝结果保持不变。只接受这一固定子集，不等于整个项目、原件三方、真实认证或生产UI通过。

不改变 Java/Kotlin 产品代码、JAR 原版兼容结论、现有浏览器请求超时、TLS 或私网保护。不涉及用户账号、起点 Cookie、私人本地书库、生产服务、registry、合并或发版。

## 当前已接受的输入

cc17b0ceb964632221168ce1c996c162ab348696 自己四条托管任务全部完成成功，并独立核对报告与身份：Java 38117044865、Vue 38117044828、Full 38117044853、Native 38117044841。测试 PR 快照 ffa8ddc1e77e15df97fee6f1a60e406afd317ca5。

- Linux Python 644 / 零跳过；JVM 205 / 35 既有环境条件跳过；前端 302，实际生成 UI 25 / 零跳过及 11 张生成截图身份。
- 双 clean JAR 都是 f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b。实际 Camoufox 24 契约、打包 HTTPS 负例、双架构重导入、发布导入流程 32 份报告均属于上述提交。
- Full 峰 860,553,216B / PID197；Native 各观察最大 905,175,040B / PID197。普通任务允许 swap，实际观察 0，不称配置零 swap。
- AMD64 镜像 sha256:157eb65f3c2310b41550790605036369f0190a2c9c851bfe5033840322a4ee3f；ARM64 sha256:f456dae4afd283b297c781e5bb797c6e22649b142335c22840544c6845c5dae5。大型归档摘要来自托管观察，没有下载到本机重新散列。
- [Java](evidence/gutenberg-normal-hosted-cc17b0ce-independent-java-acceptance-2026-10-11.json)、[Vue/JVM](evidence/gutenberg-normal-hosted-cc17b0ce-independent-ui-java-acceptance-v2-2026-10-11.json)、[Full](evidence/gutenberg-normal-hosted-cc17b0ce-independent-full-acceptance-2026-10-11.json)、[Native](evidence/gutenberg-normal-hosted-cc17b0ce-independent-native-acceptance-2026-10-11.json)；用户 58 份脏报告逐一散列未变化。

## 新的实测方式

使用上述准确 Native 输入，不重新构建它来代替已验收产物。唯一 Reader 服务容器包含 Java/Kotlin 与后端 Camoufox；另外启动同一镜像中的一次性测试浏览器客户端，不是第二个 WebView 服务，也不是部署依赖。客户端只访问这一个隔离 Reader 的 loopback 端口；它通过真实 Vue 控件注册、登录、入架、阅读、下一章、刷新、上一章、返回书架和退出。只有固定书源配置由 API 种入生成用户命名空间，不能称书源由 UI 导入。

服务端与客户端各自 2CPU / 2GiB / 256PID / 禁用 swap，分别观察 cgroup；服务端的 UI 预算文件是 API 后 UI 整个容器生命周期，不是单独 UI 峰值。客户端在服务端预算外，不能称二者合计通过 2GiB。单请求后端浏览器 18 秒不变；客户端单次界面等待 35 秒，整个测试进程上限 210 秒。

目录依然要求完整 28 项及原字段和 WebView 选项；实际正文只在内存中与已收到的真实 API 字符串作空白规范化比较，切章顺序 0→1→刷新1→0，检查重新绘制后的内容、编码与当前标题。正常 UI 的缓存和下一章预取保留，不把入口 API 次数说成目标网站请求总数。

只导出有限状态、计数、字符数、摘要、资源和清理报告，以及空登录/退出页面图片。正文或可能透出正文的工具栏图片不导出；工具栏图片只在内存计算摘要。这不是主代理逐图目视正文验收。两个 owned 容器在退出路径清理，不改变源镜像或用户数据。

`gutenberg-fixed-book` 模式内 API 与 UI 都是强制步骤，失败不得忽略；`verify-gutenberg-ui-report.py` 单独拒绝缺失旅程、失败包络、错身份、正文不匹配、错误编码、未撤销会话或资源触限。候选脚本及其生成护栏通过不等于该界面已经实际通过。

本机新增 UI 及其验证器共16项，连同旧固定书探针共33项通过；全部 Python 660项/27.424秒，只有1项POSIX平台条件跳过。Shell语法检查通过。它们是本机生成守卫，不是网络实测或后续提交自己的托管证明。

## 第一轮真实 UI：业务通过，空白退出截图使完整验收拒绝

[托管任务38118882683](https://github.com/warpdotsys/reader-dev/actions/runs/38118882683)，维护源码5fa6759e3466f66ed2129d4934ed47162bd65497 / 作业114409319171。输入是上面 cc17 自己 Native38117044841 的 ffa8dd 快照 / f5c9 JAR / AMD64 157eb65 镜像，不是5fa后续普通CI或本机旧运行时。

27.886秒实际完成注册、退出、登录、UI入架、0章→1章→刷新1章→0章→书架→退出。真实首章6822字、第二章7332字，4次画面正文都与本轮真实API字符字符串在仅去空白后匹配，U+FFFD为0；28项目录保持完整原字段。页面观察到登录2、退出2、详情3、目录2、正文3、入架1、进度3个响应，均HTTP200/业务成功/空errorMsg。缓存与预取没禁用，不将响应数当目标站点请求总数。生成会话已实际撤销，两自有容器移除；书源配置仍由API种入，不能说UI完成书源导入。

服务端整个API+UI生命周期峰1,018,081,280B / PID214；外部客户端峰796,041,216B / PID171。各2CPU2GiB/256PID/配置和观察swap0，high/hard/OOM/PID触限都0；不证明二者合计2GiB/256PID。

主代理实际查看两张无正文图片，登录页面正常可读，而退出图1280×900为单一像素值，非背景像素0。独立像素解析与本机Pillow交叉核实；登录图1122种像素值/29940非主色像素。Vue源代码有0.2秒淡入及路由过渡，但本轮没有在截图时记录CSS透明度或动画状态，因此“截图过早撞上淡入”只是有源码依据的推断，不追补成唯一已验证原因，也不归结为服务端退出失败。

[独立部分接受/完整拒绝回执](evidence/gutenberg-ui-first-independent-incomplete-2026-10-11.json)，SHA5b01ebf2a704895eb40a1861a03333bff875822e252c420456c93c2019828ee4。准确GitHub artifact11693778340 ZIP SHA9a06d63f03161d2f2383371d516f625e8e4dbb38d885c7058f0859d9adafa5cd已在本机重新计算，15个提取文件逐字节与该ZIP一致，不是仅校验任务外壳或文件名。保留[原报告](evidence/gutenberg-ui-first-raw-observation-2026-10-11.json)、[服务端预算](evidence/gutenberg-ui-first-server-budget-2026-10-11.json)、[客户端预算](evidence/gutenberg-ui-first-client-budget-2026-10-11.json)及两张原图；没有正文图片、正文或真实凭据。

后续候选只增强测试：在原35秒界面等待内验证元素及祖先的计算透明度、可见性和有限动画已结束，再等两帧绘制；不关动画，不改产品/后端18秒超时或预算。机器还必须解码无正文PNG，拒绝单色/近空白图，验证器只用标准库，不临时安装系统包。原首轮记录不改写，新的严格护栏不接受首轮旧记录。

新增绘制与PNG6项生成回归，本机全部Python666项/26.876秒/1项平台跳过通过，固定书相关共39项；真实JavaScript引擎验证隐藏祖先、低透明度及未完成有限动画不能通过，PNG验证五种过滤器、CRC/损坏/超限与单色图片。普通5fa自己的托管验收仍按自己的快照独立收集；画面重测使用单独的验收分支，不取消正在运行的PR任务，结果不得混写为普通CI已接受。

## 第二轮固定匿名 UI 实际通过

[托管任务38119618382](https://github.com/warpdotsys/reader-dev/actions/runs/38119618382)，源码8f8095dbbd4d89b88947f30cd76b0ddced25302f / 独立验收分支ci/gutenberg-ui-paint-20261011 / 作业114411471064。输入仍是准确cc17 Native38117044841 / ffa8dd / f5c9 / AMD64 157eb65镜像，与首轮产品和浏览器字节一致，不将源码提交、测试镜像快照或普通PR身份混写。托管Linux验证Python666项、零跳过，其他7个模式按条件跳过，没有宣称它们已执行。

27.456秒实际完成同一真实注册/登录/UI入架/0→1→刷新1→0/书架/退出旅程。首章6822字符、第二章7332字符，四次DOM内容与本轮真实API仅去空白后匹配，章节顺序、有限动画/祖先透明度/绘制就绪均实测通过，替换字符0；28个原相对目录、基础字段及WebView选项全部保留。实际观察响应数量与首轮相同：登录2、退出2、详情3、目录2、正文3、入架1、进度3，全部HTTP200/业务成功/空errorMsg。

服务端API+UI整个生命周期峰1,054,720,000B / PID211；客户端峰798,932,992B / PID168，各自2CPU2GiB/256PID/零swap，high/hard/OOM/PID触限0。没有降低当前UI预取/缓存行为或改原产品超时、TLS、私网保护。会话撤销和两容器清理通过，只有源配置由API种入，不称UI完成源导入，也不证明服务端与客户端合计2GiB。

登录PNG1122种像素值，退出PNG1113种，各29940个非主色像素；均不是空白。主代理通过本地图片工具实际查看这两张图，中文标签、表单和按钮可读，退出页正常可见且字段为空；没有目视网络正文图片或泛化为其他页面编码正常。机器回执不伪称它自己能确认主代理目视，另留[目视记录](evidence/gutenberg-ui-second-manual-visual-2026-10-11.json)。

[独立通过回执](evidence/gutenberg-ui-second-independent-success-2026-10-11.json)，SHA89e99aec2718264f6f50207a62f39aaef10fa19ddf506761f6f3a095d8115bfc；artifact11693639865的ZIP SHAb13fcd11cb1ab45420162f238677e7874c2e48356313ea48d6ccb2360aede0a3在本机重算，全部15个文件逐字节与ZIP一致。保留[原报告](evidence/gutenberg-ui-second-raw-observation-2026-10-11.json)、[服务端预算](evidence/gutenberg-ui-second-server-budget-2026-10-11.json)、[客户端预算](evidence/gutenberg-ui-second-client-budget-2026-10-11.json)和两张无正文原图。首轮原图/报告/失败不覆盖，不上传正文、Cookie、真实凭据或书库数据。

随后只对离线PNG验收器补了必须存在完整IEND结束标记的断言和缺失IEND负例，39项相关生成检查重新通过，对已下载的第二轮真实报告/图片也只读重验通过，没有再请求书籍。此小改动及文档仍需要后续提交自己的普通托管验收，不借8f实站或5fa普通任务取代。

## 5fa 普通四条与 Native 六作业已独立接受

源码5fa6759e3466f66ed2129d4934ed47162bd65497 / 测试PR快照f5d3eb47ebe6583f862f6f75faf27f101a8236f3。Java38118877502、Vue38118877501、Full38118877517、Native38118877494均完成成功，之后各自下载有限报告并逐项核对，不借cc17结果接受。

660 Linux Python / 零跳过，205 JVM / 35既有环境跳过，前端302 / 实际生成UI25零跳过和11截图身份，双clean f5c9 JAR、真实Cam24、打包HTTPS负例、双架构原生重导入和publisher32报告均通过。Full峰875,921,408B / PID196；Native各观察最大882,376,704B / PID209。普通任务swap允许值与实际观察0区别保留。

该次普通Native AMD64镜像sha256:c07377da214ceb16eacc6b2a4b587b6c28b2ccc0bcebfcbaedef47f6da16f13d，ARM64 sha256:80ef753114a315609fc3302e8643d9b94745b1e164082facccabddaf3fd38954。它们不是上述两轮真实UI输入的cc17 / 157eb65镜像；虽然准确JAR摘要同为f5c9，也不能将旧输入网络实测身份追写成5fa镜像。大型镜像没有下载到本机重新散列。

[Java回执](evidence/gutenberg-normal-hosted-5fa6759e-independent-java-acceptance-2026-10-11.json)、[Vue/JVM回执](evidence/gutenberg-normal-hosted-5fa6759e-independent-ui-java-acceptance-v2-2026-10-11.json)、[Full回执](evidence/gutenberg-normal-hosted-5fa6759e-independent-full-acceptance-2026-10-11.json)、[Native回执](evidence/gutenberg-normal-hosted-5fa6759e-independent-native-acceptance-2026-10-11.json)。用户58份脏报告和准确原JAR仍逐项检查未变；手动重复任务38118884770在确认自动任务存在后已取消，不算验收失败或成功。

最后本机完整Python666项/26.699秒/1项平台跳过再次通过，包含离线IEND加固；自己的后续普通CI仍需重新执行，goal保持active。本次不合并、发版、registry推送或部署。

## 已知问题与回退

OPDS 服务仍未实现；合并服务端和客户端预算的真实 EPUB 压力失败、起点验证码/首搜索失败、原件与历史 WebView 的完整真实正文对照、真实账号认证和生产容量仍未闭合。普通 CI 绿灯不能消除这些问题。

回退仅撤回新增强制 UI 探针与验证器，保留之前三轮实际 API 结果和全部失败记录；不得改写原件、用户工作区或关安全保护取绿灯。默认部署仍是一个自带浏览器的 Reader 镜像。
