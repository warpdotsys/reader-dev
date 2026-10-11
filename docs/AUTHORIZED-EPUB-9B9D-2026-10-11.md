# 准确恢复 JAR 的真实授权 EPUB 增量验收

日期：2026-10-11。仅用户授权《黎明之剑》一个 EPUB；37,809,796 B，SHA-256 bd8c68194bb26a3c3bc5edc7b6baa7023c584441f1c2835f71fab3a3bf3ced31。没有读取其它本地正文，没有复制浏览器会话、导入真实凭据、改生产进度或部署产物。公开回执只含状态、数量、SHA 和安全断言，不上传书文件、正文、标题列表、原书架 JSON 或真实截图。

## 已从准确原 JAR 验证

原件 b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c 与恢复版 9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84，在同一隔离轮次实际启动和完成 API 对照。9b9d 下载来源仍为 a5cd/2147；字节与 8e74 自己两次干净托管重建一致，[构建身份另记](STOPPED-CACHE-HOSTED-2026-10-11.md)。

只从授权 EPUB 的 OPF/1611 spine 项生成一次性测试元数据；使用合成账号和映射后的单书路径。**未使用生产书架的原始进度或其它书籍元数据**，不将它冒充整份真实 storage 迁移验收。

- 目录 1611 项，index/URL/title 投影摘要两侧一致；HTTP 200、ReturnData 字段、isSuccess=true、空 errorMsg、标题无 U+FFFD。
- 章节索引 0、1、805、1610，HTML、规范化正文摘要、字段和响应状态两侧相同。正文字符计数分别为 0、205、3888、321；第 0 项无文字是原书行为，不伪造内容。
- XHTML 资源均为 HTTP 200 / application/xhtml+xml，历史资源路由注入脚本到解压副本；资源字节与注入后的 HTML 相等，正文文本不变，原 EPUB 不变。
- 目录生成可更新一次性书架的目录元数据；随后的缓存阅读不再改书架，四个进度字段不变。第二命名空间及匿名访问均拒绝。

上述仅接受一书四章取样，不证明所有章节、生产账号、原书架字段或所有本地书兼容。

## 已实际操作恢复版界面；整轮仍失败

项目随附 Camoufox 在 headless 隔离运行时实际操作 Vue 3 页面，不使用宿主 Chrome 或用户会话。注册、预览导入、确认入架均成功。以下是 **C 轮六个已完成取样**，不是截图目视或原版 UI 三方比较：

| 场景 | 段落 | 渲染字符 | 结果 |
| --- | ---: | ---: | --- |
| 文本 805 | 66 | 3953 | 正文可见，不是资源 URL |
| 下一章文本 806 | 59 | 4017 | URL 更新、正文 SHA 改变 |
| 刷新文本 806 | 59 | 4017 | 与刷新前 SHA 一致 |
| 净化排版 806 | 57 | 4016 | 书内脚本未进入 DOM |
| 原版 iframe 806 | 57 | 4016 | SHA 与净化正文一致，iframe 高 688 px |
| 原版上一章 805 | 64 | 3952 | URL 更新、正文 SHA 改变，iframe 高 688 px |

六项均无 U+FFFD、脚本数 0、非资源 URL；iframe sandbox 保持 allow-same-origin，不允许书内脚本。渲染字符受 innerText/段落换行影响，不把它和 API 规范化文本字符数当成同一种量。

**已知失败：原版 805 刷新后等待 iframe 超时，最后返回文本场景未执行。** 整轮 fullRunAccepted=false / uiWholeAccepted=false / aggregateBudgetAccepted=false，不把六项局部成功改写为整轮通过。

[独立局部回执](evidence/authorized-epub-9b9d-partial-2026-10-11.json)，SHA d748140270289842ec120492c2e70fc0f412872df4f469665c80412d7f05782b；原始 C 轮报告 SHA 6eebf769d54d124dd8302a290853bcb9fa74780a06b38b6fe15721a486ebd90b。

## 已验证的测试工具修正；不是产品业务修复

缓存镜像的 Python 实测 3.10.12，缺少 hashlib.file_digest；该接口[从 Python 3.11 才提供](https://docs.python.org/3.11/library/hashlib.html#hashlib.file_digest)。A 轮停在原件 API 的 AttributeError，未记录精确失败行，因此不追补一个并不存在的栈作为原轮根因证明。

维护脚本现以 1 MiB 分块、完整原字节 SHA-256 代替新版接口；没有跳过校验、换算法或放宽原件/授权文件身份。新增 5 项回归包括缺失新版接口、空/已知/跨块向量和读取边界。本机该模块 9 项全通过；全 Python 627 项 / 47.386 秒 / 零失败 / 1 既有 Windows POSIX 跳过。B/C 轮实测 Python 3.10 完成两侧 API，执行维护脚本 SHA f1e4aeeaf0dda99bfc6e0adc578a6475852aee8149913ef3e27f78153fc7605f。新源码自己的托管结果另验，不能借 8e74 绿灯。

B 轮注册/导入和正文出现后，测试用 WebCrypto 指标采集报 Error；退出状态与发送停止信号之间又遇到竞态。只从生成 stdout 恢复 API 对照并独立删除已停止 PID0 的两个 runtime，**缺少完整资源回执，因此不接受整轮预算**：[保留恢复后的有限失败记录](evidence/authorized-epub-9b9d-recovered-failure-2026-10-11.json)，SHA 0037263e7a40705a2f4945c52b36e09bb690695beebc94a35013d6ef2b8516f4。

后续指标 SHA 在隔离进程内计算，授权 DOM 文本仅暂留进程内存并立即移除，输出只有计数和摘要；不改浏览器安全 profile。停止信号失败只有独立重读 stopped/PID0 才接受退出竞态。没有业务重试、延长原单请求超时或修改生产字节。独立核验首次误把 cpuPeriod 的整数类型当成字符串，纠正为采集器的实际整数契约后仅接受局部结果；原报告未改写。

## 资源失败、清理与尚未验证

UID/GID10001、空附加组、capability0、no-new-privileges、只读根/输入、仅 lo 网络。父 aggregate 2 CPU / 2 GiB / 256 PID / memory.high 1.5 GiB / swap0；这次预算同时包含 Reader 和 UI 测试浏览器，比托管普通 Full 的服务端预算范围更广，不能混作生产容量结论。

C 轮 162.498 秒，内存峰值 1,623,584,768 B，soft high 214623；hard max/OOM 均 0，swap0，但 **PID 峰值256 / max事件1**。触限首次采样在约51.572秒，尚无逐界面步骤的同步进程分类；不能证明它是后续刷新超时的唯一原因，也不归咎于用户 CPU。

同预算 D 诊断轮也失败，保留不同轮次的结果，不借用 C 六项成功：目录/四章 API 继续一致，UI 只完成文本805、文本806、刷新806、净化排版806四项；第一次进入原版806时页面明确显示既有 **15秒解压 Worker 超时**，没有 iframe，下载超时标志为false。77.463秒，内存峰值1,625,563,136B、PID256/max事件1、hard/OOM0、swap0；两实例清理剩余0。九个步骤只采集自有 PID namespace 中有限的进程分类、线程计数和僵尸数，不读命令行、环境、正文或Cookie。注册时Java29/浏览器134线程；导入后Java69–70、浏览器150–163线程；九次僵尸数均0。因此没有证据把失败归因于遗留僵尸，仍须区分父组压力、下载/解压和浏览器线程配置，rootCauseProven=false。未提高限额、延长解压超时或削弱站点隔离。

[D轮有限诊断回执](evidence/authorized-epub-9b9d-diagnostic-2026-10-11.json)，SHA 723a23dd61684b962765a087d0b3f49ba5e8768b322f30a344d156c7091f4098；原始报告 SHA c0c4188ab8e06bbf2f7ed7cf398e1f861f54d0713c4c04e04d81085c3929711b。两轮整体验收均false，不做多轮拼接来声称八场景通过。

各轮两个自有 runtime 已 stopped/PID0 并删除，剩余0。A/B/C/D 轮各仅清理三份核对过 UID、单链接、只读权限、精确路径/长度/SHA 的临时输入副本，每轮396,395,745 B，共1,585,582,980 B；源原件、备份、CI下载和授权 EPUB 均重算保留，副本可重建；没有递归删除其它目录。C 清理回执 SHA 4352be11140b2cb75be6b697f3d69e04ce524c65dfd89aa343a8b86632d61aa6；D 清理回执 SHA 37283d56b039d4d5d06e7d611a153b66e69cf16287ecd0105e6e6e99be64ede5。

缓存 runtime 仍为旧锁定只读镜像，本轮不是最新完整镜像本机启动。全部八个真实 UI 场景、当前刷新资源失败根因、真实站点认证、原件完整 HTTPS/WS/WSS、OPDS、生产迁移与发版均未完成。58 份受保护用户文件未变。保持 PR draft、不发布或部署失败轮次。

回退仅撤回自有测试工具改动，保留原件和所有失败记录；不 reset 用户工作区，不删除真实 storage，也不用空实现或字节码补丁取代业务逻辑。
