# 准确原件与恢复版：公开页面快照三方对照

## 结论与证据边界

已实际完成准确原始 JAR、恢复 JAR＋固定历史 WebView、恢复 JAR＋内置 Camoufox 的同输入业务对照。四个业务调用各执行一次：详情、目录、第0章、第1章。HTTP 状态、ReturnData 包络、完整目录和两章完整返回一致；详情除三个默认时间字段外全部 JSON 字段和类型一致，三个时间字段均在各侧自己的实际请求窗口内。

这是同一份真实公开 HTML 的**无外网 loopback HTTP 快照重放**，不是三侧实站 HTTPS、真实账号登录或本机最新整镜像验收。正文只在执行进程内比较，导出计数和 SHA，不导出原始正文；独立验收器核查执行源码身份、报告契约和摘要，不能从导出的报告重新比较已销毁的正文。

未改 Java/Kotlin 产品代码、浏览器超时、安全策略、用户数据、生产服务或部署结构。测试中的历史 WebView 只是对照，部署方向仍是一个自带浏览器的 Reader 镜像。

## 已从 JAR 和实际执行验证的输入

- 原件 `reader-pro-3.2.14.jar`：72,913,887B，SHA-256 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`，原文件和只读备份未改变，不上传 GitHub。
- 恢复 JAR：285,671,807B，SHA-256 `f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b`。下载来源仍是 Native38115254938／7739；1ddf1210307f9e963c5ee0358ec936b210ef3d6c 自己的双 clean 重建再次得到同一字节，不能追改下载来源。
- 打包 worker：`BOOT-INF/classes/camoufox/worker.py`，SHA-256 `d8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5`。
- 历史 WebView manifest `sha256:88c250043bd33715ca372b749c1dca055c14e4550882caf968bd7af933d53573`，沿用锁定源码、Node 和浏览器，只加已有 PID1 收尾器；不修改其接口实现或引擎字节。
- 本机仍是旧缓存 runtime `sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44`＋准确 f5 JAR 组合，不把它说成最新整体镜像。

页面来自 [固定公开页面](https://www.gutenberg.org/cache/epub/84/pg84-images.html)，只抓取此 URL 一次，不访问站点搜索、使用账号、购买章节或操作验证码。468,370B，SHA-256 `23403a6a3b1cace42a06d6c378a6f1acf8ec97c940c652d12fe3c0d30275e58f`；HTTPS 正常验证证书，不跟随重定向、不传 Cookie。[有限抓取记录](evidence/gutenberg-snapshot-capture-2026-10-11.json)。实际 HTML 仅留在 ignored build 中，不上传 GitHub。

三侧使用同一个随机 loopback 地址和原有固定书源规则；只替换网址别名，保留 `webView` 选项、相对章节 URL、原 Jsoup/Rhino 业务提取。没有修改 HTML、填入模拟正文或伪造业务返回。页面的非固定路径返回空 404，外站资源不可访问；因此不证明原站图片/外部资源行为。

## 官方 uidmap 与隔离条件

根据用户授权，仅安装 Ubuntu 官方 `uidmap`。本轮只读确认版本 `1:4.17.4-2ubuntu3`，包来源 `archive.ubuntu.com/ubuntu resolute/main`；`newuidmap`、`newgidmap` 可用。

`/etc/subuid` 和 `/etc/subgid` 各自 SHA-256 都保持 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`；不改映射范围、系统代理或已有浏览器会话。两个隔离容器 UID/GID10001、无附加组、零 capabilities、no-new-privileges、仅 lo、CPU0-1，由外部宿主检查再次确认。共用 user/net，mount/pid/ipc/uts 分离；输入和快照只读挂载。

Java 堆维持128/768MiB，浏览器请求18秒不变；API客户端35秒。仅在宿主已验证的无外网 namespace 中允许 fixture loopback，不能推广为产品运行时允许私网。三侧注册/登录/退出均使用临时生成账号，不能算真实账号的 accessToken/Cookie 全生命周期验证。

## 首轮失败保留，不计通过

A轮在业务观察报告产出前退出1，耗时3.963秒，峰561,639,424B／PID23，触限/OOM/swap0。两个自有容器已独立观察 stopped/PID0 后删除；[失败记录](evidence/gutenberg-snapshot-first-bootstrap-failure-2026-10-11.json) SHA `28a64ae6005584837c7f47059956db8bb5b2b0ba0f4dc8e8294dfdc6e50ece28`。

准确执行源码在输入散列处使用 `hashlib.file_digest`。缓存 runtime 的 python3 指向3.10，B轮实测3.10.12且没有此属性，因此改为标准分块 SHA；这是有准确源码和环境证据的测试工具适配。A轮未导出私有 traceback，不能把这一推断说成已捕获的唯一异常根因，也不能把工具失败归结为 Reader 业务缺陷。A轮原记录及执行源码摘要不改写。

首轮另暴露现有测试脚本清单计数需要纳入新比较器：最初693项执行时清单29/30断言失败，保留为本机工具集成失败；新增脚本仍须验证 kernel namespace，并检查在读取输入或启动 JAR 之前拒绝普通宿主。修正后完整694项/30.692秒通过，仅1项既有平台跳过，不降低原件安全门槛。

## 第二轮：实际通过及独立复核

B轮53.318秒完成三侧业务。每侧生成注册＋登录、保存＋回读同一书源、四项业务调用和退出；每项业务都HTTP200／isSuccess=true／errorMsg为空，完整 ReturnData 字段和 JSON 类型比较在执行内完成，没有重试业务调用。

28条目录的标题、顺序、索引、默认值、原相对URL和WebView选项全部保留。首章6822字符、第二章7332字符，替换字符均0；三侧完整正文返回摘要分别一致。两章正文 SHA 为 `ea86a20b840b56b45f9870836bbf8f02ac0a9051f7a2819a6291fe32640a5366`、`e055c84c6b94026a5e8bbe0064b03e6edaa4451fdc9441e159ea5f1b79d38e6d`，也与先前独立固定实站结果相符；不是重新访问实站。

详情20个字段，除 `durChapterTime`、`lastCheckTime`、`latestChapterTime` 外完整 JSON 相等；三个值均是整数并在各自请求开始/完成之间。**原始详情完整逐字相等=false**，不删除或改写原时间来伪称完全一致。目录及两章完整 ReturnData 相等=true。

fixture 观察21次固定页面请求、7次其他路径、Cookie出现0；这不是12次业务入口调用的同义计数，也不等于原站资源请求总数。

总预算2CPU／2GiB／256PID／memory.high1.5GiB／配置及观察swap0，峰1,611,476,992B／PID241。soft high3201次，hard/OOM/PID触限0；不能称毫无内存压力或据此保证生产容量。停止历史服务并独立核实PID0后，仅回收该自有空叶子的缓存：476,336,128B降至5,496,832B，再允许Cam启动；不清全局缓存、不改产品预算、不删除磁盘数据、不证明早期超时根因。收尾观察两个自有容器停止且删除，28个历史浏览器的被收养僵尸由已有精确PID1收尾器处理。

[独立接受记录](evidence/gutenberg-snapshot-second-independent-acceptance-2026-10-11.json) SHA `2c233b5ff830a8c818c2ae99e2022759e87090b858b80bc031a225c9c62567c0`；[原始有限观察](evidence/gutenberg-snapshot-second-raw-observation-2026-10-11.json) SHA `1d024bfc5758e87355f381140e0eb2dd16db5224a7dc1e775c77a47d9d6c2ac3`。执行比较器SHA `ed224408d8856a79a49eea0447d852da429eac7ae148666d87ee5694ab595ef3`；原件、备份、恢复下载及用户58份脏报告均重新散列未变化。

两轮共四份临时输入JAR副本717,171,388B已精确清理；来源、只读备份、下载和所有观察记录保留，副本可重建。清理不递归删除目录或用户数据。[A轮清理](evidence/gutenberg-snapshot-first-copy-cleanup-2026-10-11.json)、[B轮清理](evidence/gutenberg-snapshot-second-copy-cleanup-2026-10-11.json)。

## 可维护复测入口

`scripts/capture-gutenberg-public-page.py` 只抓固定页，拒绝既有目标目录、重定向、非UTF8或不匹配页面；网络正文只写 ignored build。`scripts/compare-gutenberg-snapshot.py` 是可读业务对照，不依赖反编译伪代码或字节码补丁，拒绝错误输入 SHA、普通宿主、空正文、缺章、跨书URL、丢失选项、陈旧时钟和完整JSON差异。34项纯生成回归覆盖字段/类型/时钟/正文/安全拒绝和有限报告验收；不是原件运行的替代。最终本机完整700项/33.026秒/1项既有平台跳过通过。

快照工具是1ddf之后的本机维护源码，不在1ddf提交里；按实际执行时文件SHA鉴别，不把匹配的JAR源码身份当成探针源码身份。B轮执行比较器未在补充报告回归时修改，执行摘要仍为上述ed224408。

该比较器只能由受验证的无外网UID10001容器及宿主交接门启动，不能直接在Windows/普通WSL或生产运行。入口示例（容器内，准确副本和已核验快照只读挂载；每次使用新的输出与phase目录）：

```sh
python3 scripts/compare-gutenberg-snapshot.py \
  --java /opt/java/openjdk/bin/java \
  --original /verification-inputs/original.jar \
  --restored /verification-inputs/restored.jar \
  --snapshot /verification-snapshot/public-page.html \
  --report /storage/three-way.json --phase-handoff-dir /storage/phases
```

宿主还必须验证现有历史服务的确切身份与aggregate cgroup、在历史侧完成后停止其自有容器并观察PID0，再授权Cam交接；不建议复制环境变量绕过guard。本次宿主包装和执行输入保留在 `build/local-gutenberg-snapshot-three-way-f5c9-20261011-b`，原始页面在 `build/captured-gutenberg-public-page-20261011-a`，都不提交原件或正文。

## 尚未验证、已知问题与回退

仍需原件完整HTTPS/WS/WSS、真实来源的登录Cookie模型、更多下载/SSE差分、最新完整镜像本机/长测及生产UI和容量验证。起点验证码/搜索失败、真实EPUB合并预算的压力失败和未实现的OPDS服务保持未闭合；本轮有限成功不抵消它们。

本轮不合并、打tag、发布、推送registry或部署，goal保持active。回退只撤回新的快照比较器和对应测试；保留准确原件和首轮失败，不能用关闭证书/隔离检查、提高预算或伪造空实现取得通过。1ddf自己的普通托管验收与本轮后续工具源码的CI身份分开记录，不能借旧绿灯接受新提交。
