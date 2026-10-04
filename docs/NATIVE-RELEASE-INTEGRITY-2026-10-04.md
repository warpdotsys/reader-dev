# 原生双架构发布与镜像传递验收

## 范围与证据等级

本次继续维护 Java/Kotlin `legacy` 项目，不修改业务实现，不恢复 Go/Rust 重写。
默认分支仍为 `legacy`；修改先进入候选分支 `ci/full-reader-20260926` 的 PR #56。
此前单独 ARM64 浏览器测试不能证明正式发布支持 ARM64，因此不能将旧绿灯当作本次发布路径的验收。

- **已从源码核实**：旧 `release.yml` 正式镜像构建只使用 amd64 runner。本次将其替换为调用同提交的原生双架构可复用流程。
- **已通过本机验证**：提交 `8c103d10` 的 43 项 Node 测试全部通过；两个 YAML 文件可解析；公共容器测试脚本通过 Bash 语法检查，三个涉及的工作流通过已核对官方归档 SHA-256 的 actionlint 1.7.7 检查。生产部署、回滚和 GitHub Release 作业尾段与修改前逐行一致。
- **已从真实托管作业验证**：`8c103d10` 的[首轮原生演练 37182947900](https://github.com/warpdotsys/reader-dev/actions/runs/37182947900)五个作业全部成功；实际测试的是 PR 合并快照 `b364f3f0b9119070dfee03f3e45326738a325bf3`。两次原生构建和两次新 runner 导入后的浏览器与资源检查均实际执行，不是只校验 JSON。该提交的 Java/Kotlin、Vue 3 浏览器与完整镜像工作流也均成功。
- **已实跑的后续消费端**：`59a356ef` 的[六作业演练 37183779521](https://github.com/warpdotsys/reader-dev/actions/runs/37183779521)全部成功；测试合并快照为 `bd27cac067678611a94790a020eddea474ebbf37`。两种架构在新 runner 上再次实际运行，然后第六个作业在一个 runner 上依次导入两份镜像、比较传递后的元数据并成功收集平铺报告。该提交的三道常规门禁也全部成功。
- **本地证据核对发现并修正的问题**：首轮及 `59a356ef` 的版本条件确实在容器上通过，但 `RELEASE_IDENTITY.json` 保存的是 jq 谓词返回的 `true`，并非实际版本对象。下载消费端证据后的字段核对因此失败，不能声称 20 个原始报告已全部通过本机核对。后续将输出改为 `select(...)` 保留真实对象，并让消费端同时拒绝原始/传递后报告中的布尔占位；没有根据预期值伪造或补写旧报告。本机新增负向用例后 **47 项测试全部通过**，Bash 与 actionlint 再次通过。
- **版本对象修正已实跑并核对**：`b3b0e889` 的[六作业演练 37184933190](https://github.com/warpdotsys/reader-dev/actions/runs/37184933190)全部成功，实际测试合并快照为 `68c9a842ffcaf49444ff90abe42a3f17afbbf68b`。只下载 7,034 B 的消费端证据制品，逐项读取 20 份实际 JSON：四份版本报告均为真实对象，四份容器 JAR 哈希相同，导入 image ID 一致，Cookie/GET/POST/脚本/四用户及资源字段核对通过。记录见[本地验收摘要](evidence/native-release-b3b0e889-2026-10-04.json)。该提交的 Java/Kotlin、Vue 3 和完整镜像工作流也全部成功。
- **尚未验证**：正式 registry 发布/生产部署尚未执行；后续源码修改须按各自提交重新验证。不能用此前运行证明后续修改，也不能由静态检查代替正式发布。

后续修改的本机 45 项 Node 测试全部通过，两个公共脚本通过 Bash 语法检查，三个工作流再次通过 actionlint。Vue 3 首轮制品下载的核心 JUnit 共 11 项、0 跳过/失败/错误；已目视核对生成 EPUB 的中文嵌套样式、超时后普通阅读恢复入口截图。这是隔离生成数据和当前候选的验证，不是生产或全部正文验收。

首轮下载的小型原始证据为两份 `reader-native-transfer-*` 制品，本机没有下载完整镜像归档。两侧 JAR SHA-256 均为 `1f68287ee520c8b20d4823e857ee3408af9688a599209da306def31985b829eb`；容器内实际 JAR 哈希也一致。amd64 image ID 为 `sha256:93c4cb4968cb96198271582fd3e37e5d3bd1bcf3e4cd88f5acb47200a024c290`，arm64 为 `sha256:0fe3de3ef15d0d8025591fe9635161f11f7fa8f82527d7d3fa9eb1b80bf140c0`，均与导入后的实际 ID 一致。

| 原生平台 | 完整镜像字节数 | 构建后短时内存峰值 / 任务峰值 | 新 runner 导入后短时峰值 / 任务峰值 |
| --- | ---: | --- | --- |
| amd64 / X64 / x86_64 | 4224387503 | 766533632 B / 184 | 771514368 B / 177 |
| arm64 / ARM64 / aarch64 | 4054902041 | 769236992 B / 176 | 856514560 B / 176 |

四次容器测试的 OOM 和进程限额事件均为 0，均使用 2 CPU / 2 GiB / 256 任务预算；这是短时生成样本，不是长期负载保证。

`b3b0e889` 复验的共享 JAR SHA-256 为 `2d6172a95a8c7bedf6f3903c7a54730abc4f9d2df6adcd9a162975cbc0e32593`；这是不同合并快照的新产物，不能用首轮哈希代替它。amd64 构建/重新导入峰值分别为 838,750,208 / 820,375,552 B，PIDs 峰值 190 / 192；arm64 分别为 804,544,512 / 837,013,504 B，PIDs 179 / 176，限额事件仍均为 0。完整归档仍只由 GitHub runner 处理，本机字段核对不等于本机再次运行完整镜像。

## 实际发布路径

1. `release-native.yml` 的 `build-jar` 使用 JDK 11、Node 24、锁定依赖构建一次 Vue 3 + Java/Kotlin JAR，执行 Python、前端和后端测试。保存源码 SHA 对应的 JAR 与使用文件名而非工作区路径的 `SHA256SUMS`。
2. `native-images` 的 amd64/arm64 作业分别使用 `ubuntu-24.04`、`ubuntu-24.04-arm`，检查 runner 架构与内核架构；下载同一份 JAR，核实字节哈希后从锁定镜像和 Python wheel 构建单个完整 Reader 镜像。没有 QEMU、自托管 runner 或运行时宿主 Chrome。
3. 公共脚本 `smoke-native-release.sh` 使用新的生成书源、临时账号和独立存储，验证实际容器启动、版本与源码标识、中文字体、GET/POST、脚本、Cookie 与四用户隔离。对容器内正在运行的 `/app/reader.jar` 实测 SHA-256 并与共享 JAR 比较，不仅信任构建参数。只监听回环地址，使用 2 CPU、2 GiB 内存、256 进程限额、临时 `/tmp`、非 root 用户及禁止新增权限；随后断言实际 cgroup 资源数据。
4. 测试全部成功后才导出该镜像，记录 JAR SHA-256、镜像归档 SHA-256、架构、版本、源码 SHA 和实测 image ID。大镜像使用流式哈希，不整包载入内存。归档只保留三天，且仅在 GitHub runner 内传递；本机不下载或压缩这些大镜像。
5. `verify-transferred-images` 等待两个构建作业均完成，在两个新原生 runner 上验证归档字节、`docker load` 后的架构/image ID/非 root 用户/入口/版本标识，然后在全新存储和账号上再次执行同一个完整容器测试。不能用单纯解包成功代替实际运行。
   传递后报告统一复制到 `transferred/` 再上传，保持制品根目录平铺，与正式发布作业的读取路径一致。混合 `imported/` 与 `transferred/` 上传会保留两个目录的层级，静态结构检查与专门负向测试拒绝这种配置。
6. 正式 `release.yml` 必须等待同提交 Vue 3 浏览器测试，以及上述全部构建和传递测试完成。发布作业只导入并推送已经测试过的镜像，不重新构建；两种架构的校验完成之前不得登录 registry。
   新增的第六个 `verify-publisher-imports` 作业还会在一个无凭据 runner 上依次导入两种镜像，用与正式发布完全相同的 `import-native-release.sh` 比较传递后的真实元数据、收集报告并只删除已验证的临时镜像归档，实际验证制品目录衔接和两种镜像共存，而不是仅检查路径字符串。它仍不推送任何 registry。
7. 每个架构先在 GHCR 与 Docker Hub 取得相同的原生 digest，再组成双平台 manifest。校验器要求恰好包含 `linux/amd64`、`linux/arm64`，且 descriptor 的 digest 必须对应实测原生镜像；拒绝多余/缺失/重复平台、嵌套 index、外部 blob URL 等。两个 registry 的最终 index digest 一致且可匿名读取后，才继续正式 Release 和生产部署。

可复用工作流使用同仓库的相对路径，随调用提交固定；不依赖矩阵最后一个完成作业的输出汇集两个 digest。
参考：[GitHub 可复用工作流](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows)、[Docker 镜像导入](https://docs.docker.com/reference/cli/docker/image/load/)、[Docker manifest 组合](https://docs.docker.com/reference/cli/docker/buildx/imagetools/create/)。

## 可重复的本机校验

在项目根目录执行：

```powershell
node scripts/check-release-pipeline.mjs
node --test scripts/check-release-pipeline.test.mjs scripts/release-native-artifacts.test.mjs scripts/verify-release-manifest.test.mjs
```

其中 CLI 单元测试使用生成的小型文本夹具来证明哈希和拒绝篡改逻辑。它们**不是可运行 Docker 镜像**，不证明 Camoufox、镜像导出/导入或真实书源兼容性。
结构检查也不是任意恶意 YAML 的安全证明；实际 Actions 执行与制品仍须验收。

## 本轮不触及的数据与已知缺口

- 不复制、读取或上传任何真实书籍正文、生产用户数据、密码、accessToken、Cookie；没有新增生产、registry、标签或 Release 写入。
- 现有未提交 `reports/` 与 `.gradle-user-home/` 不纳入本轮提交、不覆盖、不删除。
- 已核实并停止此前由代理启动的隔离测试进程 PID 2172（端口 18937），保留其所有测试文件；用户端口 18931 不受影响。
- 合成登录/Cookie 用例不能证明真实需登录书源的原 JAR/旧远程 WebView/内置浏览器三方兼容性。
- 字体存在不证明所有特殊字体/复杂 EPUB 均能正确渲染；有限四用户突发不证明长时间生产负载安全。
- 既有 EPUB 导入不是全事务操作；缓存移动重试只针对短暂 `AccessDenied`，不是对所有 Windows 文件占用问题的修复。
- 当前候选版本不等于已正式发版。发版前仍需同步三个版本字段、撰写对应稳定标签的发布说明并列出已知问题，通过正式 registry 和生产端用户可见验收。
- 2026-10-04 只读查询的正式最新版为 `v6.0.45`；GHCR/Docker Hub 镜像变量及仓库级 Docker Hub 凭据名称存在；四项生产 SSH 配置位于 `production` 环境级 secrets。没有读取密钥值。名称存在不证明值正确或 SSH/registry 登录已成功，仍须由实际正式流程验收。

## 回滚与追溯

本轮未改变生产恢复逻辑：保留停服前验证快照、失败态救援包、旧镜像固定引用、空间预检和公开版本标识核对。任何生产回滚仍按现有正式流程及项目回滚文档执行，不使用本次生成测试存储。
候选流程出问题时应修复候选提交并重新验证，不覆盖原始 JAR 或用户未提交文件，不强行合并、不将演练产物伪装成稳定 Release。
正式产物会附带两种架构的原始与传递后浏览器/资源/镜像标识证据、JAR 校验和、基础镜像锁及最终 registry index JSON，供逐项核对。
