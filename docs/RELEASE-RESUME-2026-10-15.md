# v6.1.0 发布失败边界与同制品续发

## 已证实的失败，不是应用测试失败

[原始正式标签流程](https://github.com/warpdotsys/reader-dev/actions/runs/38667600061)的九项前置作业全部成功：输入检查、两项 Vue 3、六项 Native。96 个生成大书场景及 32 份发布器报告已独立消费；源码为不可变标签提交 cc4593a1f707b30366005974f45eb1436da9b8fb，JAR 为 b4954ac7…，完整字节与已接受候选相同。

失败在正式发布器向 Docker Hub 推送：登录成功，但目标 medwarp/reader-pro-restored 返回 `denied: requested access to the resource is denied`。GHCR 的 AMD64 单架构标签已写入，不能据此宣称双架构正式发布；后续 GitHub Release 和生产作业跳过，旧生产服务仍健康。原标签及失败记录保留，不移动标签、覆盖原件或手工部署掩盖失败。

Docker 的[官方认证说明](https://docs.docker.com/reference/api/registry/auth/)区分身份认证与目标仓库操作授权。后续[只读托管核查](https://github.com/warpdotsys/reader-dev/actions/runs/38672500909)不执行 registry 写入，凭据和 Token 不导出：认证服务器对配置命名空间返回无 pull/push，对与 GitHub 仓库所有者同名的账号空间返回 pull/push。该声明不是实际推送成功；仍需正式发布器实测。

## 配置修正

2026-10-15 02:53:54 UTC，仓库变量 DOCKERHUB_IMAGE 从 medwarp/reader-pro-restored 改为 warpdotsys/reader-pro-restored，已 API 读回。阅读域名 read.medwarp.cn、许可证域名 license.medwarp.cn、生产数据目录及旧本地镜像标签不变。没有读取或更改 Secret 值，没有给其他账号新增权限。

正常 release.yml 新增昂贵构建前的只读推送授权检查。旧 v6.1.0 标签仍包含原工作流，不能移动它来偷换历史；因此新增可维护的 release-resume.yml，复用原流程的发布、备份、部署和回滚命令。

## 续发约束

- 只接受同仓库、原稳定标签 push 的已结束失败流程；精确提交必须等于标签且是 legacy 的祖先。当前已有正式 Release、版本不高于最新稳定版、前置作业失败 / 跳过 / 自托管、生产已经启动，都会拒绝。
- 必须有九个成功的原前置作业及 13 个未过期、绑定原提交与 run ID 的制品角色，包括四份原生 / 重载压力报告。下载原制品，校验其摘要、原归档、包内 JAR、TLS、预算和身份后导入；不重建应用、不反复重跑阅读测试取绿。
- 发布和部署都检出原标签；新的基础设施提交不伪装成应用构建提交。原始 run ID、续发 run ID、标签提交分别记录。
- 续发作业由 scripts/build-release-resume-workflow.py 从规范 release.yml 确定性生成，单元测试检查完整一致，防止复制后回滚逻辑漂移。另加同标签已有回滚快照就拒绝的保护，禁止重跑覆盖首次备份。
- 仍要求两仓库的同一双架构清单和匿名可读验证、正式 GitHub Release、不可变镜像部署、快照 / 救援与公开版本检查。没有删减原发布门禁。

新输入 / 规范派生共 16 项单元测试、凭据只读检查 8 项已通过；128 项原发布结构回归通过。真实 GitHub 输入预检已接受本次原 run 的 13 个制品角色。上述不等于续发、生产登录或回滚验收已完成；结果须另行记录。
