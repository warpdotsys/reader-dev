# 6.1.0 当前候选：托管产物接受，真实大书尚未接受

源码`c2970fbfc0eba698fa332a344233d1bbdc689477`，测试合并快照`e06ea3b107c26579e41c033b6e1bbb3939e0bbcf`。GitHub比较ahead=1、behind=0、文件差异为空。默认分支legacy；PR56仍OPEN草稿，未合并、创建标签、正式发布或部署。

## 已成功重建和直接复核

| 本候选自己的托管流程 | 实际核验 |
| --- | --- |
| [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38518410049) | 47份XML，222项／35既有环境跳过／零失败；两次独立clean的完整JAR字节、条目顺序、ZIP元数据相同 |
| [Vue3](https://github.com/warpdotsys/reader-dev/actions/runs/38518410173) | 380前端测试、类型／Vite；25实际Chromium零跳过，14登录／注册布局无水平溢出 |
| [完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38518410092) | 779 Python实际执行、零跳过；生成账号登录／导入／阅读／刷新／退出11.489秒；登录和阅读两图实际查看，中文可读 |
| [双架构和发布器](https://github.com/warpdotsys/reader-dev/actions/runs/38518410035) | 六作业全部完成；AMD64/ARM64原生／新runner重载／发布器32JSON分别核验；24 Camoufox实际执行、零跳过 |

完整镜像UI客户端在服务端预算外；服务端峰892,219,392B／PID200。发布器有限资源子集最大891,838,464B／PID202。不合并声称整个旅程总2GiB，也不当成真实账号或大书验收。缓存导出实测66.0秒，严格60秒总上限和因果加速不作宣称。

## 已从准确制品验证

D盘实下载JAR：285,683,587B／1568条目，SHA-256 `012b12d407faa230990ac165af320f6c166141db81a81c0ffd6e6a7c3c89fb2d`。外制品／JAR重核，MANIFEST为6.1.0／ReaderApplicationKt；非Vue3全部条目与f6的1f114993基线逐字节相同。新Worker确有transferring，SHA `bdcf7d8fe3281ab59077774064c8d63c9e9b3740ce75c474a7f54d8e21793870`。

| 镜像 | 原配置ID | 原内层归档SHA |
| --- | --- | --- |
| AMD64 | `sha256:0298b97d5ca14af28ed2e43a8a46eb9ee56ca51169bd46dd3feb2ffca4789fb8` | `8c5e875ce27a1e6319004f210d38d135f27d95f258252cf2d9242a66b90c5b15` |
| ARM64 | `sha256:5f958f121cf2f666a0d9ee6c84855aab26dd44865d554218c09d3d9c39c4cccd` | `fb562fc3836b4c479a4e968efd30399f2ddc04a73f6047dcd85d7ec5a62cfebc` |

此托管汇总时点的镜像身份来自小报告，不代替实物接入。后续准确AMD64实下载／每层／包内JAR已接受，原配置0298映射到Docker29的ad24；三轮同书默认／单变量／反向默认已[独立验证和清理](C297-REAL-EPUB-CONTROLS-2026-10-14.md)。最后原默认配置8/8通过，首轮6/8拒绝保留，不能称冷态稳定或唯一优化因果。原[托管时点汇总](evidence/candidate-c2970fbf-hosted-acceptance-2026-10-14.json)保持原值，后续以[系列证据](evidence/candidate-c2970fbf-book-series-2026-10-14.json)补充，不回写历史成功范围。

## 已知问题与下一步

f6/c76 [A轮](AUTHORIZED-EPUB-REJECTION-2026-10-14.md)6/8，原排版上一章刷新下载60秒超时；[B轮](EPUB-TRANSFER-PHASE-CANDIDATE-2026-10-14.md)4/8，完整下载约0.266秒后旧Worker阶段15秒超时。两种故障不同，均保留，不宣称修复。c297只拆分阶段，不重置计时器、提高资源或放宽内容／登录断言；当前准确新镜像同书八场景尚缺，阶段文字也不能单独证明唯一原因。

f6本机8/8有明确文件系统适配，678a半小时长测及f6真实公开书三方均保留原身份，不移借。起点认证只限本机隔离当前Chrome会话，尚未成功；旧Cookie已清，不复用、不上传、不购买或自动验证码。原件共享default、历史UTF8 POST损坏、空Cookie和忽略证书均按差异记录，不重新引入缺陷伪造一致。

完成准确归档接入后仅测试获准单书；总CPU2／硬2GiB／high1.5GiB／swap0，保留60／15秒期限。结束后清除该轮书副本和生成数据，保留无正文结果及清理回执。协议回滚须在独立干净检出同时恢复Worker／主线程／对应测试，再托管构建；不做后端／存储迁移。原JAR、Vue2回退、历史镜像和D盘恢复路径保留，不覆盖用户reports／storage／旧工程。
