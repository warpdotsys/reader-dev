# EPUB3 导航验收与生成 EPUB 双 JAR 对照

日期：2026-10-03。此记录只使用自行生成的样本，不包含私有书籍正文、账号或截图；不代表正式发布或生产已切换。

## 已在恢复版本机验证

`Vue3PreviewLocalReadingTest` 在已有 TXT、双 XHTML EPUB、NCX 同页锚点的全部断言之后，增加第四本生成 EPUB3。导航位于 `Nav/toc.xhtml`，使用 EPUB 命名空间的 `ops:type` 别名，包含嵌套中文锚点、后一 XHTML 文件、重复项和独立 landmarks。另有冲突的兼容 NCX，确保原版书内目录实际优先读取 nav，而不是借 NCX 的成功冒充 EPUB3。

- 后端 `/reader3/getChapterList` 仍为资源级的两项，`Text/shared.xhtml` 对应 0，`Text/later.xhtml` 对应 1；前端没有重编号。
- 原版补充目录恰好三项，展平嵌套目录、去除重复目标、不显示 landmarks 或旧 NCX。
- 中文百分号编码锚点、同页刷新、跨文件目录跳转、跨文件内链返回通过；目标元素距 iframe 顶部误差不超过 2 px。
- 后一文件的主动滚动 180 px 触发既有 `{url,index}` 保存，`index=1`；刷新恢复该内层位置，误差不超过 2 px。
- iframe 保持 `sandbox="allow-same-origin"`，脚本节点为 0，无错误锚点提示。

初次本机全程 JUnit 1 项、0 跳过/失败/错误，18.400 秒；这一项包含所有四本生成 TXT/EPUB 的完整旅程，不是只有 EPUB3 的孤立测试。受测 JAR 为 `d6655420` 功能实现，SHA-256 `9fc21e5b49fbebb23b1f706f206ba86042aad5189b03374cb1363fc140462f5d`。这是第一次本机观察，不代表随后托管 runner 的结果，也不代表进度竞争已经排除。

新生成截图 `vue3-local-reading-nav-fragments-synthetic.png` 已目视核验，中文可辨、原版正文及书内链接可见。截图中的宿主章节标题“重复目录”来自夹具特意重复的最后一个资源标签；原版补充导航去重后保留首个标签，不能据此把后端资源级标题悄悄改成锚点级标题。只有生成截图进入托管制品。

## 保留的诊断与契约纠正

首轮测试等待额外的 `/reader3/saveBookProgress index=1` 请求超时。随后带诊断重新运行，观察到 URL 已为 `chapter=1&epubAnchor=finale`，iframe `aria-busy=false`、内滚动 1902 px、目标标题正确，实际书架 `durChapterIndex=1`。这两次超时不能证明业务保存缺失：legacy 正文接口本身更新章节进度，不能要求导航后额外发生一条 POST；但也不能据此排除并发竞争。

首轮 68.009 秒及带诊断 47.725 秒的失败 JUnit 在忽略的 `build` 目录保留。有效回归改为断言目标 `getBookContent index=1&epubContent=1`、真实书架编号和锚点位置；独立主动滚动仍严格检查保存 POST 的状态、ReturnData 和 `{url,index}` 键集合。没有删除断言中的业务结果或放宽位置容差，也没有把初始错误断言包装成产品修复。

`e99982e8` 的 GitHub 托管 [Vue 3 运行 37113117920](https://github.com/warpdotsys/reader-dev/actions/runs/37113117920) 随后失败：10 项核心浏览器旅程中 9 项通过，生成阅读旅程 11.464 秒失败，跨文件正文已读取而书架实际编号为 0（应为 1）。经理密钥文件和子目录旅程被跳过，不能记为通过。相同提交的 Java/Kotlin 和单镜像检查通过，不能代替失败的界面验收。

原因从实际源码核实：切章先异步发出旧章 `saveBookProgress` POST，然后 `getBookContent` GET 本身写入新章进度；旧 POST 晚到会覆盖 GET 的新编号。回归现在扣住真实旧章 POST，严格禁止新章 GET 越过它，再释放 POST 并验证真实书架和锚点。旧候选在此可控条件下 18.007 秒失败，消息为 `New chapter read overtook an unresolved old progress POST`；该有效失败与前两次错误额外 POST 断言分别保留。

修复在前端跟踪实际进度写入 Promise，仅在真正调用正文接口前等待已发出的写入完成，不改变后端、章节编号或本地缓存读取。失败仍保留原调用者的失败语义，不伪报成功。新增 5 项屏障测试后，前端 155 项、类型检查、Vite 构建通过；Python 45 项通过。修复版完整浏览器旅程和新提交托管运行结果须独立记录，不能沿用初次本机通过或旧提交绿灯。

**已成功重建并在新隔离实例验证**：修复候选 JAR 为 `74fbd5a5a4b77a78352b54e82dc446590197eb99ffe2a96495acff7834084306`，只监听 `127.0.0.1:18933`，独立工作目录只含生成测试数据。上述完整浏览器旅程 21.637 秒，1 项、0 跳过/失败/错误；扣住旧 POST 的可控断言通过。生成 EPUB3 截图再次目视核验，中文、书内链接正常。原 `18931` 候选和原 JAR 保留未动；本机首次构建因沙箱无法读取既有 JavaFX 依赖失败，正常授权环境下同命令重建成功，不修改源码绕过依赖错误。

本机修复前/后报告与截图摘要见[机器可读证据](evidence/epub-progress-order-2026-10-03.json)。正文 GET 因服务器超时但实际晚到写入的异常网络情况，以及音视频分支的专门界面旅程，未在此样本证明；本修复不是后端全局事务或跨设备进度同步保证。

## 新修复提交的实际托管验收

源码提交 `c9f9ce6e338a64d22510c04bbacc0d03bfb608a4` 的三道 GitHub 托管 workflow 均成功：

- [Java/Kotlin 37115145388](https://github.com/warpdotsys/reader-dev/actions/runs/37115145388)：构建与测试、155 项前端、45 项 Python、发布安全结构检查及 5 项负向检查通过。
- [Vue 3 37115145387](https://github.com/warpdotsys/reader-dev/actions/runs/37115145387)：下载核心 JUnit 10 项均无跳过/失败/错误，生成阅读旅程 14.659 秒；新旧进度顺序断言通过。经理密钥文件和 `/reader/` 子目录步骤也实际执行并通过其 JUnit 非跳过检查。新增生成 EPUB3 截图已下载并目视核验中文及内链。制品使用 PR 合并快照 `63c5671ec3766b19b4a3cc770d46eb44fab3d95c`，不与源码 SHA 混淆。
- [完整镜像 37115145390](https://github.com/warpdotsys/reader-dev/actions/runs/37115145390)：镜像内真实 Camoufox 11 项无跳过/失败/错误，合成脚本、POST、Cookie 和四账号请求均通过。本轮可选历史引擎、原件来源与公开页配对 jobs 未启用，不记为通过或第三侧对照。

实际 jobs 的标签为 `ubuntu-24.04`、runner 名为 GitHub Actions，不是自托管。[下载的资源 JSON](evidence/c9f9ce6e-browser-resource-budget-2026-10-03.json)记录 2 CPU / 2 GiB / 256 PIDs，内存峰值 `756506624` 字节（约 721 MiB）、182 个任务，OOM/PIDs 限额事件为 0；这是短时顺序样本加四账号突发，不是长期吞吐保证，也不是下节两个 JAR 对照的内存峰值。[合成结果](evidence/c9f9ce6e-browser-synthetic-2026-10-03.json)和[提交/制品摘要](evidence/epub-progress-ci-c9f9ce6e-2026-10-03.json)分别保存。

## 原 JAR 同条件对照入口与当前状态

新增 `scripts/compare-generated-epub-fragments-in-netns.py`，直接消费上述浏览器测试可选导出的同一份 NCX/nav EPUB，不另写一份可能不等价的夹具。两份受测文件摘要分别是：

| 生成文件 | SHA-256 |
| --- | --- |
| `generated-ncx-fragments.epub` | `96ab4c49198aecb8fbf4ddceb6bed997b9d06dcb7d4ae79c74f1ff03d814bc0f` |
| `generated-nav-fragments.epub` | `16887fa26dc1753758b1675da6d1c0d4a2f404a53a914638f8195aa06958975d` |

程序在读取任何输入前检查真实 Linux 内核网络命名空间必须与 PID 1 不同且只有 `lo`，启用回环后降权到 UID/GID 65534。仅接受固定生成标识、白名单 ZIP 条目和不超过 2 MiB 的夹具；裸环境变量不能代替隔离。原 JAR 固定校验 `b26fb476...30c8c`，报告必须为 `/var/tmp` 下未存在的新文件，两个 JAR/两个 EPUB 保持只读。

比较 HTTP/ReturnData、章节 JSON 默认值与编号、规范化 XHTML 精确哈希、登录/token/用户命名空间、原 EPUB 下载字节、书架进度与格式。仅显式列出的正数时间字段被掩码，0/false/null/空字符串保留；报告记录比较投影和限制。

**已从原 JAR 实际验证**：第一次 SCP 在执行前被安全审查拒绝，没有上传。用户随后明确授权两个 JAR 副本进入 `cdn.medwarp.cn:/var/tmp/reader-generated-fragments-20261003.RMZay2`；本轮 WSL 不可用，因此实际使用该服务器的独立临时目录。输入包只含两份 JAR、两份诊断脚本和上述两本生成 EPUB，358569472 字节、不重复压缩；SHA-256 `3a1f76465ce599e5db1658f6f0c4b58c60f3de25609f2e9cf2c554fb75c55246`。远端四份输入摘要与本机一致，解包后的输入移除写权限。JRE 从现有容器只读复制，Temurin `11.0.32+9`；没有停止、重启或改动生产容器，不传真实 EPUB、私有书架或正文。

`systemd-run` 临时单元设置 `CPUQuota=200%`、`MemoryMax=2G`、`TasksMax=256`、`RuntimeMaxSec=240`，再由 `unshare --net --fork` 和探针的真实内核检查建立只回环环境、降权 UID/GID 65534，先后实际运行两份 JAR。作业成功退出，`equal=true`，两个生成样本两侧如下：

| 检查 | NCX | EPUB3 nav |
| --- | --- | --- |
| HTTP / ReturnData / 完整章节 JSON 默认值 | 相同，资源项 1 | 相同，资源项 2 |
| 规范化 XHTML 精确哈希 | 相同 | 两章均相同 |
| 读完后服务端资源级编号 | 两侧 0 | 两侧 1 |
| EPUB 下载 | 200，2079 字节，输入哈希不变 | 200，3027 字节，输入哈希不变 |
| session / token / 匿名 / 他人命名空间 | 全部边界断言通过 | 全部边界断言通过 |

完整[原始 JSON 报告](evidence/generated-fragment-diff-2026-10-03.json)的 SHA-256 为 `332a0f02157d2c3a4f9d2c3b8ab105dbdbf96d7f0e9d5423e96b76c750645252`，远端、本机取回及入库三份字节相同。报告不含真实正文、口令或 token。普通 PowerShell 到 SSH 的管道在末行附加 CR，第一次报告名带 CR；已保留该原证据，只复制为正常名称再下载。后续使用 `tr -d '\r' | sh -s` 消除管道换行问题，没有删除或重写报告来伪造成功。

两份 JAR/两个 EPUB 的输入保持不变。对照单元运行很快，结束后单元已回收，因此未获取它的可靠资源峰值；不把完整镜像的 721 MiB 采样或结束日志的瞬时数字冒充双 JAR 的测量。本轮不证明原 UI 导航像素、全格式/全书源等价或生产部署；恢复前端的像素与 sandbox 由独立浏览器旅程验证。

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

服务器实际运行另加上述外部 cgroup 限额；程序中的 `-XX:ActiveProcessorCount=2` 和 `-Xmx768m` 不能替代总进程树限额。输入与报告须在新目录中复现，保留已存在的证据，不覆盖本轮报告。

## 恢复构建入口

在工程根目录，Node 24、JDK 11 环境下用锁文件与 Gradle Wrapper 重建 Vue 3 候选：

```bash
npm ci --prefix web-vue3 && npm run build --prefix web-vue3 && PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 bash ./gradlew -PreaderWebUi=vue3 test bootJar --no-daemon --max-workers=2
```

产物为 `build/libs/reader-4.0.7.jar`。本机此次使用独立 init 脚本把候选输出到 `build/epub-order-candidate`，避免覆盖运行中的旧 JAR；不修改发布版本号，也不以本机构建代替正式托管发版。

## 仍未验收

嵌套 CSS `@import`、超大 EPUB 解压/资源上限、异常网络晚到写入、跨设备原版进度、真实需登录书源的三方差分、长期负载、正式双架构 manifest、稳定版 Release/registry/生产部署仍未完成。原 JAR 生成样本后端双侧运行已完成，但不能称为原 UI 锚点取样。回滚继续保留旧 JAR/Vue2 与原存储，不删除其他书籍缓存。
