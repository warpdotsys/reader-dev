# 当前恢复产物：生成请求、脚本、POST 和 Cookie 三方实测

记录日期：2026-10-12。使用生成样本、无外网隔离环境，实际执行原始 JAR、历史 WebView 和当前恢复 JAR；不使用真实凭据、不读取私人书籍。

## 输入与实际结果

原 JAR 完整 SHA-256 为 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`。恢复 JAR 为产品源码 `6a133dfe`、受测合并快照 `08df81bf` 的准确正式产物，完整 SHA-256 为 `30e8cafcea6b07781aae8039c5ae0230340c189884f61eba445bab72e5b440af`。本轮不是仅根据相同类文件推断兼容。

历史引擎沿用已核验 manifest `88c25004…573`；Camoufox 库/浏览器来自缓存 runtime `401d895c…44`，运行准确当前恢复 JAR。缓存运行时并非最新完整原生镜像，不能将本轮称为最新整镜像本机验收。

三侧各实际完成五个搜索场景，以及一次延迟生成 DOM 的 `getBookInfo`。完整五组搜索 ReturnData 的状态、字段、类型、默认值相等；实际 GET/POST 方法、`q=post` 请求体及生成测试头符合对照，POST 目标实收媒体类型均为 `application/x-www-form-urlencoded; charset=UTF-8`。生成脚本的结果也参与完整响应比较。

已知 Cookie 行为差异保持可见：原件与历史路径的五次目标 Cookie 均为空；当前内置引擎在第二次回放生成的 Cookie，第三次已删除，后续两次为空。不是恢复历史缺失行为或冒称三侧 Cookie 字面相等。

延迟元数据的完整非时钟 ReturnData 相等，三个默认时间字段各在自身实际请求窗口内；完整原始 JSON 字面仍不同，未抹去差异。元数据中的书名来自自造 DOM，不是读取《黎明之剑》EPUB 或其他本地正文。源规则往返保留、生成 Cookie/请求字段及五组完整响应由比较器再次独立检查。

## 隔离、预算和清理

两测试运行时均为 UID/GID 10001、空附加组、零 capabilities、no-new-privileges。仅共享私有 user/net 命名空间，mount/pid/ipc/uts 独立；网络只存在 lo，输入只读。历史引擎先确认 stopped/PID0，再允许当前 Camoufox 阶段，没有放开外网或扩大资源预算。

整轮 50.091 秒；服务运行 53.863 秒、CPU 累计 61.761 秒。合计限制为 2 核、2 GiB、high 1.5 GiB、256 PID，配置与实际 swap 均为零。峰值 1,611,694,080 字节、243 PID；soft-high 3666，硬内存/OOM/PID 触限均为零。仍只有 13 个 PID 余量，不能据此关闭真实大书压力或容量问题。

两个自有运行时均已 stopped/PID0/delete，inventory 为零，服务另确认 inactive/dead/MainPID0/Result=success。随后只精确清理了这次测试的两个只读 JAR 输入副本，共 358,586,921 字节（约 342 MiB），可从保留源重新复制；没有递归删除目录。原 JAR、只读备份、下载的恢复 JAR、授权单书和 58 份用户报告均保留。

[有限结果摘要](evidence/generated-current-cookie-three-way-2026-10-12.json) SHA-256 为 `3997628d57bc165bcebecf70fc283a6246cd3550fd1f83343cb58bc453e8cb20`，绑定本机原始回执 SHA-256 `416de61ba8970a5510276bbb221753cfb99835bfcec1660bf87fa4242c8701d8`。只导出有限事实，不导出正文、请求头值或原运行日志。原失败/旧回执未反写。

## 尚未验证

这不是实站 HTTPS、真实认证、跨源请求头严格安全三方、真实大书合计压力或生产验收。旧跨源凭据泄漏、UTF-8 截断等有意安全/兼容差异仍保留，不为追求全相等而恢复漏洞；旧超时和 POST 类型故障的唯一根因也未由本轮证明。

维护提交 `7daa2c6a` 的 [Java/Kotlin 托管构建](https://github.com/warpdotsys/reader-dev/actions/runs/38198554839) 已完成，GitHub 自带 Ubuntu 24.04 runner 执行 732 项 Python 回归、13.367 秒、零跳过；此前失败的优化模式安全预检已通过。记录时完整镜像和界面测试仍在运行，Native 仍待运行，不能提前宣布四条均通过。本地三方结果不代替这些流程。本记录和有限摘要先保存在本地，未为记录结果额外触发新一轮构建。

没有合并、发版或部署；goal 保持 active。回退限独立干净工作树的源码版本，保留原件和本轮证据，不 reset 用户工作区，也不放宽断言、隔离、超时或预算。
