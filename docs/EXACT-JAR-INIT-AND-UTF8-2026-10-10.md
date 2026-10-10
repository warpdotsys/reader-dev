# 准确原件三方增量：隔离 init 回收、UTF-8 差异及生成详情

## 当前结论和证据等级

2026-10-10 后续 d/e 两轮实际完成原件＋固定历史 WebView、当前 JAR＋同参考、当前 JAR＋内置 Camoufox的五例生成比较，完整十五次搜索响应一致，且父级严格资源门禁通过。只在诊断容器 PID1 回收已被收养的僵尸子进程，没有提高资源上限、减少用例、修改历史服务或产品字节码。[a/b/c 原失败](EXACT-JAR-GENERATED-THREE-WAY-2026-10-10.md)保持原结论，不倒填为成功。

f 实际取得十八次搜索观测，但历史 UTF-8 POST 仍被截断，**严格六例验收失败**，主探针退出1、外层整轮失败。g 实际十五次搜索及三次生成详情通过限定契约：非时钟完整 JSON 相同，三个默认时间字段是整数并位于各自真实请求窗口；不声称原始详情 JSON 逐字相等。

- **已从 JAR 验证**：原件 SHA `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`／72,913,887 B；当前 JAR `e59a01bf190de757f73216557ff353411019507d25bad76f0c808b48366adb84`／285,665,012 B。两者均实际执行，原件来自授权路径的只读副本，原文件未覆盖。
- **已实际运行**：固定历史 AMD64 manifest `sha256:88c250043bd33715ca372b749c1dca055c14e4550882caf968bd7af933d53573`、config `sha256:5d85fb04eb6a8900d56956f5598b50487209b72c736441bc954221ae8ac4e6bb`；历史 `index.js` 为5,391 B、SHA `963b7365504cc19a0345aa9cd83b3ea52583da11cbc5ae44d9b64d48512af030`，未改源码、浏览器字节或添加关闭 sandbox 的参数。这不证明它就是用户历史生产实例，也不单独证明默认 sandbox 的全部行为。
- **已成功重建**：下述 11df 自己的托管 runner 实际两次干净构建仍得到 e59；不是本轮再次本机编译。诊断用 e59 worker 和先前完整散列的只读8d依赖组合，**不是最新完整 Native 镜像的本机验收**；Java runtime 11.0.32.1+1，诊断堆仍 `-Xms128m -Xmx768m`，不冒称镜像默认堆。
- **源码观察／推断边界**：历史服务按 JS `body.length` 填 Content-Length；finally 中调用 `browser.close()` 未等待。前者与实际截断相符，后者不独自证明所有资源高峰的原因。隔离 init 的回收已真实观察，但没有证明长期泄漏全部解决。
- **尚未验证**：真实认证书源、完整请求头及 Content-Type 差分、真实站点是否接受此 POST、最新整镜像本机／长测、生产 UI／部署和正式发布。所有书名、网页、账号、正文均为生成夹具，未重导已清除的起点凭据、复制私人浏览器会话或读取其他本地正文。

## 隔离不变，修正诊断进程生命周期

保持父级2 CPU（0–1）／2 GiB／memory.high=1.5 GiB／256 PID／零 swap。实际 UID/GID10001、宿主映射110000、附加组空、能力零、禁止提权；仅 loopback，无外网，user/net 共享而 mount/PID/IPC/UTS 独立。JAR、依赖、执行脚本均只读，生成工作数据使用 tmpfs。宿主独立验证身份、namespace、挂载、原件 SHA 和父级资源后放行。

d 的诊断 init 原型每100ms枚举自身 private `/proc`，只回收 `PPid=1 && State=Z` 的已收养子进程，并排除直属 Node Popen PID；不发送信号给活进程、不抢占 Node 的 wait 状态。e/f/g 改为只读挂载的可维护 `scripts/guarded_pid1_reaper.py`，执行 SHA `33dc3f3282a9c4ad3a0d6fec4bb5d9d3ca70b560845b72e5694549c6ebe43ea8`。模块要求 Linux PID1、UID/GID10001及空组；调用方仍须独立验证隔离和预算，不能把该模块单独当成宿主门禁。

每轮都先实际观察历史服务 stopped/pid0，再放行 UID10001 自己创建的交接标记；比较器还检查8050拒绝连接，之后才执行 Camoufox。结束时全部自有容器 stopped/pid0、删除、剩余0，原件及58份用户保护文件再次散列一致。Node 优雅退出码未采得，不能将精确清理冒充应用自然退出。未触碰日常 Reader、系统代理、已有映射或生产服务；官方 uidmap `1:4.17.4-2ubuntu3` 已就绪，subuid/subgid SHA 均仍 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`。

## 四轮实际结果

| 轮次 | 实际观测／判定范围 | 整体 | 内存峰值 B | PID峰值／触限 | high次数 | 回收僵尸数 | 秒 |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| d | 五例、十五次完整搜索比较；init原型 | 通过 | 1,611,530,240 | 244／0 | 5,400 | 20 | 42.825 |
| e | 同五例；正式 helper 实际复测 | 通过 | 1,611,894,784 | 241／0 | 4,664 | 20 | 39.237 |
| f | 六例十八次观测完整；旧 UTF-8 截断，严格判定拒绝 | **失败** | 1,611,558,912 | 243／0 | 4,466 | 24 | 40.091 |
| g | 五例十五次搜索＋三次延迟 DOM 详情；非时钟完整一致／默认时钟按窗口验收 | 限定契约通过 | 1,611,554,816 | 247／0 | 5,867 | 24 | 42.059 |

四轮父级 memory.max／OOM／oom_kill／PID触限／swap均0，资源门禁通过。memory.high 明显非0，峰值包含文件缓存，不能当纯 JVM RSS或无泄漏证明；PID最高247距上限256仅9，短时顺序用例不证明长时／并发安全。f 的红灯不是资源失败，也不因观测齐全改成兼容通过。

GET四例＋POST一例的完整 HTTP状态／ReturnData／JSON字段类型默认值及已记录的请求方法、正文、Xfixture头严格比较；历史两侧目标 Cookie 空，Camoufox准确重放 `session=alpha==` 再删除。此生成 Cookie 已审查改善明确保留，不要求重现历史缺陷；未验全部请求头。

## UTF-8：保留准确故障，不把相同搜索结果当正确请求

f 实际使用 `--exercise-encoding --characterize-historical-utf8`：它是已知缺陷观测模式，不是严格六例通过模式。生成 JSON 正文包含中文和扩展字符，不是私人书籍正文。

历史原件接入／当前历史接入各只发送44 B，完整 SHA `c61ca3e561f770d4fc8551da0bddb98afd0be801801502504a73221ed11ec5d9`；Camoufox发送正确60 B，SHA `8d0383070028f4f457194a194e8e9133df2487261e341c1da121171d59eeeef1`。旧正文在 `"note"` 之后截断、不是完整 JSON。生成服务故意可对截断请求返回同一本书，十八次搜索 JSON 一样**不能证明真实站点会接受损坏请求**。

观测模式仅识别准确已知字节差异，不归一化未知差异；仍明确 `strictSixCaseParityAccepted=false`、`historicalUtf8BodyCorrect=false`、整轮失败。没有修改旧参考去伪造严格等价。

## 生成详情：保存原始时钟并限定比较

g 独立执行 `--exercise-metadata --metadata-clock-contract`，不与 UTF-8 模式混用。三侧各真实调用一次 `/reader3/getBookInfo`，延迟 DOM 脚本各执行，HTTP200／isSuccess=true；除 `durChapterTime`、`lastCheckTime`、`latestChapterTime` 外，完整 ReturnData JSON 逐字段／类型严格相同。

| 侧 | 实际请求窗口 ms | 三个默认时间字段的实际值 |
| --- | --- | ---: |
| 原件＋历史 | 1791598752630–1791598754030 | 1791598752636 |
| 当前＋历史 | 1791598761604–1791598762887 | 1791598761612 |
| 当前＋Camoufox | 1791598779923–1791598782315 | 1791598779933 |

时钟必须为整数、落在各自窗口；不删除原始值、不声称三侧原始 JSON 完全相等。生成 SCRIPT SHA `045f640185166e2d9415e09b1e2d574ba45cf6119345ba180f4fe6e5a9bf4b04`，诊断 RULE SHA `603a86357f45c3b850ec3c8cce2fe98266053aa1276fdf7178fbd5a4fe61166a`；执行 metadata helper SHA `3a8659c5496f81523047d293cd0db2c321eb0a900b9dcc26eab08f53079a1735`。这些规则未替换真实书源。

## 可复验记录与本地单测

四轮执行比较器 SHA `afe45db938d0abc3be8cf8d18b461bad63288bcadfe6c752c95f0a6f45563daf`。独立 verifier 对四份完整报告 SHA、执行源码、隔离挂载／身份、预算、停止交接、清理、已回收PID、完整业务 JSON及限定 metadata 重新验收：**63份搜索响应＋3份详情**，同时保留 f 严格失败。有限回执 SHA `5c532f3b5e7ac93322bdb2f3ef5f1c462b08eb8649728d0cb5086e943d72145e`；[有限机器记录及原报告 SHA](evidence/exact-jar-init-and-utf8-2026-10-10.json)。完整原报告和执行副本留在本机被忽略的 build 目录，不上传原件／正文／凭据到 GitHub。

新增 init helper 十一项单测验证只回收已收养僵尸、排除直属子进程、拒绝非PID1／root／额外组、非法PID／symlink／超大及损坏status等。首次 Windows mock 六项错误源于 Windows 无 `WNOHANG`；修正的是模拟 Linux 的测试环境，不改生产 Linux常量、不添加跳过，原失败保留。最终 Windows443／1项既有平台跳过、WSL443／0跳过；模块在实际 Python3.8参考容器中也完成 e/f/g 运行。新 helper 和截图隔离增量尚需自己的托管结果，不能借下述11df绿灯。

## 11df 自己的四条托管结果，不代替新增量验收

源码 `11df6be72ec482e51c9847004c1469c1b26d0e41`，实际tested merge `6926f309278acc74a0d2902597d8efde43a2625e`。独立验收 API 来源、实际作业、报告SHA、XML／JSON／PNG结构及 runner 名称／labels，实际都是 GitHub托管 `ubuntu-24.04` 或 `ubuntu-24.04-arm`，不是自托管。

- [Java 38014695025](https://github.com/warpdotsys/reader-dev/actions/runs/38014695025)：432 Python／0跳过；181 JVM／31既有环境跳过，TTS编辑11全执行；两次clean的完整JAR字节及1,569条目一致。独立回执 SHA `e08689fa7b7cb7f6ecba396429c6cf27586043d4be2c25246c2f17e87a20fb77`。
- [Vue 38014695032](https://github.com/warpdotsys/reader-dev/actions/runs/38014695032)：25实际浏览器旅程／0跳过、296前端测试＋20截图守卫、11 PNG；独立回执 SHA `5e2cbb176ca1a76df4ba5dbd4cb0b09ea17a44d92542526dd2ffc8e8cd638cda`。
- [Full 38014695026](https://github.com/warpdotsys/reader-dev/actions/runs/38014695026)：实际 Camoufox20／Chromium23／helper3及默认UI／异步／metadata接受；峰值894,402,560 B／PID193。独立回执 SHA `8d509f9293f89471c1efb3277ff6888f85a8095c225717f26429649272701493`。
- [Native 38014695045](https://github.com/warpdotsys/reader-dev/actions/runs/38014695045)：共享JAR、双原生构建、双重新导入及publisher共六实际作业／24JSON接受；最高871,714,816 B／PID202。独立回执 SHA `3e2024b91973bb7f64fa902fe3181b31050a25f13db51fc21ce16aa27e9f0182`。

Full／Native普通短测允许1 GiB swap、实际0，不冒称零swap长测。只下载小回执并复核声明的11df Native来源；没有下载新完整大归档并在本机复算／加载。此前完整8d归档身份不能替代新的整镜像。新 init helper不包含在11df，11df的Python是432、不是443。

## UI观测、已知问题与回退

实际查看11df的TTS编辑、OPDS不可用两张截图，所见中文可读不是全界面乱码验收。OPDS配置适配器明确拒绝未实现操作，且请求计数0；截图里的网络错误来自前一段TTS断网测试，**不是OPDS发送网络请求的证据**。Java/Kotlin目前没有OPDS路由或独立账号存储，不能用普通登录假冒实现。

`Vue3PreviewSettingsDialogTest` 新增通过用户可见“关闭消息”按钮清理上一场景通知，然后等待退出并断言OPDS图不含残留网络错误；原TTS失败断言／截图先保存，OPDS不发替代请求及密码不落浏览器存储的断言继续保留。它是证据场景隔离修正，不是已实现OPDS或已修好生产登录；需新runner实跑及新截图确认。

仍需最新完整镜像本机／长测、真实认证书源及生产登录／阅读验收，不能发布成全部完成。回退可停用新诊断 helper和截图隔离增量，仅回收精确自有单元；四单元已独立 inactive／MainPID0。保留历史失败、原件、下载缓存和用户dirty文件，不 reset／覆盖三个既有工程，不更改代理、认证语义或生产部署。
