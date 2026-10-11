# 停止实例交接改动自己的托管验收

日期：2026-10-11。源提交 8e74b255d351448baec0b795541e19d5b18c7cfe；实际 PR 合并快照 ef7c72ab0a7a2883cfb52e01fc88d5eb53517e13。以下均为该提交自己的终态作业和下载回执，不借父提交结果接受后续修改；不代表已合并、发版、上传 registry 或部署生产。

## 已成功重建与实际执行

| GitHub 托管流程 | 实际结果 | 独立回执 |
| --- | --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38109308988) | 622 Linux Python 零跳过；201 JVM / 35 既有环境跳过；TTS 编辑 11、日志 9、代理 4 项全执行；两次干净构建字节一致 | [Java](evidence/stopped-cache-hosted-8e74b255-independent-java-acceptance.json) |
| [Vue 浏览器旅程](https://github.com/warpdotsys/reader-dev/actions/runs/38109308980) | 301 前端测试；25 实际生成 UI 场景零失败/跳过；11 张合成图回执，未据此声称本轮已逐图目视 | [UI](evidence/stopped-cache-hosted-8e74b255-independent-ui-java-acceptance-v2.json) |
| [完整单镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38109308985) | Camoufox 24 契约零失败/跳过；实际 Reader HTTPS、默认完整 UI、源辅助接口门槛接受；峰值 857,575,424 B / PID 194 | [Full](evidence/stopped-cache-hosted-8e74b255-independent-full-acceptance.json) |
| [原生双架构与导入](https://github.com/warpdotsys/reader-dev/actions/runs/38109308989) | 6 作业全部终态成功；AMD64/ARM64 原生构建、重导入实际启动及 publisher 的 32 份报告逐字段接受；最大峰值 896,196,608 B / PID 211 | [Native](evidence/stopped-cache-hosted-8e74b255-independent-native-acceptance.json) |

所有 runner 均为 GitHub 托管 ubuntu-24.04 / ubuntu-24.04-arm，没有改回自托管。没有下载新的大型完整镜像，仅下载本轮的小型测试/身份回执。

双干净 JAR：9b9d0c9808968c0390022156a5fcbfcc03f595ed228f124afbb06e830cd7dd84；285,672,062 B / 1570 条目；四类差异计数均为 0，完整 ZIP 注释和条目顺序也相同。它与 a5cd 下载副本字节相同，但该副本的下载来源仍保留 a5cd/2147，不改写为 8e74。

AMD64 实际镜像 ID：sha256:43ad7e2bc69f6f4bcf716f1677cef079c1ad5f5f403cec6281660ce237fd79b4。

ARM64 实际镜像 ID：sha256:6871dd1ab73691d7d9a4269af112643e84a2518c96390abb7e87e310eb037c9d。

回执 SHA-256：Java 7854301eb70ef9af217cadf651e880592ca006dfe8c90de9f345bc165d2bb9ad；UI 96a16dcef43d2e4175f3a53a5774d2a43ad300734bfcb3a8564f354dbbe98d57；Full 261347ef8d949d1be2a4e41d7f6fed631418a116290a49211f67e357c8ea5277；Native aed5c809a4f87450d407caf1da79a48b97241807901d837c448197b428507015。

## 已知问题、范围与回退

普通 Full 短测允许 1 GiB swap、实测为 0，不改称配置禁止 swap；外部 UI 客户端不属于服务端容器预算。原生 HTTPS 专项和本机隔离三方各自的零 swap 预算单独记录，不混用。

8e74 只修测试交接，生产 JAR 没变。此前同源 Authorization 实际丢失和首搜索超时仍保留；本次未复现不等于根因修复。准确原件的生成元数据同次三方结果及时间字段差异见[原件报告](GENERATED-METADATA-THREE-WAY-2026-10-11.md)。

原件完整 HTTPS、WS/WSS、真实站点认证、OPDS、最新整镜像本机入口、生产容量和正式部署仍未整体验收；真实授权 EPUB 另记，不把生成旅程冒充真实书。PR56 保持 draft。新 Python 3.10 哈希兼容修改属于后续源码，不能借此提交绿灯。

回退只撤回自有测试交接提交，保留原件、失败证据和发布守卫；不 reset 用户工作树，不改生产服务，也不采用字节码补丁或空实现。
