# 当前 Vue 3 截图的可验收性与听书取图守卫

## 已从当前托管产物验证

本机目视检查 [Vue 37762083669](https://github.com/warpdotsys/reader-dev/actions/runs/37762083669) 的四张原图，源码 `47cc4440c50f494e1c6bee02934b72992a16af23`、受测合并快照 `63b51c0bda768383842d04eee72560c4b747c4d0`。它们来自生成账号／生成文本，不是用户私有数据。[图名、完整散列与观察等级](evidence/ui-snapshot-readiness-2026-10-08.json)单独保留，不修改此前接受回执里的“当时未额外目视”记录。

- 桌面登录完整文档、360×320 注册滚动到底视口、窄屏管理权限表：所见中文正常、按钮与权限标签可辨。这仅适用于这三张图；登录完整文档不是同高度的一屏，也不能代表全部页面没有编码或布局问题。
- 听书返回账号 A 的图中，卡片文字／背景与正文重叠，外观不足以作为稳定、可读的听书面板验收图。原 XML 证明缓存／生成音频旅程通过，但不证明这张图的稳定性。

## 已从测试源码推断，而非产品故障定论

旧测试只等到 blob 音频开始播放就取图；没有等待原样 CSS 过渡结束或记录实际 opacity。源页面有 `pop` 淡入，故“截图赶在淡入时取得”是合理推断，**不是已记录动画数值后的唯一结论**；尚不能据此认定产品持续透明、遮挡或 z-index 错误，也不能宣称已经修复产品故障。

## 可维护的定向修正

只改变测试与取图条件，不改产品 UI、缓存、业务、依赖、动画时长、JAR 或资源预算：

- 新只读 `ui-screenshot-readiness.js` 等字体就绪、Vue 进出阶段结束、有限动画完成；不取消、禁用或快进动画。`Document.getAnimations()` 包含 CSS 动画和过渡，`pending` 与迭代时序用于区分未完成状态及合法无限活动指示。[API 说明](https://developer.mozilla.org/en-US/docs/Web/API/Document/getAnimations)、[pending](https://developer.mozilla.org/en-US/docs/Web/API/Animation/pending)、[计算时序](https://developer.mozilla.org/en-US/docs/Web/API/AnimationEffect/getComputedTiming)支持这些检查。
- 播放／返回图必须仍有面板，卡片及祖先 opacity 为 1、在视口内、中心命中卡片或子控件，生成音频仍实际播放；设置页图必须没有旧听书卡片。仍使用原 15 秒页面超时，不靠固定睡眠或提高超时取绿。
- 20 项 Node 测试使用生成 DOM／动画替身，拒绝待定、暂停中间态、透明、裁切、遮挡、字体未就绪及场景类型混用，并验证实际 Java 取图入口先执行守卫。它们全执行通过，但不算真实浏览器验收。
- 当前本机 JDK 11.0.8 使用既有 Playwright 1.63.0／JUnit 4.13.2 依赖完成定向编译，128 MiB 编译堆／2 个 JVM 活跃处理器，不安装软件、不启动 Reader／浏览器。托管 Vue build 同时运行这 20 项守卫测试；实际 Chromium 旅程须在该增量自己的运行中另验，不能借用 47 的旧绿灯。

## 增量自己的托管实跑已通过

源码 `8e4f5bf82d410c1d74e3342c799ffed5d76beb82`、受测合并快照 `1d6f1f95c8d64c9d60ef4a7294cf871fe1d2b662`，官方比较为多一个合并提交、文件差异 0。它不是借用父提交的绿灯或旧图：

- [Vue 37772913604](https://github.com/warpdotsys/reader-dev/actions/runs/37772913604) 两实际作业成功：20 项取图守卫／256 项前端 Node 测试全执行，19 核心＋2 管理＋1 子目录实际 Chromium 旅程全部通过、跳过 0；14 组登录几何通过。听书旅程实际 7.092 秒、跳过 0。
- 本机逐张目视本次 artifact `11548559217` 的四张 1280×720 原图：播放与返回账号 A 的卡片背景不透明、所见中文与控件可辨、正文不透过卡片；账号 B 的设置／清理图没有旧卡片，缓存由 1 条／125.0 KB 变为 0 条／0 B。只接受这四张生成图，不扩张成全部 UI 已修好；旧图的淡入根因仍只是推断。
- [Java 37772913607](https://github.com/warpdotsys/reader-dev/actions/runs/37772913607) 自己的两次干净构建逐字节相同：JAR `591ffbce8aba9b3fd36d83c4581f0fdf64848d6a0af0b0860edf82f1e943dcec`、285,649,077 B／1,562 条目，内容／ZIP／缺失差异 0，第二次六任务全执行／75 秒。Python 382 全执行；JVM 170 中 31 环境跳过保留，不把它们说成全部实跑。
- [Full 37772914072](https://github.com/warpdotsys/reader-dev/actions/runs/37772914072) 自己的单 Reader 镜像小报告独立接受：Camoufox 20／helper 3／Chromium 23 实跑及默认 Vue 3／异步／生成书源解析守卫通过，与自己 Java CI 的 JAR 散列相同。峰值 832,471,040 B／PID196，2 CPU／2 GiB／256 PID，触限／OOM0；实际 swap 0，但普通短测配置允许 1 GiB，不能冒充零 swap 长测或最终三方。
- [Native 37772913608](https://github.com/warpdotsys/reader-dev/actions/runs/37772913608) 自己六个实际作业均成功：共享 JAR、两个架构原生构建、两个架构下载后载入运行、同发布导入器双架构消费；Camoufox 20／helper 3／24 个发布导入小文件逐项独立接受，Java／Full／两个 Native 架构的 JAR 都为自己这次 `591ff` 字节。四份原生资源报告峰值最大 872,206,336 B／PID197。本机只取小报告，不下载／重新散列大型原生归档；归档散列与实际 image ID 来源绑定到自己的托管载入及 publisher 回执。新镜像未做本机完整启动或新长测，不能替代 8d 旧镜像的 30 分钟证据。

[本次四图完整散列、自己的回执及范围](evidence/ui-snapshot-settled-8e4f5bf8-2026-10-08.json)与旧图记录分开保存。程序回执不替代人工目视，也不声称本机亲自操作候选页面；58 份用户未提交报告的完整 SHA 再次全匹配。结果回填先保留本地，不为纯文档追加反复触发大型镜像构建。

## 本机限制及未完成项

当前浏览器连接在初始化及重置后各失败一次，均报“failed to write kernel assets: 系统找不到指定的路径。 (os error 3)”。没有取得可操作的 Chrome／内置浏览器状态，因此遵循电脑操作技能停止输入，没有绕过改用私有浏览器协议或把托管截图说成本次亲自打开候选页面。

此前 Ubuntu 中本次隔离运行器的网络内核等待仍在，未收到重启授权，不重复启动。完整 Linux 镜像、本机真实认证和最终三方等范围仍保留，见[本机镜像准备与失败](LOCAL-LOCKED-IMAGE-2026-10-08.md)。无生产变化、正式发布、真实凭据或私有正文读取。

本轮只读复核确认官方 Ubuntu `resolute/main` 的 `uidmap 1:4.17.4-2ubuntu3` 已安装、`newuidmap/newgidmap` 存在；`/etc/subuid` 与 `/etc/subgid` 的 SHA-256 仍各为 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`。安装依赖并未解除已有的 `rtnl_lock` 内核等待，不把依赖齐全说成完整镜像已启动。托管作业仍有旧 action 的 Node 20／setup-java v4 弃用提示，属于待迁移维护警告，不是本次失败；没有为消除提示擅自升级整组依赖。

## 回滚

恢复此前测试类与 Vue 工作流即可停用新取图守卫；它不影响产品运行或数据格式。旧四张图、XML 和散列不删除；新门禁失败时保留报告并确定动画、遮挡或环境原因，不放宽可读性条件、不增加资源预算。
