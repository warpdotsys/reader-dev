# Camoufox 只读快照与导航竞争

## 实际失败：不能用另一侧绿灯覆盖

`0e8e00ea` 修正源规则 load 时机后，自身两份真实新增契约都通过：Native 7.081 秒、完整镜像 7.516 秒。完整镜像 18 项均实际通过；但 Native 18 项中 1 失败，发布守卫拒绝，未导出共享 JAR，也未执行双架构／重载／发布导入。失败用例是 `endlessGeneratedNavigationTimesOutAndTheNextRenderRecovers`，8.198 秒；它实际失败于要求 worker TimeoutError 的断言，不能因此改成“任意错误都算超时”。[独立下载的 XML、散列与范围](evidence/source-load-fix-hosted-2026-10-06.json)。

失败前约 102 毫秒，托管作业的既有固定诊断实际记录 `snapshotContent / documentChanging / Error`。只提取固定字段，不保存／提交原日志；不是 SourceScriptStateLost 或初始 goto 超时的推断。该用例没有源脚本，load 分支未作用到它；不据时间顺序称已证明所有实站故障的唯一原因。

Playwright [固定 v1.62.0 的 frames.ts](https://github.com/microsoft/playwright/blob/v1.62.0/packages/playwright-core/src/server/frames.ts) 在只读 content 操作失败时区分不可重试异常，并生成已观察的导航／内容变化错误。这是诊断与窄范围恢复依据，不代表可以重做导航、表单或源脚本。

## 先复现，再有界修复

在旧 worker 上先执行生成纯测试：53 项中 4 个实际 Error，0 assertion failure；分别为有限内容竞争、持续竞争、预算已到期、竞争期间网络拒绝。另两项“相似消息但其他类型／其他浏览器错误”和“错误消息不可读取”保持原异常，原有测试不变。它们是 mock 事件／时钟，不是真实浏览器复验。

修复只在 MainDocumentSnapshot 的 `page.content()` 周围接受类型名 Error 和固定完整导航变化句子；其他错误仍原样 fatal。再次检查网络拒绝和原单调截止时间，更新 generation／200 毫秒 quiet window，再回到同一只读循环。不新增 goto、POST、setContent、规则执行、页面修改或 HTTP 请求，不重设预算，也不输出原错误文本。源脚本、sourceRegex、Cookie 和 load 规则选择不改；字节上限仍在成功读回后执行。

新增 6 项纯测试核对最终文档、持续竞争只消耗同一预算、过期不再读、网络拒绝优先、其他错误不重试及不可打印错误不替换原异常。修改后本地 Python 203 项中 202 实际通过、1 Windows 环境跳过，0 失败／错误，12.065 秒；Node 发布结构／导入／manifest 57 项实际全通过、无跳过。JDK 11 离线 `processResources compileTestKotlin` 实际成功，2 分 41 秒，保持 2 活动处理器／512 MiB／单 worker；是资源复制和编译，不是 JAR 启动或真实浏览器验证。下面的真实引擎验收来自此修复自己的新运行，未复用 `0e8e00ea` 的另一侧绿灯。

## 本提交自己的正常托管验收

源码 `01459443`、被测 PR 合并快照 `1d93cc49...` 的四条正常工作流均成功，Native 六个实际作业均成功。独立下载的两份真实 Camoufox XML 各 18 项，0 失败／错误／实际 skipped 标记；原精确具名守卫分别再次通过。无限导航用例 10.162／9.825 秒，仍严格要求 worker TimeoutError、原始 POST 不重放、后续新上下文恢复与请求停止；源规则 load 用例 7.097／6.978 秒、早期导航 3.061／3.079 秒。共享 JAR 导出前的包内 worker 比对及原守卫步骤确实执行成功。[工作流、作业、实际 XML 散列与限制](evidence/snapshot-race-fix-hosted-2026-10-06.json)。

Java 普通作业独立读到 42 套／161 个 case，实际跳过 29，执行 132；详情解析 9、存储读取一致性 3 实际通过。普通作业跳过的 18 个 Camoufox case 不当作真实引擎通过。Vue 3 核心 18＋子目录 1 的真实 XML 无失败／错误／跳过，仅覆盖隔离生成账号和生成数据；未以此证明生产登录，也未独立审阅这一个候选的截图。

20 份小型发布导入 JSON 的精确文件集合、身份、散列与生成基线独立核对，四次原异步守卫及四次原预算守卫实际通过；AMD64／ARM64 同一 JAR 身份 `210df8b4...`，两侧分别原生构建、传输重载、正式发布导入器消费，未重新构建替代被测制品。最高峰值 889,114,624 B、PID 197，实际 swap 0，正常测试配置仍允许 1 GiB swap，不能称零 swap 配置。只下载小报告，未在本机下载或重散列大镜像／JAR 归档；归档 SHA 属于明确标注的 GitHub／导出元数据。

## 同一个已测试镜像的实站结果仍失败

匿名专项 `37460945863` 消费上述 Native 的 AMD64 镜像，只有一次书籍详情请求，7.19 秒，HTTP 200／`isSuccess=false`／data null；书源六项读回均完整。有限诊断仍为 `sourceScriptRead / sourceStateMissing / SourceScriptStateLost`，等待期间观察到主框架导航，没有页面快照。快照只读竞争修复与 load 夹具绿灯没有修好这个实站失败；导航事件也可能是同页导航，不证明新文档、验证码或唯一根因。[九份 JSON 的实际散列、身份、失败及清理](evidence/public-metadata-load-snapshot-fix-hosted-2026-10-06.json)。

这次实站专项原零 swap 守卫实际通过：2 CPU／2 GiB／PID 256、配置及实际 swap 0，峰值 855,392,256 B／PID 189，触限／OOM 事件均 0。UID 10001、私网守卫开启、不发布端口，Cookie 0→0，生成测试会话撤销及容器清理已核验；不宣称全部设备令牌撤销。未导入真实凭据、请求章节正文、操作验证码或改生产。用户点开的免费首章在内置浏览器里可见且无验证码遮挡，但仍保留验证码 iframe；没有导出该浏览器会话，不能用它回填 Reader 后端成功。

## 已知限制和回滚

有限截图／load 事件不是任意未来动态页面“全加载”证明；初始导航被打断、源脚本开始后丢失状态、其他 driver／关闭／transport 异常仍可能失败，不能静默恢复为成功。起点 Reader 解析仍红灯；认证三方、原 JAR／历史引擎同输入、本候选新的长期测试及生产仍未由此完成。下一步需要区分文档替换与同页状态删除，不能靠重放源规则、POST 或重复外站请求把诊断变成成功。未放宽 DNS／私网规则、预算，未读取私有正文、使用旧凭据或改生产。

回滚仅撤回 `is_read_only_content_navigation_race` 和 content 周围的捕获／等待分支，保留生成回归、18 项实际门禁和失败证据，以免把该缺口隐去。不用撤门禁、重跑直到偶然绿灯或放大资源预算代替修复。
