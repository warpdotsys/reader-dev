# 大镜像制品传输失败与精确回归（2026-10-07）

## 已实测，不混为详情失败

`3eb418c9`／受测 `15ed874a...`：Java、Vue、Full image 成功；原生共享 JAR、AMD64／ARM64 构建和 ARM64 重载成功，但 Native workflow `37612911968` 总体失败、publisher 跳过。

失败发生在 AMD64 重载作业 `112768265079` 的 `Verify transferred bytes and the actually loaded image without rebuilding`，`ENOENT imported/metadata.json`。前一下载步骤报 success，但没有输出该大制品的最终下载 digest／完成日志；没有执行 Docker load 或 AMD64 重载浏览器。不能用原生构建的通过结果替代它。

上传步骤记录八个文件、平坦 exported 根目录并成功 finalized。仅 HTTP Range 读取制品 `11479257168` 的 ZIP 末尾目录和六个小 JSON，共 71,004 B／镜像正文 0 B；所有目录名称／局部头／长度／JSON CRC 验证通过，确实存在 `metadata.json`、生成详情通过回执和其他报告。因此不是 producer 漏写 metadata；完整 ZIP digest、镜像正文和失败 runner 文件系统没有在本机独立复查。

ZIP 总长 2,174,844,316 B，首项镜像归档 2,174,837,843 B，后续 metadata 位于偏移 2,174,838,209；ARM64 总长 2,113,106,272 B 并已重载通过。跨 2 GiB 是诊断线索，不是已经唯一证实的解析器阈值或 Node 版本根因。[有限目录与元数据证据](evidence/artifact-transfer-failure-3eb418c9-2026-10-07.json)。

## 修复候选与严格回归

改用官方 `actions/download-artifact` v8.0.2 的完整 commit `9000827ccba6bdab643e8b6fd33ac0654aef8333`。其 action.yml 明确 Node 24，`digest-mismatch` 默认 error；所有正式／原生／浏览器消费显式设 error，并用发布结构守卫拒绝旧组件或 warn／ignore。未关闭 metadata、归档 SHA、共享 JAR、镜像身份、UID、实际浏览器或资源守卫。[官方固定版本](https://github.com/actions/download-artifact/blob/9000827ccba6bdab643e8b6fd33ac0654aef8333/action.yml)。更新本身尚不等于修好。

新增手动复用任务，只经既有 Browser workflow 的 `artifact_download_regression=true` 模式运行。首先核对旧 run 的失败／旧 AMD64 producer 成功、精确 artifact ID／名称／大小和有效期；消费原 `11479257168` 与原共享 JAR，不重新构建或换成小归档。新下载 digest 必须通过，metadata 原散列 `c51a877c...`、完整原归档／JAR／镜像身份必须一致，之后实际运行原镜像、生成详情／Cookie／UI／原预算；只保存小 JSON。旧 Native 失败不追改成成功，专项也不替代新正常双架构／publisher。

## 当前范围与已知问题

- 本机发布结构 46 项通过，四个 YAML 可解析。新组件对原大归档的实际回归和新正常双架构／publisher 尚待执行。
- 生成详情已在 Full、原生两个架构及 ARM64 重载执行；本机已独立读到 Full／AMD64 producer Range／ARM64 transfer 的有限回执，不能声称 AMD64 重载或 publisher 已通过。
- 起点实站空元数据、真实认证三方、原 JAR 新详情差分、当前完整镜像长期／高核数和生产可见流程仍未完成。版本更新不改变这些红灯。
- 新专项只在 GitHub hosted runner 传输旧 2 GiB 归档；本机不下载它，不处理真实书籍、会话、生产或 registry，不使用独立 WebView。
- Actions checkout／setup-java／upload 等仍有 Node 20／旧主版本弃用警告，此次没有扩大到全组件升级；警告不等于本次业务失败。
- 回退方式：反向应用下载组件及专项 workflow 提交，恢复旧 pin，保留失败和有限证据；旧下载缺文件仍必须阻止发版。不要恢复用户存储或覆盖原始 JAR。
