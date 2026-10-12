# 优化模式安全预检：保留失败并修正拒绝路径

更新日期：2026-10-12。只修改测试探针及回归，不修改 Reader Java/Kotlin、界面、JAR 或生产服务。

## 已核实的失败

源码 `4c4985c513a74e394f8556fbfb2aad6b6bda5925` 的 [完整镜像构建 38197688234](https://github.com/warpdotsys/reader-dev/actions/runs/38197688234) 失败，作业 `114643939153` 在 `Test and build the full JAR` 的 Python 测试阶段结束：731 项、22.000 秒、1 个错误。

错误为 `test_disabled_assertions_cannot_bypass_real_cli_guards` 的真实 `python3 -O` 子进程超过原 10 秒限制（`subprocess.TimeoutExpired`）。Runner 使用 Python 3.10.22。记录没有证明子进程具体停在哪个导入或检查位置，不能声称已确定唯一根因，也不能归因于 Reader 浏览器业务。

此时还没有构建完整 JAR/镜像或执行后续浏览器合约。后续上传不存在的报告是级联结果，不是已经复现的其他业务故障。[有限失败摘要](evidence/optimized-guard-full-ci-failure-2026-10-12.json) SHA-256 为 `abf121ffbde47187721a9dee86736ae3490d067c015f7f9ee1a97a404c97ecd2`；本机保留原始作业日志，其 SHA-256 为 `f8173eadeb1db91f14fdea5637c6551339427e99c9d16bdcb42ddc1f14963224`。原日志不提交至 GitHub，原失败不覆盖。

## 源码修正与验证范围

`scripts/probe-baseline-api-in-guarded-runtime.py` 在仅导入 `sys` 后立即拒绝优化模式，先于 SSL/HTTP/进程等辅助模块导入及参数、路径处理。`main()` 原有拒绝检查仍保留，所有原断言、安全隔离及资源限制不变。

新增一个导入顺序回归：在 `-O -S` 测试子进程内拦截重型模块导入，必须先得到 `Refuses disabled safety assertions`。`-S` 只用于这个新回归；原真实 CLI 回归仍使用普通 `-O`，原 10 秒限制不延长、不跳过。

修正后已有本机完整 732 项测试通过（1 项既有 Windows/POSIX 平台跳过），相关 14 项在 Windows 和 WSL Ubuntu 均通过。当前 WSL 的 `/usr/bin/python3` 活验为 3.14.4，不等同托管 3.10.22；这些结果不代替新提交自己的 GitHub 托管构建。后续托管结果需按新的源码和运行编号独立核实，不重跑旧失败来掩盖它，也不借用 6a 的旧绿灯。

## 安全边界与回退

本机官方 `uidmap` / `libsubid5` 已安装，版本 `1:4.17.4-2ubuntu3`，`newuidmap`、`newgidmap` 可用；`/etc/subuid`、`/etc/subgid` 仍各为原 SHA-256 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`。未更改映射范围、代理或生产服务。

原 JAR、只读备份、授权单书和 58 份用户报告哈希保护不变。没有读取其他本地正文，没有复用真实 Cookie，没有合并、发版或部署。新预检并不关闭真实认证、请求/Cookie 三方和单书合计压力等既有缺口。

如需回退，仅在独立干净工作树恢复此探针及新回归的前一提交；保留失败证据及原 10 秒门槛，不 reset 用户工作区，不绕过优化模式拒绝检查。
