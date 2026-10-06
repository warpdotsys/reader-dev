# 内置浏览器的匿名起点详情验收

## 范围和当前状态

本机仍缺 `uidmap`，不能用宿主 Chrome 或 root 运行冒充最终 UID 10001 的 Camoufox 验收。另建一个明确的托管消费模式：只加载已成功原生演练的完整镜像，不重新构建、替换 JAR、推 registry 或部署。探针源码提交和被测镜像内的修订分开记录，不把旧镜像写成新源码构建。

已完成可读验收入口和本机安全测试，并实际执行首轮托管匿名详情：元数据验收失败；初版清理断言也误读了 legacy 的 setter 顺序，修正断言的独立托管结果尚待补齐。不能由安全检查或成功外壳宣布起点已修好。目标仍包括真实认证、原 JAR / 历史远程 / 默认 Camoufox 的同条件三方、关键 Vue 3 业务和生产用户可见验收。

## 实际验收内容

- 输入限制为同仓库已成功的 `Native release artifact rehearsal`、精确 40 位镜像修订和原生架构。使用现有共享 JAR / 归档 SHA / Docker config / UID / renderer 的守卫，实际加载后再次核对镜像内 JAR。
- Reader 及浏览器仍在同一镜像内，UID 10001；新生成存储和账号，只有容器内部回环监听，没有发布 HTTP 端口。容器允许访问公开站点，但产品的私网拒绝保持开启，未设公网 DNS 替代或修改宿主代理。
- 2 CPU / 2 GiB / 256 PID / 零 swap，权限收紧；采集真实 cgroup 计数。预算只约束 Reader 容器，不把 runner 的镜像下载、Docker daemon 或硬盘用量算成该预算。归档在 GitHub runner 消费，不下载多 GiB 到用户电脑。
- 仅一次 `/getBookInfo`，使用已观测的新详情选择器；不调用搜索、目录、正文、真实账号或 Cookie 导入接口。要求 HTTP、严格布尔 `isSuccess`、`errorMsg` 和期望书名/作者/公开封面一起匹配；成功外壳但空元数据明确失败。
- 只保留字段存在性/匹配布尔值、错误长度/固定原因/域名指纹、请求耗时和进程类别计数；不保存原错误、跳转 URL、HTML、元数据值、正文、匿名站点 Cookie 或 Reader 日志。Reader API 跳转不跟随，响应限 64 KiB。
- 清除生成空间 Cookie，按 legacy 的退出 `isSuccess=true / NEED_LOGIN / 空 errorMsg` 和随后受保护接口 `isSuccess=false / NEED_LOGIN` 两个不同契约核对 Cookie 会话失效。不是全设备令牌撤销。finally 移除本次新容器及其进程；只上传小报告，存储目录和镜像不上传为该探针输出。

## 可重复命令

在已提交的候选分支上，复用已核实的镜像修订而不是探针提交 SHA：

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 -f native_arch=amd64 -f public_metadata_native_run=37339021759 -f public_metadata_revision=a82f08945a50c1520170fb6e8b3fb3c7a0b296fb
```

该模式与原件检查、旧远程、离线长测及通常镜像集成互斥，单独的并发组不会取消其他验收。不能填真实账号、Cookie、任意 URL 或正文参数；未来输入制品过期时须选新的已验收演练并重新核对其准确修订，不能静默换镜像。

## 本机检查、已知限制和回滚

### 首轮真实托管失败已保留

探针提交 `12fc067c26498d3d981f209482c75096decd3059` 的[专项 37418653036](https://github.com/warpdotsys/reader-dev/actions/runs/37418653036)在 GitHub 托管 AMD64 runner 上实际完成镜像加载与一次默认引擎请求；使用旧已验收演练 `37339021759` 的原镜像修订 `a82f08945a50c1520170fb6e8b3fb3c7a0b296fb`，不是新探针提交的产品构建。镜像 ID `sha256:229a96a1...`、共享/运行 JAR `0491a832...`、UID 10001 和两次身份守卫匹配；观测到 worker 1 / driver 1 / browser 类进程最高 7。

真实请求 5.521 秒，HTTP 200、严格布尔 `isSuccess=true`、空 `errorMsg`、data 为对象，但期望书名/作者不匹配且无封面，明确失败。产品私网拒绝未放宽，没有专用公网 DNS 文件，也没有真实会话或正文请求。内存峰值 826,904,576 B（约 789 MiB）、PID 峰值 201，2 CPU / 2 GiB / 256 PID / 零 swap；原始触限/OOM/PID 最大计数均为 0。

初版探针把退出和退出后受保护接口都误写为 `isSuccess=false / NEED_LOGIN`。核对可读 `ReturnData` 后确认：退出的 `setErrorMsg(...).setData(NEED_LOGIN)` 会由后一个 setter 恢复 `true / 空 errorMsg`，受保护接口相反顺序才是 `false`。只纠正验收夹具的两个明确契约，不修改产品返回、不放宽元数据断言；并增加两次脱敏鉴权观测以独立复验。首轮 `cleanupCategory` 不能当作产品退出未执行的证据，纠正后仍须实际托管验证，不能仅凭源码推断清理通过。

Cookie 列表前后为 0，最后新容器实际移除。下载的是 3,014 B 的脱敏制品，八份 JSON 已逐项核对并复算解压文件散列；远程 ZIP digest 只记录 GitHub 声明，没有冒充本机重算原 ZIP。[首轮完整原结果、身份、预算和夹具限制](evidence/public-metadata-hosted-first-2026-10-06.json)保留。匿名空元数据仍未唯一归因；不能借夹具修正把此次真实解析失败改写成绿灯。

新增 19 项无网络安全/生命周期检查实际通过，覆盖成功外壳拒绝、严布尔/错误契约、非公开封面、原始字段不回显、64 KiB 上限、拒绝跳转和正文接口、UID/私网设置、正常/失败/传输异常后的生成会话清理。它们使用生成数据，不是 Camoufox 真实运行证明。现有正式发布安全静态检查及 24 项测试通过；本机全量 Python 126 项，其中 125 实际执行 / 1 环境跳过、0 失败/错误，不能拿跳过证明 Linux 隔离。Shell 语法和工作流 YAML/九个输入解析通过；真正托管结果另行补记。

一次公开无凭据详情不能验证登录接受、章节权限、所有动态资源、长期并发、原 JAR/旧远程等价、Vue 3 真实用户或生产。失败需保留脱敏结果，不能补空实现、放宽公网防护、绕过验证码或宣称唯一根因。

回滚仅撤销 `browser-image.yml` 的新可选模式及两个探针脚本、配套测试，不涉及产品类、镜像内容或数据迁移。正常集成/离线长测入口保留；不要 reset 用户工作区，也不覆盖原 JAR/现有三个工程。
