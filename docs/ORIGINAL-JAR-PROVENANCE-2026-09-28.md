# 3.2.14 公共镜像 JAR 来源核验

核验日期：2026-09-28。此报告只比较文件身份，不把公共镜像当作用户提供的原始 JAR，也不评价镜像中的业务行为。

| 对象 | 大小 | SHA-256 | 结论 |
| --- | ---: | --- | --- |
| 本地只读原件 `reference/original/reader-pro-3.2.14.original.jar` | 72,913,887 字节 | `B26FB4769D689D98FF26408CE79A275D719F360906C84ACF52FF404E98030C8C` | 本项目黑盒基线 |
| 公共镜像站所存 `hectorqin/reader:3.2.14` 中 `/app/bin/reader.jar` | 72,914,622 字节 | `7222FBD0C55D8B7E6F6BC1BBF5F6637FBEE45DFBD65EDA0EF59E38B2E24CD43A` | **不是同一 JAR** |

第二项来自[GitHub 托管 runner 来源核验](https://github.com/warpdotsys/reader-dev/actions/runs/36418763589)：只从公开镜像站拉取 `swr.cn-north-4.myhuaweicloud.com/ddn-k8s/docker.io/hectorqin/reader:3.2.14`，创建但不启动容器，复制出 JAR 并计算大小及 SHA-256。该镜像拉取摘要为 `sha256:910043799b510b5796baf99537507a63ca95995cda7de95cbf2ba5eeb9dcb5d1`。作业按预期因哈希不符失败；JAR 没有上传为 Actions artifact，也未提交到仓库。原 JAR 的哈希和大小已再次从本地只读备份核对。

2026-09-28 从 [Docker Hub 官方标签接口](https://hub.docker.com/v2/repositories/hectorqin/reader/tags/3.2.14)查询 `hectorqin/reader:3.2.14` 返回 404；公共镜像站的归档存在不证明它与用户文件同版。相反，[旧远程 WebView 3.2.0 官方标签接口](https://hub.docker.com/v2/repositories/hectorqin/remote-webview/tags/3.2.0)仍返回镜像摘要 `sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`。这只能证明一个可追溯的旧实现镜像仍可获取，不能证明它就是以前生产部署的远程实例。

因此，不得把公共 Reader 镜像中的 JAR 替代本地原件来凑成“原 JAR／旧远程 WebView／内置 Camoufox”三方通过。后续若在隔离环境使用该旧远程镜像，还须单独标记它是参考实现，并记录部署参数及真实响应；原 JAR 仍以本地只读文件为唯一精确基线。
