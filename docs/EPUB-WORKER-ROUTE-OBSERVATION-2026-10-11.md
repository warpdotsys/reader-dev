# EPUB 实际阅读文档的观察缺口：L 仍不接受整轮

更新：2026-10-11。此前 H/I/J/K 的压力、身份与超时失败原样保留，见[前轮诊断](EPUB-WORKER-DIAGNOSTIC-2026-10-11.md)。本次不是压力修复或完整八场景验收。

## 本次实测与边界

**已从 JAR 验证**：使用自己的托管 Native 运行 38127563397 下载、自己的双 clean 重建独立核对的 be87 JAR，285,672,055B、SHA `be87f00490fc53a0462c48b06cc5500c6fea80ee07ee3f48b2fde035eb83eeab`；源码 `3e225f97efe1036e405490e0b4b253f1285a7ce4`。浏览器仍来自旧锁定 runtime `401d895c…bc44`，不是最新完整 Native 镜像本机验收。

**已实际运行**：只使用授权《黎明之剑》一份 EPUB；新生成账号经界面注册、导入预览和保存后，顺序采样 text805 / html805 / first raw805。不读取原书架进度、不运行原 JAR/API 双方对照、不复制其他本地正文、不导入真实凭据、不发布正文/截图。

| 采样 | 字符 / 段落 | 替换乱码字符 / script | 有限结论 |
| --- | --- | --- | --- |
| text805 | 3953 / 66 | 0 / 0 | 该次文本就绪 |
| html805 | 3952 / 64 | 0 / 0 | 该次 HTML 就绪 |
| raw805 | 3952 / 64 | 0 / 0 | 该次原排版就绪，iframe 872px |

HTML/raw 正文 SHA 同为 `47f23ed8aa5dd68549872f3f73e4c0e971811248bff0abdd21afe5d4f06225f6`。该次没有可见下载/Worker 错误；这不否定 H/I 的真实 15 秒超时，也不等于刷新、连续换章、后端对照或生产容量通过。

**整轮仍失败**：生成 module Worker 自检有四事件，但主文档 route hook 仅见 fixture/login 两次。Reader 导航计数为 0；Reader `goto` 后立即及 raw 就绪后的两个 marker 均 missing。最终校验正确拒绝，最初错误为有限 `validation`，没有再被诊断/收尾异常掩盖。

**同轮完整资源**：父级包含 Reader、浏览器与控制进程；2 CPU、2 GiB、memory.high 1.5 GiB、256 tasks/threads、swap 0。峰值 1,621,876,736B / 248 tasks；PID/hard-memory/OOM 事件 0，soft-high 11,603。探针耗时 45.544 秒、单元运行 55.063 秒；两者不能混作业务 Worker 耗时。服务停止/MainPID0、两个自有容器 stopped/PID0/delete、inventory0 均观察到。

[公开有限回执](evidence/authorized-epub-be87-l-finite-diagnostic-2026-10-11.json)没有正文、原始 URL、Cookie、凭据、私有日志或临时账号。原始有限 JSON SHA `214714d2…1cd4` 留在本机；公开回执不是整份原始 JSON 的复制。

## 可维护探针变更，不是业务修复

`scripts/epub_worker_lifecycle_probe.py` 新 SHA `e4d2573dbb51e2259ef6299b681ba8c27337a79e33441323d96a41c38df3e81f`：Worker 包装本身未改；新增 Python/JS 一致的有限路径分类、route/navigation/html/injection 数值、CSP 有无及主异常保护。候选 URL 即时分类后丢弃，不输出 query/fragment；计数饱和到 4096，实际 harness 拒绝 cap-hit。Worker 仍限定 64 事件，仅字节量/时间/布尔，不输出文件名、正文或构造 URL。

25 条纯生成探针测试通过；本机完整 Python 725 项 / 64.353 秒，724 执行通过、1 项 POSIX 符号链接测试因 Windows 跳过。L harness 另有 19 项 AST/合成检查；不是托管该新提交的验收，不取代实际 Reader Worker 接点。

异常路径将 marker、资源、context/browser/Java 收尾分别降为有限诊断结果并保留 primary；独立核验仍明确 `accepted=false`。页面 HTML 注入有真实时序开销，不能声称无扰动或原产物时序等价。

## 尚未验证与下一步

现在只能确认 Reader 顶层导航没有派发给此 hook，不能把原因直接认定为重定向、缓存或 Firefox 跨进程。[Playwright 官方路由约定](https://playwright.dev/python/docs/api/class-browsercontext#browser-context-route)说明路由会停用 HTTP cache，而当前 service workers 已 block；本次没有缓存命中或跨进程事件证据。[Camoufox 官方隔离评估说明](https://camoufox.com/python/main-world-eval/)亦不能替代实际页面注入证据。下一步用纯生成文档比较 regex/glob、request/frame/goto 与 hook 的有限计数，先补齐观察链路，不再为此重复读取真实书。

不提高产品 15 秒 Worker / 60 秒下载上限，不增加合并预算，不改 main-world-eval、个人浏览器、映射、代理或生产配置。fflate 扩容风险、完整八场景、OPDS、同章 anchor 等已知问题仍未闭合。

## 清理与回退

停止与身份核对后，只 unlink 两个确切临时输入副本，共 323,481,851B，按绝对目录、UID、nlink、只读权限、大小与 SHA 校验；没有递归删除目录。临时副本可由保留的托管 JAR/授权 EPUB 重建，来源仍保留且复哈希一致。58 份用户脏报告与原始 JAR/只读备份的保护检查通过。

回退仅撤回这个测试探针及对应回归，不改业务 JAR、前端、规则、安全或资源上限；保留 L 失败与 H/I/J/K 原证据。没有合并、tag、发版、registry 推送或部署，goal 保持 active。
