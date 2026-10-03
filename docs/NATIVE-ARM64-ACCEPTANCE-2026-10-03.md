# 完整 Reader 镜像的原生 ARM64 验收

更新日期：2026-10-03。本文记录候选验收入口；未取得对应作业及制品证据前，不代表 ARM64 产品已经交付。

## 验收入口

`browser-image.yml` 新增手动 `native_arch=amd64|arm64` 选项，默认仍为 amd64。ARM64 使用 GitHub 托管 `ubuntu-24.04-arm`，依据为 [GitHub 官方 runner 参考](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)。不注册自托管 runner，不安装 QEMU，不用 x64 上的镜像模拟结果替代原生浏览器运行。

```powershell
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 -f native_arch=arm64
```

作业沿用唯一完整镜像的同一 Dockerfile、OCI index 摘要、CPython 3.10 wheel 哈希锁、ARM64 Camoufox ZIP 大小/SHA-256、JDK 11、Vue 3 构建和 renderer 测试。不会另建功能缩水的 ARM 镜像。`uname -m`、`RUNNER_ARCH` 和生成镜像的 `Architecture` 必须匹配请求架构，错误标签或模拟环境直接失败。

ARM runner 额外安装 Chromium/Firefox 的主机端测试依赖，仅用于 renderer 回归；产品镜像依赖仍来自锁定的 OCI 基础镜像，与这一步主机安装无关。运行时不依赖 runner 的 Chrome 或 Firefox。

## 必须核验的证据

- JAR 构建、Vue 3 资源、32 项 Python CLI/协议测试与 Java/Kotlin 测试通过。
- 真实 Camoufox 合约 JUnit XML：GET/POST、脚本、Cookie、用户隔离、资源捕获、超时及恢复；核实跳过、失败、错误数，不只读取 workflow 状态。
- 同一 ARM64 完整镜像启动 Reader；镜像内 Python 与固定浏览器能实际渲染，中文字体存在，服务仅监听回环。
- 合成书源脚本书名改写、目标 POST 字段、Cookie 回放/删除和四用户请求全部成功。
- 下载 `bundled-browser-synthetic-and-resources`：`architecture` 实际为 `aarch64`，2 GiB / 256 PIDs / 2 CPU 限额存在，OOM/PIDs 限额事件为 0。PIDs 包含线程；短样本不等于长期吞吐。
- 生成镜像大小与架构、JAR 摘要和 Git revision 来自实际日志；这些候选产物不等于已推送的正式双架构 manifest。

## 边界和当前状态

截至本入口写入时，尚未执行新增 ARM64 作业。现有 amd64 托管验收已经通过，但不能证明 ARM64。正式多架构发布、生产出口策略、长期资源预算、真实登录书源与升级回滚继续单独验收；不因为有 ARM64 选项就创建 Release 或切换生产。

资源报告新增原生架构字段，6 项无浏览器单测验证正常 ARM 报告、缺少可选 PIDs 峰值不编造数据、OOM、PIDs 限额事件、未限制内存和错误 CPU 配额的拒绝行为；全套 Python 32 项本机通过。测试用临时 cgroup 文本，不是 ARM64 浏览器执行证据。
