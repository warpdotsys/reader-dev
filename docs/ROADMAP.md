# Reader-dev 路线图（Roadmap）

> 状态：远期规划（非当前实施项）。当前主线：JAR 兼容维护 + 自托管许可系统。
> 更新日期：2026-08-02

---

## 远期计划（按优先级）

### 1. Rust 重写（终极目标）

**目标**：将服务端从 Kotlin/JVM（Vert.x）重写为 Rust，获得：
- 单二进制部署（无 JVM 依赖、启动毫秒级、内存占用大幅降低）
- 高并发低资源消耗（参考 [Maple0517/reader-next](https://github.com/Maple0517/reader-next) 的 Rust 路线验证）
- 更安全的依赖链（Rust 生态 CVE 面小）

**参考**：
- `Maple0517/reader-next`：reader-rust 独立续作，Rust 120 万行 + Vue 3 前端，已实现书源解析/书架/本地书/AI 功能
- `givenge/reader-rust`：上游 Rust 移植

**产物策略（双形态，行业最佳实践：Gitea/Traefik 同款）**：
- **形态 1：scratch 镜像**（`FROM scratch` + COPY 静态二进制）——生产主部署（148 容器形态不变：reader_share_net/storage 卷/pangolin/回滚机制）
  - 镜像仅 1 个文件，无 OS 层/shell/包管理器 → **系统层 CVE = 0**（终结现 Alpine apk 维护），仅剩 Rust 依赖层（cargo audit，CVE 面极小）
- **形态 2：裸静态二进制**（`x86_64-unknown-linux-musl` 编译）——GitHub Release 附件分发，`scp` 即可部署 + systemd 示例 unit
- **前端二进制内嵌**（rust-embed 编译进二进制）→ 真·单文件全功能，镜像形态复用

**实现要点**：
- CA 证书：scratch 镜像 COPY ca-certificates；裸二进制依赖宿主证书或内置
- 时区数据：内置 tzdata（许可到期校验/时间显示）
- 数据目录：两者指向 `storage/`（env 指定），数据格式与迁移工具（远期 2）一致

**约束**：
- **API 兼容**：`/reader3/*` 路径与行为保持与现版一致（客户端无缝切换）
- **数据兼容**：现有 `storage/` 数据可平滑导入（见远期 2）
- **前端可复用**：Vue 前端保留，仅适配新后端 API

**时机**：许可系统稳定运行后评估；可作为独立分支推进，与 Kotlin 版并行维护一段时间。

---

### 2. 数据库性能升级（含兼容迁移）

**目标**：JSON 文件存储（`storage/data/*.json`）升级为高性能数据库：
- **SQLite**（首选）：单文件、零运维、支持事务/索引/并发读，参考 reader-next 的 `storage/reader.db` 方案
- 解决 JSON 存储痛点：全量读写、无索引查询、并发写竞争、大数据量（书源 500000+/书籍多）性能劣化

**兼容迁移（硬性要求）**：
- 提供**一键迁移工具**：现有 JSON（用户/书架/书源/书籍/许可/配置）→ SQLite，**自动检测 + 增量迁移 + 校验**
- 迁移前自动备份 JSON（storage 卷快照）
- 迁移失败可**回滚**（保留原 JSON）
- 许可系统数据（申请/机器登记/吊销）同步迁移

**参考**：reader-next 的 storage 层（SQLite + 文件缓存 + 上传资源分离）。

**时机**：Rust 重写前可作为独立优化（Kotlin 版先上 SQLite）；或并入重写一并完成。

---

### 3. 书源解析多规则（对齐 warpdotsys/legado）

**目标**：书源解析从单一规则体系扩展为**多规则引擎**：
- **CSS Selector / JSONPath / XPath / Regex / JavaScript** 五种规则类型（对齐 legado 生态的 analyzeRule）
- 实现上**对齐 [warpdotsys/legado](https://github.com/warpdotsys/legado)**（阅读Sigma，gedoor/legado 分支）：
  - legado 是 **Kotlin** 技术栈，与现版（Kotlin/JVM）**可直接移植** analyzeRule 实现
  - 移植其规则解析/执行核心（CSS/JSONPath/XPath/Regex/JS 调度）
  - 兼容 legado 书源规则语法（让 legado 书源可直接导入使用）

**现有基础**：已对齐 reader-pro JAR 的 analyzeRule（字节码级审计），作为兼容基线；
**扩展方式**：在保持现有规则行为不变的前提下**增量引入**多规则支持（向后兼容，旧书源不受影响）。

**参考**：`warpdotsys/legado`（阅读Sigma）analyzeRule 实现、`Maple0517/reader-next` 的 parser 层（Rust 版多规则）。

**时机**：可先于 Rust 重写在 Kotlin 版实现（直接移植 legado Kotlin 代码），重写时再迁移。

---

### 4. 本地书籍格式扩展

**目标**：本地书籍支持从现有限制扩展：
- 现有：TXT / EPUB / PDF / CBZ（对齐 reader-pro JAR）
- 扩展：**MOBI / AZW3 / AZW / FB2**（Kindle 系格式优先，参考 reader-next 已支持 MOBI 的解析方案）
- 配套：格式转换（上传时归一化为内部格式）、章节解析优化、大文件分章性能

**参考**：`Maple0517/reader-next` 的本地书籍支持（TXT/EPUB/MOBI/PDF + 跨设备进度）。

**时机**：与多规则解析解耦，可独立推进。

---

## 依赖关系

```
远期 3（多规则解析）──┐
远期 4（书籍格式）  ──┼──▶ 远期 1（Rust 重写吸收全部能力）
远期 2（SQLite）   ──┘
```

- 远期 3/4 可先于重写在 Kotlin 版落地（Kotlin 生态直接复用 legado 代码）
- 远期 2 若先落地，重写时需保留迁移工具
- 远期 1 是收敛点：重写版继承全部远期能力 + 许可系统

---

## 非远期（当前主线，不列入上表）

- JAR 兼容性维护（reader-pro-3.2.14 对齐）
- CVE 修复与依赖升级
- 发布流水线（GitHub Actions + DE runner + ghcr/Docker Hub）
- 用户/功能**不做许可限制**（决策：永远不限制；`READER_APP_USERLIMIT` 等 env 默认宽松 500000）
