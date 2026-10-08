# JAR 位级复建门禁

更新日期：2026-10-08。本文不是新版本发布公告，生产服务未修改。

## 已验证的差异与证据边界

源码 `0b60e41535eb7383d0c722de13c376bc1e9d0a6e` 的被测 PR 合并快照为 `c8929acae111232ddc9c5d19c298a00a670c0777`；GitHub compare 显示 ahead 1、changed files 0。自己的四条普通工作流已成功，不借父提交的绿灯：

- [Java Kotlin CI 37722098263](https://github.com/warpdotsys/reader-dev/actions/runs/37722098263)：Python 334 项全执行／10.491 秒；JVM 42 个套件／170 项，其中 31 环境跳过，不计作实际浏览器。
- [Vue 3 preview 37722098324](https://github.com/warpdotsys/reader-dev/actions/runs/37722098324)：核心 19＋管理安全 2＋子目录 1 实际执行、无跳过；TTS 6.527 秒。14 组登录几何独立核对；本次四张 TTS 图未目视，不借父提交的图作本次视觉验收。
- [Full integration 37722098550](https://github.com/warpdotsys/reader-dev/actions/runs/37722098550)：真实 Camoufox 20／helper 3／Chromium 23、默认 UI 包装字节、异步和生成详情均独立核对。JAR SHA-256 `531dcb87b6c8257e59fa8f2fc354ab1539aa73909d559daddc2f4be32028026e`。
- [Native rehearsal 37722098220](https://github.com/warpdotsys/reader-dev/actions/runs/37722098220)：六个实际作业成功；Camoufox 20、helper 3、24 publisher JSON、两架构原生及重载身份接受。共同 JAR SHA-256 `35d0f739af8ec90e99a58d958150a9234391441b30371bbbab514b5f76bc2d99`。

两个 JAR 绑定各自的证据，不能混用。当前只验证了 SHA 不同，**没有取得两个旧 JAR 的逐条目比较，差异唯一原因仍未证明**。旧完整镜像归档未下载到本机，也没有冒充本机重算约 2 GiB 归档的散列。Full 短测峰值 827,486,208 B；Native 最高 843,259,904 B／PID 203，实际 swap 为 0，但普通配置允许 1 GiB swap，不是零 swap 长期容量验收。

本机独立回执分别为 `bef991a6689cf11ca088ea523faeca2020a9ce456befdd0977b6ff99068be630` 和 `7938151d843359477677921d0f0073b7e301b76e8b9375817902dbff20907085`；原 58 用户报告散列均未变化。

## 本增量：配置＋实测门禁，而非配置即证明

正式 Gradle Wrapper 仍为 6.1.1，修改实际 `build.gradle.kts` 中的 `bootJar`：禁用来源文件时间戳，启用可重复条目顺序。不升级 JDK、Kotlin、Spring 或依赖，不修改保留的 `cli.gradle` 替代路径。[Gradle 官方 API](https://docs.gradle.org/current/javadoc/org/gradle/api/tasks/bundling/AbstractArchiveTask.html)注明这两个选项自 3.4 起提供；这里不使用后来新增的归档属性。

普通 Java Kotlin CI 在同一托管 runner 上执行：

1. 构建 Vue 3，运行原测试及第一次 `clean test bootJar --no-build-cache`；最多两个 Gradle worker。
2. 先上传第一次的真实 JVM XML，再将第一次 JAR 复制到 runner 临时目录；第二次 clean 不会毁掉已保存的测试证据。
3. 第二次 `clean bootJar --no-build-cache`，不复用任务输出，不修改或重建第一次 JAR。
4. `verify-reader-jar-reproducibility.py` 比较完整 SHA／长度、所有条目内容、ZIP 元数据、顺序和归档注释；任何差异都失败。有限 JSON 在比较失败时也保存。

依赖下载缓存仍可复用；禁用的是 Gradle 任务输出缓存。两次在同一 runner／JDK／已生成前端下重建通过，最多证明这个范围；不能自动扩大成跨 Windows/Linux、不同 JDK、重新下载依赖、重新构建前端或完整 Docker 镜像的位级一致性。不能用此门禁替代 Native 运行、原件差分或真实登录。

只读比较命令：

```sh
python3 scripts/verify-reader-jar-reproducibility.py --first /path/to/first.jar --second /path/to/second.jar
```

比较器拒绝同一路径／软链接或硬链接指向同一文件、重复条目、缺少 Boot 入口／worker／两套 UI／依赖，以及超出归档大小、条目数、展开字节预算的输入。不解包、不归一化、不删字段；报告仅含有限计数与散列，不含源码内容或异常路径。测试中的小 JAR 都是生成夹具，不冒充真实 Java/Kotlin 编译产物。

新增门禁测试最初 11 项中 1 error：旧 CI 不存在第二次构建步骤；这是新流程缺失基线，不叫旧产品回归。实现后最终 14 项在 Windows、WSL 各实际通过。Windows 全套 348 项中 347 通过、1 POSIX 专属跳过；Node 发布契约及现有源脚本／DOM 语言测试共 119 项全执行通过。Shell／YAML及发布静态守卫通过。托管真实构建结果另行回填，未执行的结果不预填通过。

## 首轮真实执行的失败与精确预算修正

源码 `0622447c6f34f4b4b53a214b4383dc85c860db19`／合并快照 `62e4e189d6d857f874b40cdec2a782092378f854` 的 [Java CI 37725173365](https://github.com/warpdotsys/reader-dev/actions/runs/37725173365)实际失败。第一次构建／测试、JVM XML 保存及第二次干净构建均成功；第二次六个 Gradle 任务真实执行、52 秒。比较器报告 `InvalidReproducibilityInput`，失败 JSON 保存成功；还没有条目比较结果，不是“两次字节不一致”的观察，也没有把作业改成可忽略。

本机对现有 `build/libs/reader-4.0.7.jar` 做只读检查，初版比较器同样在展开预算处拒绝。归档 SHA-256 `f9c9082a352d0d609f3a82497f4d4b0ed2907da72bec833a962ce5948a184f2b`，285,688,820 B、1,565 条目，声明展开总量 310,874,253 B，最大条目为 `BOOT-INF/lib/driver-bundle-1.63.0.jar`／203,821,698 B，未加密；有 30 种条目时间戳。该本机归档不冒充上述 CI 的产物，也不唯一证明托管失败原因。

`build.gradle.kts` 实际固定 Playwright 1.63.0。修正仅给精确条目 `BOOT-INF/lib/driver-bundle-1.63.0.jar` 256 MiB 预算；其他条目仍 128 MiB，归档仍 512 MiB、总展开仍 1 GiB、条目数仍 40,000。不是取消大小守卫、允许任意 driver 版本或改字节相等要求。新增三个测试覆盖精确名称、其他版本／应用条目拒绝、专属上限、总预算、加密和真实依赖版本绑定；17 项在 Windows／WSL 全执行通过。修正后同一本机归档通过只读检查，不据此宣布真正两次构建通过。

首轮失败继续保留；修正后的托管执行须用自己的提交和两次真实干净构建另验。其他三条 `0622447c` 的工作流仍按各自状态接受，不能以其中绿灯抵消 Java 复建门禁的失败。

修正后 Windows 全套实际 351 项：350 通过／1 POSIX 跳过／21.046 秒；没有额外浏览器下载或本机 JAR 重建。首轮失败 JSON SHA-256 `4a60c84f0d31a23d347522db84a5ebb650985f274e84f9b18757d32f88d68edd`，准确预算与本机归档身份见[机器证据](evidence/jar-reproducibility-pinned-driver-budget-2026-10-08.json)。58 用户报告散列仍不变。

## 修正后的真实重建已通过，范围仍有限

提交 `7fea00a601e97a2894a77346fa1a5d4908bbd06e`／被测合并快照 `1e67b0d85f3a578d085d447cac10a9ed7187ffa5` 的 [Java CI 37726229090](https://github.com/warpdotsys/reader-dev/actions/runs/37726229090)终态成功；compare 显示 ahead 1／changed files 0。第二次干净构建 72 秒、六任务全部执行；Python 351 项全执行／10.588 秒。先保存的 42 个 JVM XML／170 项／31 环境跳过已独立核对，没有把跳过计为真实浏览器。

两份真实 JAR 各 285,670,231 B／1,564 条目，完整 SHA-256 均为 `e6ae6b4cf8ad3917398fa8c1bac3616e6a2432d163455ca60ebe334c35ca8951`；条目缺失、内容和 ZIP 元数据差异均 0，顺序、注释和完整字节相同。原报告 SHA-256 `60e8284f8b1c05c662b6bf9f5f884184e38f912ca258b6237746e9bd27e0f3a0`；本机完整独立回执 `3ff7a1d2f3a9f8b9796f5485dd19a36be9b0e0150dd16cf300ead99d49ff8265`，记录 43 个实际下载文件的散列。[有限机器证据](evidence/jar-reproducibility-hosted-7fea00a6-2026-10-08.json)区分 CI 中重算 JAR 与本机仅重算小报告，不冒充本机下载／复算大 JAR。

这证明同一 runner／JDK／既有 Vue 3 dist 的两次干净后端构建，不证明重新生成前端、不同主机或完整镜像的位级一致性。首轮 `0622447c` 的 Full JAR `630a4564c4ad64cebf847fe0a81b230c6021f373e1f048051c94d7e99f6f7c45` 与 Native 共享 JAR `50ba765480574d23beb0c8f21ee702b4f5ac2f507f2679efff8aa6353b83bca4` 在归档设置后仍不同，且来自相同 `62e4e189...` 快照；各自身份仅分别接受，跨流水线唯一差异原因未核验，不能把它全部归为时间戳。后续仍检查编译输出／归档元数据和环境差异；不混用产品字节。

`7fea00a6` 的 Vue／Full／Native 已全部终态成功，自己的完整镜像／原生小报告也已独立接受：Vue 19＋2＋1 实际用例、14 几何、两侧 Camoufox 各 20、helper 各 3、publisher 24 份及完整身份／异步／详情／预算通过。本次没有目视新 UI 截图。Native 独立回执 `fa99b7c4249d96f18db211e3025f7546ab9c1c013f537f16d346f13f59def7ff`，共同 JAR `0f2c206925c45d27e53cbf338161c903bec1699680d1cf7a1a817de6e649a419`，不是 Full `eeb8b718...` 或 Java CI `e6ae6b4c...`；最高 866,697,216 B／PID 210。实际 swap 0 但普通短测配置允许 1 GiB，不是零 swap 长测。原失败、58 用户报告和未完成目标保留，没有合并／发版／部署。

后续 UA 入口提交 `ce12b21bd7ca521eb918608896da7b6a7c42979d`／同树合并快照 `2cf82193a32fa745db9494d93ad46f61e53076dc` 的[Java CI 37728606632](https://github.com/warpdotsys/reader-dev/actions/runs/37728606632)也实际两次干净后端构建成功：第二次六任务全部执行／43 秒；JAR 285,670,232 B／1,564 条目／SHA `20a8ce2b675cc320c2cec04ecab4573ec7d01785467409c2043d680c0c5a6686`，所有差异计数 0、完整字节一致。358 Python／零跳过／9.196 秒，42 JVM XML 中 170 项／31 环境跳过保留。Full `b002ff09...`／Native `641b9392...` 仍分别绑定身份；四条普通工作流及六 Native 作业／publisher 24 份已独立核对，不证明跨流水线复建或实站通过。[该提交自身证据](evidence/rebuild-ui-native-hosted-ce12b21b-2026-10-08.json)。

## 已确认的 Python 资源缓存污染与针对性修正

本机现有历史 JAR `f9c9082a352d0d609f3a82497f4d4b0ed2907da72bec833a962ce5948a184f2b` 只读检查确实包含 `BOOT-INF/classes/camoufox/__pycache__/` 及两份缓存：`worker.cpython-311.pyc`／25,735 B／SHA `2b24ceeaebc7d554ac0bef24cc8099f4a9e195750e6a1ab2e13f98bb841af9b8`，`worker.cpython-314.pyc`／44,895 B／SHA `99fc67ce93fdb35399467b29e1dd65a2026c898ea43ea64d234fdbacb3c6a69b`。本机资源目录也保留这两文件。测试通过 `importlib` 导入可读 worker，旧 `processResources` 没排除 Python 缓存。这证明本机产物污染，不冒充已下载／比较托管大 JAR，亦不能唯一认定它导致全部跨流水线差异。

修正正式 `processResources` 排除 `**/__pycache__/**`、`**/*.pyc`、`**/*.pyo`，不删除磁盘缓存、原 JAR、源码或用户数据；可读 worker、两套前端、类和完整依赖仍打包。比较器同时拒绝带应用资源缓存的输入，即使两份污染 JAR 逐字节相同也不能通过。**不是比较时忽略差异或归一化旧文件**；完整字节／条目／时间／顺序及所有旧预算守卫保留。相似名字的普通文本、读取源码和嵌套依赖 JAR 不被排除，仍比较完整字节。

新增四项定向用例先在旧实现实际出现五个断言失败，21 项修正后在 Windows／WSL 全执行通过。Windows 全套 362 项／361 通过／1 POSIX 跳过／19.581 秒；发布 Node 82 项全执行、原静态门禁通过。本机没有重新编译大 JAR／下载浏览器；正式 KTS 编译和新产物无缓存的真实检查交给本增量自己的托管 runner，结果尚未预填通过。[本机有限证据](evidence/python-resource-cache-exclusion-local-2026-10-08.json)。

回退只撤销正式资源排除、缓存拒绝守卫及相关测试，不变更原归档顺序／时间设置、预算或用户文件。回退会重新允许缓存污染，必须保留这项已知风险；旧失败和旧产物不删除。

## 未解决项与回退

本次没有修复匿名起点空元数据、真实认证、严格 UTF-8 历史差异、最终三方或长期容量。无额外等待脚本的匿名结构专项仍失败，详见[该专项独立结果](PUBLIC-METADATA-SNAPSHOT-STRUCTURE-2026-10-07.md#2026-10-08实际结构观察仍是业务红灯)。不改生产、代理、UID/GID 范围，日常 Reader 和原 JAR 不变；官方 uidmap 已安装也不等于本机最终镜像验收。

若需要回退，只撤销 `bootJar` 两项归档设置和 Java CI 第二次构建／比较步骤以及相关比较器测试；保留旧失败和身份报告。没有数据迁移、用户文件删除或工作区重置。
