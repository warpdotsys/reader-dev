# 固定网络书的真实界面旅程

## 当前边界

这是新增的强制验收候选，尚未接受它自己的托管实测。此前三项真实 Gutenberg 业务 API 成功不等于前端阅读成功；此文不提前宣布界面通过。

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

## 已知问题与回退

OPDS 服务仍未实现；合并服务端和客户端预算的真实 EPUB 压力失败、起点验证码/首搜索失败、原件与历史 WebView 的完整真实正文对照、真实账号认证和生产容量仍未闭合。普通 CI 绿灯不能消除这些问题。

回退仅撤回新增强制 UI 探针与验证器，保留之前三轮实际 API 结果和全部失败记录；不得改写原件、用户工作区或关安全保护取绿灯。默认部署仍是一个自带浏览器的 Reader 镜像。
