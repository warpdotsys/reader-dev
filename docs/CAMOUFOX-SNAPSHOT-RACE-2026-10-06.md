# Camoufox 只读快照与导航竞争

## 实际失败：不能用另一侧绿灯覆盖

`0e8e00ea` 修正源规则 load 时机后，自身两份真实新增契约都通过：Native 7.081 秒、完整镜像 7.516 秒。完整镜像 18 项均实际通过；但 Native 18 项中 1 失败，发布守卫拒绝，未导出共享 JAR，也未执行双架构／重载／发布导入。失败用例是 `endlessGeneratedNavigationTimesOutAndTheNextRenderRecovers`，8.198 秒；它实际失败于要求 worker TimeoutError 的断言，不能因此改成“任意错误都算超时”。[独立下载的 XML、散列与范围](evidence/source-load-fix-hosted-2026-10-06.json)。

失败前约 102 毫秒，托管作业的既有固定诊断实际记录 `snapshotContent / documentChanging / Error`。只提取固定字段，不保存／提交原日志；不是 SourceScriptStateLost 或初始 goto 超时的推断。该用例没有源脚本，load 分支未作用到它；不据时间顺序称已证明所有实站故障的唯一原因。

Playwright [固定 v1.62.0 的 frames.ts](https://github.com/microsoft/playwright/blob/v1.62.0/packages/playwright-core/src/server/frames.ts) 在只读 content 操作失败时区分不可重试异常，并生成已观察的导航／内容变化错误。这是诊断与窄范围恢复依据，不代表可以重做导航、表单或源脚本。

## 先复现，再有界修复

在旧 worker 上先执行生成纯测试：53 项中 4 个实际 Error，0 assertion failure；分别为有限内容竞争、持续竞争、预算已到期、竞争期间网络拒绝。另两项“相似消息但其他类型／其他浏览器错误”和“错误消息不可读取”保持原异常，原有测试不变。它们是 mock 事件／时钟，不是真实浏览器复验。

修复只在 MainDocumentSnapshot 的 `page.content()` 周围接受类型名 Error 和固定完整导航变化句子；其他错误仍原样 fatal。再次检查网络拒绝和原单调截止时间，更新 generation／200 毫秒 quiet window，再回到同一只读循环。不新增 goto、POST、setContent、规则执行、页面修改或 HTTP 请求，不重设预算，也不输出原错误文本。源脚本、sourceRegex、Cookie 和 load 规则选择不改；字节上限仍在成功读回后执行。

新增 6 项纯测试核对最终文档、持续竞争只消耗同一预算、过期不再读、网络拒绝优先、其他错误不重试及不可打印错误不替换原异常。修改后本地 Python 203 项中 202 实际通过、1 Windows 环境跳过，0 失败／错误，12.065 秒；Node 发布结构／导入／manifest 57 项实际全通过、无跳过。JDK 11 离线 `processResources compileTestKotlin` 实际成功，2 分 41 秒，保持 2 活动处理器／512 MiB／单 worker；是资源复制和编译，不是 JAR 启动或真实浏览器验证。真实 18 项、包内 worker 与双架构等仍须此修复自己的新托管结果，不能复用 `0e8e00ea` 的完整镜像绿灯。

## 已知限制和回滚

有限截图／load 事件不是任意未来动态页面“全加载”证明；初始导航被打断、源脚本开始后丢失状态、其他 driver／关闭／transport 异常仍可能失败，不能静默恢复为成功。起点 Reader 解析、认证三方、原 JAR／历史引擎同输入、长期及生产仍未由此完成；新匿名专项尚未派发，因为可消费的正常 Native 制品被门禁阻止。未放宽 DNS／私网规则、预算，未读取私有正文、使用旧凭据或改生产。

回滚仅撤回 `is_read_only_content_navigation_race` 和 content 周围的捕获／等待分支，保留生成回归、18 项实际门禁和失败证据，以免把该缺口隐去。不用撤门禁、重跑直到偶然绿灯或放大资源预算代替修复。
