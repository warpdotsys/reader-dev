# 实际 Reader HTTPS 业务门槛：可维护入口与独立验收

日期：2026-10-10。仅生成账号、正文、证书及认证头；没有真实凭据、真实正文或生产写入。默认分支仍为 `legacy`，PR56 为 draft。本轮不合并、发 Release、推 registry 或部署。

后续c29自己的四条托管全部独立接受：新增实际Reader HTTPS8项已在Full及双架构构建/重导入准确镜像真实执行，Native最终publisher32份JSON也逐字段重验。本轮另整理有限证书错误提示候选，本机编译/协议检查已通过、其自己的实际提示尚待新构建。[后续准确状态](CERTIFICATE-ERROR-HINTS-2026-10-10.md)。下文保留该入口提交前的捕获状态。

## 已成功重建：父提交自己的托管结果

源码 `5ecbc609abb66194b96630f375750b880cbcd36d`，实际合并测试快照 `4e3a4d2923351008b37ac4e5d9689020f89e3a92`，API 比较一提交领先、文件差异零。下列任务均为 GitHub 托管 runner；独立核对实际作业、产物身份、XML 与原始报告，不借前一提交绿灯。

| 任务 | 此任务实际接受的有限范围 | 独立回执 |
| --- | --- | --- |
| [Java](https://github.com/warpdotsys/reader-dev/actions/runs/38056764981) | 558 Linux Python；198 JVM、44套件、35原环境跳过；两次干净 JAR 位级相同 | [Java 回执](evidence/portable-parent-hosted-java-5ecbc609-2026-10-10.json) |
| [Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/38056764983) | 25实际生成浏览器流程、零跳过；296前端检查与20截图守卫 | [UI / Java 回执](evidence/portable-parent-hosted-ui-java-5ecbc609-2026-10-10.json) |
| [完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38056765026) | 24 Camoufox契约、生成Reader API、打包worker12项HTTPS；普通阶段峰值893,562,880B / PID193 | [Full 回执](evidence/portable-parent-hosted-full-5ecbc609-2026-10-10.json) |
| [原生与重导入](https://github.com/warpdotsys/reader-dev/actions/runs/38056764988) | 六作业；amd64 / arm64构建与重导入、publisher28份JSON，各阶段worker12项HTTPS重验；普通阶段最高895,512,576B / PID205 | [Native 回执](evidence/portable-parent-hosted-native-5ecbc609-2026-10-10.json) |

上述任务的 JAR 都为 `fcd28a8d974666f12de6671570f2435a9161789d29f20314c7e1f28bf8f2285f`，285,671,163B / 1,570条目，与已在本机重算的22f949产物字节相同。5ec的复建记录来自它自己的452B产物11671386957，不是复制前轮记录。大 native 归档没有在本机重新下载计算SHA，保留这一边界。上述流程**尚不含本节新增的托管 Reader HTTPS 业务模式**；33项业务报告守卫是静态记录测试，不冒充真实API执行。

## 已从实际 JAR 验证：复用源码后的两轮真实运行

把原一次性客户端整理为 `scripts/reader_tls_api_client.py`，JAR、worker SHA与浏览器版本由构建传入，去掉固定产物身份。`scripts/smoke-camoufox-tls.py --mode reader-api` 直接启动校验过的实际JAR，用真实注册、登录、书源保存/回读、搜索API；不替换worker、不伪造ReturnData。`--mode worker` 保留原12项直接打包worker回归。

| 新一轮 | 原始观察 | 实测结果 |
| --- | --- | --- |
| d：实际Reader API | [完整原始报告](evidence/portable-reader-tls-business-d-2026-10-10.json) | 8场景、12目标HTTP、14CONNECT、178头字段、2生成用户、2证书负例；53.056秒，峰值1,611,370,496B / PID237 |
| e：共享夹具worker回归 | [完整原始报告](evidence/portable-reader-tls-worker-e-2026-10-10.json) | 12场景、24目标HTTP、334头字段、2证书负例；50.279秒，峰值1,611,325,440B / PID221；本轮没有启动Reader API |

d报告同时由历史rootless包络与新的 `validate_packaged` 独立严格接受；e报告由原worker门槛独立接受。认证只保存操作、状态、字段名及Cookie数量，不保存账号密码/token值。Java PID起始tick、fd/socket inode与代理连接五元组实时匹配。正例完整搜索默认字段不变；证书负例保留实际省略 `data` 的ReturnData形状。同源保留生成认证头，跨源剥离；303转GET、307/308保持原始UTF-8正文，Cookie不跨源/用户。

两轮 UID/GID10001、空能力/no-new-privileges、仅lo、只读root/home/浏览器/JAR，生成区为私有tmpfs；共同2CPU / 2GiB / 256PID、high1.5GiB、零swap。d / e 的 soft high 分别2627 / 995；hard max、OOM及PID触限均0。报告的收尾file缓存不等于峰值RSS拆分，也不是读取或压缩用户书库。Java在外层清理前退出，两轮自有运行时均独立确认PID0、删除、自有清单0。

原始d报告SHA `ca3fff00e7de7e92c37ba6f6d2e458944dd9c988d182242d2195d42e1d89de72`，e报告SHA `c0f307d9b8c66598c9489f4a2c4e0bedd933981426c549b67c68fa1b64c5505f`；b / c初始化失败SHA分别为 `20d52753d0ac3261894e236493b7035ba0fd429edc1be76d68e450b35216e31b` / `f89e2bb90625e089cae14f8f1683583e02c082b7b190878d557a548d2d6a351b`。上述是完整捕获副本的实际字节哈希，不是下载ZIP的哈希；Git对新证据单独保留字节，不改全局换行设置。

运行时仍为旧缓存 `sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44`，只读绑定准确fcd JAR；不是最新整个native镜像的本机验收。本轮测试源码仍是提交前候选，不能将它与父提交的CI混为一轮。

### 保留失败和本机权限适配

[b初始化失败](evidence/portable-reader-tls-init-b-2026-10-10.json) / [c初始化失败](evidence/portable-reader-tls-init-c-2026-10-10.json) 都在Reader/浏览器启动前停止，外层捕获为 `StopIteration`；此前私有stderr分别定位为：读取未选择当前成员组的 `/sys/fs/cgroup/cpu.max` 不存在，以及当前自有client组目录0700阻止映射UID读取只读资源指标。不是TLS证书/业务失败，不能把外层异常当底层原因。两轮各约21秒、44MiB、无触限，自有清理0，原失败不改写为通过。

共享夹具现在按 `/proc/self/cgroup` 选择且限制在已暴露controller树内；Docker私有命名空间根和WSL嵌套成员路径分别用5项明确生成路径夹具检查。d/e本机控制器仅对**当次自己新建的client目录**0700→0755，允许读取指标；父目录及所有指标文件权限不变。完整before/after路径在原始报告中。没有修改controller预算、其他cgroup、UID映射范围、系统代理或生产。临时scope已清理，不通过加root或capability让测试通过。

## 新接入：待自己提交的托管实测

```sh
# 仅GitHub托管runner，使用已构建的准确镜像，报告必须是新的不存在路径。
bash scripts/smoke-native-tls.sh "$image" "$arch" "$revision" "$jar_sha" "$fresh_report" reader-api
```

新增模式继承独立容器的无外网/只读/UID10001/零capability/2CPU/2GiB/256PID/零swap策略，仅增加128MiB私有生成 `/storage` tmpfs。CPU/内存指标从真实当前组采集，Docker的 `memory.high=max` 不假称本机high1.5GiB。生成CA仅在私有distribution临时挂载，宿主信任和不可变镜像不变；不忽略HTTPS错误。

- Full流程：worker12项结束并清理后，再实际执行Reader8项，保存 `bundled-reader-business-tls-generated`；之后才启动普通Reader阶段。
- Native：按worker12 → Reader8 → 普通Reader顺序执行，两架构构建/重导入各有完整 `BROWSER_READER_TLS.json`，不存在多份2GiB预算并发运行。
- Publisher：逐字段重新核验built与transferred两套Reader8项和产物身份；JSON从父流程28份变为32份，不复制历史报告充数。验证失败不能忽略。
- 本机全量586项Python / 32.840秒 / 失败0 / 1原Windows平台跳过，84项流水线检查 / 零跳过；新增45项业务报告守卫、11项客户端守卫、5项cgroup路径守卫是静态/生成双桩，不是额外浏览器运行。

**当前仅接入候选、本机实测通过；新托管真实8项、两架构和重导入尚未取得自己的运行结果。** 后续必须按新源码与实际测试快照分别核验，不能沿用上表5ec的绿灯。

## 已知问题、保护与回退

当前fcd的有限HTTP原件/历史远程/Cam三方结果见[业务报告](READER-HTTPS-BUSINESS-2026-10-10.md#后续补充当前-fcd-的首轮有限-http-三方)，不改写完整三方未接受标志。原件HTTPS及编码扩展、真实站点认证/授权真实书完整旅程、WS/WSS、并发长测/冷启动/崩溃恢复、最新整个镜像本机运行及生产部署仍未齐。旧跨源泄漏不能为了相等恢复；原件正文/getBookGroups编码差异、泛化TLS错误消息与未实现OPDS后端继续公开保留。

保留原JAR/只读备份、所有用户未提交报告、Go/Rust目录、已提交生产逻辑。本轮新门槛仅测试基础设施；回退采用对本轮提交的独立revert，保留原始成功/失败观察，不reset工作树，不删除用户数据，不放宽证书、网络或头隔离。

收尾重新计算58份受保护用户报告及原JAR/只读备份SHA，全部未变；原件b26与备份仍相同，备份仍只读。六份新增/修改Python源码通过3.10语法解析，三个Shell脚本分别通过语法检查。官方uidmap已使用，未修改映射范围。完整项目目标仍未完成。
