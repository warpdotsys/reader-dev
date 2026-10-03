# EPUB3 导航验收与生成 EPUB 双 JAR 对照

日期：2026-10-03。此记录只使用自行生成的样本，不包含私有书籍正文、账号或截图；不代表正式发布或生产已切换。

## 已在恢复版本机验证

`Vue3PreviewLocalReadingTest` 在已有 TXT、双 XHTML EPUB、NCX 同页锚点的全部断言之后，增加第四本生成 EPUB3。导航位于 `Nav/toc.xhtml`，使用 EPUB 命名空间的 `ops:type` 别名，包含嵌套中文锚点、后一 XHTML 文件、重复项和独立 landmarks。另有冲突的兼容 NCX，确保原版书内目录实际优先读取 nav，而不是借 NCX 的成功冒充 EPUB3。

- 后端 `/reader3/getChapterList` 仍为资源级的两项，`Text/shared.xhtml` 对应 0，`Text/later.xhtml` 对应 1；前端没有重编号。
- 原版补充目录恰好三项，展平嵌套目录、去除重复目标、不显示 landmarks 或旧 NCX。
- 中文百分号编码锚点、同页刷新、跨文件目录跳转、跨文件内链返回通过；目标元素距 iframe 顶部误差不超过 2 px。
- 后一文件的主动滚动 180 px 触发既有 `{url,index}` 保存，`index=1`；刷新恢复该内层位置，误差不超过 2 px。
- iframe 保持 `sandbox="allow-same-origin"`，脚本节点为 0，无错误锚点提示。

本机全程 JUnit 1 项、0 跳过/失败/错误，18.400 秒；这一项包含所有四本生成 TXT/EPUB 的完整旅程，不是只有 EPUB3 的孤立测试。受测 JAR 仍是已经重建的 `d6655420` 功能实现，SHA-256 `9fc21e5b49fbebb23b1f706f206ba86042aad5189b03374cb1363fc140462f5d`；本轮目前只增补测试、诊断和文档，没有为了使测试通过修改后端或前端业务实现。

新生成截图 `vue3-local-reading-nav-fragments-synthetic.png` 已目视核验，中文可辨、原版正文及书内链接可见。截图中的宿主章节标题“重复目录”来自夹具特意重复的最后一个资源标签；原版补充导航去重后保留首个标签，不能据此把后端资源级标题悄悄改成锚点级标题。只有生成截图进入托管制品。

## 保留的诊断与契约纠正

首轮测试等待额外的 `/reader3/saveBookProgress index=1` 请求超时。随后带诊断重新运行，观察到 URL 已为 `chapter=1&epubAnchor=finale`，iframe `aria-busy=false`、内滚动 1902 px、目标标题正确，实际书架 `durChapterIndex=1`。这不是进度保存故障：legacy 正文接口本身更新章节进度，不能要求导航后额外发生一条 POST。

首轮 68.009 秒及带诊断 47.725 秒的失败 JUnit 在忽略的 `build` 目录保留。有效回归改为断言目标 `getBookContent index=1&epubContent=1`、真实书架编号和锚点位置；独立主动滚动仍严格检查保存 POST 的状态、ReturnData 和 `{url,index}` 键集合。没有删除断言中的业务结果或放宽位置容差，也没有把初始错误断言包装成产品修复。

本机前端 150 项、Python 45 项、正式发布链路静态检查及 5 项结构/负向检查通过。托管结果应以本记录后续实际运行与下载制品为准，不借上一提交的绿灯替代。

## 原 JAR 同条件对照入口与当前状态

新增 `scripts/compare-generated-epub-fragments-in-netns.py`，直接消费上述浏览器测试可选导出的同一份 NCX/nav EPUB，不另写一份可能不等价的夹具。两份受测文件摘要分别是：

| 生成文件 | SHA-256 |
| --- | --- |
| `generated-ncx-fragments.epub` | `96ab4c49198aecb8fbf4ddceb6bed997b9d06dcb7d4ae79c74f1ff03d814bc0f` |
| `generated-nav-fragments.epub` | `16887fa26dc1753758b1675da6d1c0d4a2f404a53a914638f8195aa06958975d` |

程序在读取任何输入前检查真实 Linux 内核网络命名空间必须与 PID 1 不同且只有 `lo`，启用回环后降权到 UID/GID 65534。仅接受固定生成标识、白名单 ZIP 条目和不超过 2 MiB 的夹具；裸环境变量不能代替隔离。原 JAR 固定校验 `b26fb476...30c8c`，报告必须为 `/var/tmp` 下未存在的新文件，两个 JAR/两个 EPUB 保持只读。

计划比较 HTTP/ReturnData、章节 JSON 默认值与编号、规范化 XHTML 精确哈希、登录/token/用户命名空间、原 EPUB 下载字节、书架进度与格式。仅显式列出的正数时间字段被掩码，0/false/null/空字符串保留；报告记录比较投影和限制。

**尚未执行原 JAR**：本轮 WSL 启动报连接错误。备用服务器具备 `unshare`/`ip`/Python/Docker；安全审查在 SCP 启动前拒绝原 JAR 外传，尚未上传任何 JAR 或真实正文，未启动对照服务。已询问是否明确允许两个 JAR 副本进入用户服务器的唯一临时目录；没有绕过拒绝或用近似源码结果代替。原 JAR 和唯一授权真实 EPUB 的本机摘要复核不变。服务器当前仅创建一个空测试目录，生产容器未停止、重启或修改。

普通主机的真实负向测试确认拒绝发生在读输入/启动 Java/写报告之前。新增 Python 单元测试验证两个生成标识、条目边界、严格 JSON 类型、精确 HTML 投影及普通主机拒绝。

导出时为测试设置 `READER_GENERATED_EPUB_EXPORT_DIR` 到新的隔离输出目录，运行已有 `Vue3PreviewLocalReadingTest`；文件已存在则失败，不覆盖旧证据。仅在已经授权的 Linux 隔离测试主机运行：

```bash
sudo unshare --net --fork python3 scripts/compare-generated-epub-fragments-in-netns.py \
  --java /absolute/jdk-11/bin/java \
  --original /absolute/reader-pro-3.2.14.original.jar \
  --restored /absolute/restored-reader.jar \
  --fixtures /absolute/generated-fixtures \
  --report /var/tmp/reader-generated-fragments-NEW.json
```

服务器实际执行前另加 2 CPU / 2 GiB / 256 Tasks 的外部 cgroup 限额；程序中的 `-XX:ActiveProcessorCount=2` 和 `-Xmx768m` 不能替代总进程树限额。上面的复现入口不是已经执行的服务器证据。

## 仍未验收

原 JAR 新锚点样本双侧运行、嵌套 CSS `@import`、超大 EPUB 解压/资源上限、跨设备原版进度、真实需登录书源的三方差分、长期负载、正式双架构 manifest、稳定版 Release/registry/生产部署仍未完成。回滚继续保留旧 JAR/Vue2 与原存储，不删除其他书籍缓存。
