# 原生双架构发布与镜像传递验收

## 范围与证据等级

本次继续维护 Java/Kotlin `legacy` 项目，不修改业务实现，不恢复 Go/Rust 重写。
默认分支仍为 `legacy`；修改先进入候选分支 `ci/full-reader-20260926` 的 PR #56。
此前单独 ARM64 浏览器测试不能证明正式发布支持 ARM64，因此不能将旧绿灯当作本次发布路径的验收。

- **已从源码核实**：旧 `release.yml` 正式镜像构建只使用 amd64 runner。本次将其替换为调用同提交的原生双架构可复用流程。
- **已通过本机验证**：新增校验包含 43 项 Node 测试；两个 YAML 文件可解析；公共容器测试脚本通过 Bash 语法检查，三个涉及的工作流通过已核对官方归档 SHA-256 的 actionlint 1.7.7 检查。生产部署、回滚和 GitHub Release 作业尾段与修改前逐行一致。最终测试结果以实际执行记录为准。
- **尚未验证**：本次新流程在 GitHub runner 上实际完成五个作业、跨 runner 镜像传递后的浏览器行为，以及正式 registry 发布/生产部署。这些必须由实际运行结果补证，不能由静态检查代替。

## 实际发布路径

1. `release-native.yml` 的 `build-jar` 使用 JDK 11、Node 24、锁定依赖构建一次 Vue 3 + Java/Kotlin JAR，执行 Python、前端和后端测试。保存源码 SHA 对应的 JAR 与使用文件名而非工作区路径的 `SHA256SUMS`。
2. `native-images` 的 amd64/arm64 作业分别使用 `ubuntu-24.04`、`ubuntu-24.04-arm`，检查 runner 架构与内核架构；下载同一份 JAR，核实字节哈希后从锁定镜像和 Python wheel 构建单个完整 Reader 镜像。没有 QEMU、自托管 runner 或运行时宿主 Chrome。
3. 公共脚本 `smoke-native-release.sh` 使用新的生成书源、临时账号和独立存储，验证实际容器启动、版本与源码标识、中文字体、GET/POST、脚本、Cookie 与四用户隔离。对容器内正在运行的 `/app/reader.jar` 实测 SHA-256 并与共享 JAR 比较，不仅信任构建参数。只监听回环地址，使用 2 CPU、2 GiB 内存、256 进程限额、临时 `/tmp`、非 root 用户及禁止新增权限；随后断言实际 cgroup 资源数据。
4. 测试全部成功后才导出该镜像，记录 JAR SHA-256、镜像归档 SHA-256、架构、版本、源码 SHA 和实测 image ID。大镜像使用流式哈希，不整包载入内存。归档只保留三天，且仅在 GitHub runner 内传递；本机不下载或压缩这些大镜像。
5. `verify-transferred-images` 等待两个构建作业均完成，在两个新原生 runner 上验证归档字节、`docker load` 后的架构/image ID/非 root 用户/入口/版本标识，然后在全新存储和账号上再次执行同一个完整容器测试。不能用单纯解包成功代替实际运行。
6. 正式 `release.yml` 必须等待同提交 Vue 3 浏览器测试，以及上述全部构建和传递测试完成。发布作业只导入并推送已经测试过的镜像，不重新构建；两种架构的校验完成之前不得登录 registry。
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

## 回滚与追溯

本轮未改变生产恢复逻辑：保留停服前验证快照、失败态救援包、旧镜像固定引用、空间预检和公开版本标识核对。任何生产回滚仍按现有正式流程及项目回滚文档执行，不使用本次生成测试存储。
候选流程出问题时应修复候选提交并重新验证，不覆盖原始 JAR 或用户未提交文件，不强行合并、不将演练产物伪装成稳定 Release。
正式产物会附带两种架构的原始与传递后浏览器/资源/镜像标识证据、JAR 校验和、基础镜像锁及最终 registry index JSON，供逐项核对。
