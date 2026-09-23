# 本地 TXT 导入、阅读与进度差分

验证日期：2026-09-23（Asia/Shanghai）。原始 JAR 与恢复版分别运行在随机端口和独立临时工作目录，使用临时账号与自生成的两章 TXT；不接触生产账号或存储。运行：

```powershell
pwsh -NoProfile -File .\scripts\build.ps1
pwsh -NoProfile -File .\scripts\compare-file-lifecycle.ps1
```

本次原始 JAR SHA-256：`B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C`；最新复测恢复构建 SHA-256：`2CE4C1ABA8B40471A036C266F7E4ECFC586EB91C83878CB246194B21C25DA8EF`。

## 已从 JAR 验证并成功重建

文件链路的 17 项探针全部一致；新增的本地阅读 8 项中，6 项与原 JAR 语义一致：TXT 保存、`file/parse?import=1` 导入、书架初始记录、两章目录、第一章正文、阅读进度写入。正文比较完整字符串，包含标题、换行和首段空白；目录比较完整 JSON 字段与默认值。书架记录中的动态毫秒时间只比较数值类型，其它字段逐项比较。进度写入后，书架记录显示 `durChapterIndex=0`、`totalChapterNum=2`。

差分初跑发现恢复版章节 JSON 多出内部 `_userNameSpace` 字段；该运行时字段现标记为 `@Transient`，不再进入 Gson 响应。恢复版 TXT 内容原先去掉标题并添加全角缩进；现返回原 JAR 同样的原始章节文本。两个行为均有源码级单元测试，并经重建和黑盒复测。

## 已审查但不伪称一致的 2 项

| 探针 | 原 JAR | 恢复版 | 处理 |
| --- | --- | --- | --- |
| `getBookInfo` 读取已在书架的本地书 | `isSuccess=false`，`未配置书源` | 成功返回本地书对象 | 保留恢复版修复；避免为匹配旧错误而破坏本地书信息接口 |
| 阅读后书架进度标题 | 章节数、索引等一致，但标题字符串乱码 | 正常中文标题 | 保留恢复版修复；动态时间不比较字节值，除标题外的字段一致 |

脚本只接受这两种精确界定的差异，结果标记 `acceptedDivergence=true`；出现其它差异会失败。逐项机器可读结果见 `reports/file-lifecycle-diff-latest.json`。这不是“25/25 完全等同原 JAR”，而是 23 项一致、2 项有记录的改进。

## 尚未验证

- 仅使用一个 ASCII 文件名、两章 UTF-8 TXT；未覆盖中文文件名的实际阅读、EPUB/PDF/CBZ、超大 TXT、不同编码、跨章节缓存、导出或书源切换。
- 未覆盖 SSE 搜索、真实第三方书源和生产书籍的端到端行为。
- 本测试验证写后读取和进度字段，不代表所有生产数据或 WebDAV 同步已兼容。
