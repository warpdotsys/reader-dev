# 准确原件三方请求头实测与 POST 默认类型修正

日期：2026-10-10。全部为本机隔离、生成数据；不修改生产、不读取私人正文、不重新导入起点凭据。官方 `uidmap 1:4.17.4-2ubuntu3` 已安装，来源为 Ubuntu `archive.ubuntu.com/ubuntu resolute/main`；`/etc/subuid`、`/etc/subgid` 散列均仍为 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`，没有扩大映射范围。

## 已从原始 JAR 与实际请求验证

原件 b26／72,913,887B、恢复 JAR da54／285,665,781B，后者来自源码 `2a3db5106d0d1e3a638a166f627cdfd1896852cb`／tested merge `1b18ea95106148111e662cfabf689f4fea750bb2` 的 Native `38028479215` 共享产物 `11661352148`。本机下载后完整 SHA 与自己的 Java／Full／Native 相同；下载进程限制2核／512MiB提交内存，实际峰值141,594,624B。只下载约273MiB的JAR，不重复下载大整镜像；GitHub声明的ZIP digest不冒称本机独立ZIP散列。

本机复用已验证的只读8d runtime（image ID `sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44`），覆盖为上述准确JAR副本。历史引擎固定 manifest `sha256:88c250043bd33715ca372b749c1dca055c14e4550882caf968bd7af933d53573`。这是已缓存runtime与新JAR组合的行为诊断，不等同最新e500完整镜像本机验收，也不证明生产原来使用的参考引擎版本。

两轮实际使用 UID/GID10001、空附加组和能力、禁止提权、私有命名空间、只有loopback、只读原件/依赖/脚本、全新tmpfs存储。父级总预算2CPU／2GiB、memory.high=1.5GiB／256PID／零swap，包含两侧及其Java、浏览器后代；先独立确认历史引擎stopped/PID0，再允许Camoufox运行。没有修改浏览器或历史引擎业务字节。

| 实际轮次 | 请求头与完整搜索响应 | 严格结果及资源 |
| --- | --- | --- |
| h：三次GET、脚本GET、普通POST，三侧 | 15份完整解析字段对；旧严格方法/正文/指定头/ReturnData通过 | 原有五项契约通过，不是所有请求头一致；46.603秒，峰值1,611,730,944B／PID243，high5536 |
| i：增加显式JSON UTF-8 POST，三侧 | 18份完整解析字段对；完整结果保存 | 非接受诊断完整，probe退出1／主驱动2；45.451秒，峰值1,611,571,200B／PID239；严格六项兼容仍失败 |

两轮max／OOM／PID触限／swap均0，独立确认各自两容器stopped/PID0、删除后自有清单0；PID1仅回收已被收养的僵尸20／24个，不干预Node直接子进程退出。high是实际提前回收次数，不是OOM或峰值改善唯一归因。PID接近256，不能从短测推断长期无泄漏。原件/只读备份未变，58保护文件变化0。

原始h报告SHA `fb291f88669aa31d0a5a04bd02c87754c54307f115d43873ac9662729fa56e0b`，i报告 `ebfa8d032a7a4b728e29da4ace23aec9d4aae26a58e3db9e8e380461d59b42fc`。独立重读原始报告、重新严格判定、检查输入/运行身份与累计预算后，产生[有限复核回执](evidence/target-header-differential-2026-10-10.json)，SHA `6579cc7bb98d5cede4c6e6db5aba4b6be0bfb1bb2e729fbe5d6ab8eff8a71a92`。执行比较器SHA `1728f2789023212cbe1c305fca2d2611f6ea0383320b08676f972a109bb0bcea`。记录的是所有**解析后的字段对**，保留大小写、顺序、重复项；不是原始HTTP线缆字节。诊断按不区分大小写字段名分组只用于说明，不降低严格门禁。

## 新发现：不能用“相同正文”替代请求头兼容

- 原JAR与恢复JAR连接相同历史引擎时，五／六次请求头字段对逐项完全一致。只证明这两轮，不泛化所有书源。
- User-Agent、Host在三侧相同；Camoufox仍有字段顺序、Accept-Language、Accept-Encoding、Connection大小写、Sec-Fetch-User、Priority及Cookie差异。普通POST的历史链路只有6字段，Camoufox有15字段，不能宣称字面完全兼容。原版／历史恢复的Cookie为空；Camoufox第二次回放生成session、随后删除，保留既有明确行为改善边界，不证明真实登录态。
- 普通POST正文均为6字节 `q=post`，但历史两侧 `Content-Type: application/x-www-form-urlencoded; charset=UTF-8`，Camoufox为 `application/octet-stream`。原有生成fixture并未要求正确表单类型，所以此前能返回相同书籍并不证明真实表单站点能解析。新增全字段观测实际发现了这个缺口。
- i中书源明确要求 `application/json; charset=utf-8`，历史两侧实际上仍发送表单类型和44字节正文；Camoufox保留显式JSON类型和完整60字节正文。旧44字节SHA `c61ca3e561f770d4fc8551da0bddb98afd0be801801502504a73221ed11ec5d9`，正确60字节 `8d0383070028f4f457194a194e8e9133df2487261e341c1da121171d59eeeef1`。不复制这个历史缺陷，不把成功书籍JSON当请求正确，也不把已知差异改判严格通过。

## 可读源码修正与验证边界

`camoufox/worker.py` 仅在首次匹配的POST导航、且书源没有显式Content-Type时补原版实测的表单默认值；保留已有浏览器普通头与书源头。显式类型按字段名大小写无关识别，包括空值，不覆盖JSON、自定义类型；不改全context默认头、不URL编码或重编码正文、不重放请求、不对GET/子资源再次注入。继续由浏览器管理Cookie，未把浏览器安全Cookie快照复制进请求头。

实现参考官方[Route覆盖行为](https://playwright.dev/python/docs/api/class-route#route-continue)与[Request headers边界](https://playwright.dev/python/docs/api/class-request#request-headers)。覆盖头可能沿HTTP重定向传播，仍须保留重定向/跨站及认证的实际门禁；此修正不证明所有此类路径。没有改原有请求隔离、代理或Cookie存储逻辑。

4项新生成协议用例覆盖缺省表单、大小写/空显式类型、GET、空正文，以及子资源/二次导航不重放、原输入/context头不被改写。修正前67项产生3个failure条目，修后全Python471项／26.297秒通过，Windows原有POSIX用例跳过1。测试替身不是浏览器实测。已有实际 `CamoufoxWebviewRendererTest.getPostScriptsAndSubresourcesUseTheBrowser` 加强为目标HTTP服务器检查GET无form头／POST准确form头，原显式JSON60字节用例保留；它须由新修正自己的托管完整镜像执行，不能借2a/da54的20项旧绿灯。

**尚未验证**：修正后的独立构建/完整镜像、首次修正后的准确原件三方、跨站重定向及真实认证、长期资源稳定、所有字符集/请求头、完整生产UI、OPDS与正式发布部署。h/i是修正**前**da54的完整证据，不把它写成修后成功；新源码的托管结果另外接受。

回退仅撤销此次默认POST头与新断言，不覆盖原件/旧报告或用户数据；回退会重新引入表单类型缺口。保留原JAR作为对照，历史UTF-8差异继续列为已知兼容风险。

## 修正自己的产物和第二轮三方已取得

以上“尚未验证”是修正提交时的边界，随后取得以下新证据，不能借修前h/i冒称通过。

源码 `bb6afa4e2295a1bfabc88ae45ef8e68b364f631d`／tested merge `9a86d3b70d70c0041f084569dc31891056d527d5` 已提交推送，仍为PR56 draft，默认legacy，未合并、发版或部署。

- [Java 38030983280](https://github.com/warpdotsys/reader-dev/actions/runs/38030983280) 独立接受：471 Python／12.26秒／零跳过，43 JVM套件／188例／31既有环境跳过，日志9例及编辑11例全执行。两次干净JAR均 `a873136863b8f9fba61dd4245325771e983b34e613f97b398f040b41fbb3f3cd`／285,666,064B／1,569条目，完整字节/顺序/ZIP元数据差异0；重建JSON SHA `186d43b88663a6f5156110dfd520069b7f9ec86c0b4dea99efbac9f24e47e3cc`，独立回执 `5b7472920415110a9c4df880570b915843308712f3309425de2b9ad76f088bcd`。
- [Vue 38030983284](https://github.com/warpdotsys/reader-dev/actions/runs/38030983284) 独立接受：25实际UI／零跳过，296前端＋20截图守卫、类型/构建和11图完整性通过，回执 `ddc178113e3ebfa64386395d184fec4def65ec67dd0488cc13e7eab618e44f0b`。
- [Full 38030983259](https://github.com/warpdotsys/reader-dev/actions/runs/38030983259) 独立接受：Cam20／Chrom23／helper3，新增普通POST目标表单类型断言与原显式JSON60字节断言均在真实浏览器执行，Cam XML SHA `acbf586c90df4e24c8e3b28835d32ac8f18ab159a627e08c8d167afaa442ec59`。默认界面生成旅程1例／8.346秒／零跳过，实际注册/错与正确登录/导入/中文两章翻章和刷新/退出，XML SHA `7f7e743bfb54db9c81179dd24bea7579e162d5d6a80a36ba7dc8db13102b44e3`。新书架图 `773cbc3aa7cf01d09fbcc3c0d5f6547591cc603c498ac911d7b9339b959714f4`、阅读图 `5b00a6ef8b0d9e0282c37930fcc82b3d7b45953a4eb3f04ea6fd15f26ba0779c` 主agent实际目视中文可读；登录和退出图与此前已目视图SHA相同。导入完成弹窗仍保留、阅读仍直接旧bookUrl路由，不声称所有卡片点击和全UI无乱码。累计服务端峰值881,217,536B／PID194，既有2CPU／2GiB／256PID、触限/OOM0、实际swap0；普通允许swap，外部客户端不计入服务端预算。回执 `6a78eddf70761468e0ed1d4b46c209640cdbbdc41c6f397a34e256da5281e143`。
- [Native 38030983318](https://github.com/warpdotsys/reader-dev/actions/runs/38030983318) 共享JAR作业 `114151693006` 成功后，只下载自己共享产物 `11662625330` 并核对上述a873完整SHA；下载峰值142,159,872B／仍2核512MiB。下载时整体Native仍在运行，没有把单个JAR作业成功当整体成功。之后六个实际托管作业全部terminal success，双架构原生构建/实际运行、各自归档重新导入后运行、同正式publisher导入器无仓库凭据演练，24份小JSON均独立接受；四份预算最大875,327,488B／PID211，触限/OOM0、实际swap0但普通允许swap。AMD64 image `sha256:c9c870da248447f9ba46ee9c3aea85d48ac8a9f903f6cc872b366d1bf52edca0`，归档 `55740aae9150a5ffb917a456463ddb032fa0ff62fa25bbdaf63e47e1244fac11`；ARM64 image `sha256:b42123c46fbad49131954297e5167f23e517b8a2e7e911e462e0fb10b4cf1bb2`，归档 `1ebca45a29b6de7cca43ff8cf5a11fafbb52ada0dd15b1254443913874d3bfda`。两侧同a873／version4.0.7／tested9a86，独立回执 `5805b60dc5a1f87f4a604ee83e3cc8f7d33992e28b8f44ee7ab98137efa8fb49`。没有在本机下载大归档或启动这两个新整镜像，也没有发布仓库；默认UI渲染新旅程只有Full AMD64，不冒称ARM全UI渲染。

旧da54与新a873两JAR逐个读取全部1,569条目，唯一内容差异是 `BOOT-INF/classes/camoufox/worker.py`，新资源SHA `fdfa1a21fbeb440861fe8ace062abb44022aa0679e57ac9e9ebfcabe8ab0e985` 与可读源码一致；Java类、依赖、前端逐条字节相同，ZIP元数据差异0。没有给JAR做热补丁，也没有把不同版本标为可重复构建相同。独立完整增量回执SHA `3a4e78da935d73425307b4489485e6333fd8968bf1817aead0d15ce2028ddf8d`。

原件与新a873组合重跑同一隔离门禁和脚本：j实际15次，旧五项JSON/方法/正文/指定头契约通过，**三侧目标普通POST类型都为form**；41.753秒，父级峰值1,611,649,024B／PID244／high5176。k实际18次，普通form与显式JSON均正确观察，历史44／Cam60字节仍完整保存，严格probe1／整体2／不兼容；47.836秒，峰值1,611,661,312B／PID241／high5078。两轮触限/OOM/swap0，自有容器独立stopped/PID0、清单0，收养僵尸20／24；没有提高限额或重置累计峰值。

j原始SHA `a8766b4dfbbaa37da8497341f85b5ead7f5084c8067d486488f67ee5ad879a5c`，k `fa00b27f5f93a12c54dcbbb9a22301589559787f26ed6fd6cccd0b5cb563a96e`。完整33份新字段对重新独立判定、输入/预算/原件/58保护文件核对后，产生[修后有限回执](evidence/form-post-restored-differential-2026-10-10.json)，SHA `5651d4f7977226a8e5d3c45493ddc7485b454a22b2eb64b8faae267ff127d21e`；独立复验源码SHA `a346de7227647e7ed3ccf92ce6b47f175662433c93a93cd1ff173397323cbf4c`。修前后总66份字段对；只补普通POST默认类型，不将其他字段顺序/大小写/语言/压缩/指纹/Cookie差异判为全头一致，也不修复或复制历史截断行为。

67项生成协议红绿回归又从准确2a Git源码和新源码独立重放：修前3失败条目，修后0，均零错误/跳过；回执SHA `bbadec79ff91623b8bcf96a80ab65c72f1d0882e121e936f1a8dfd24b7dfefcb`。这是测试替身，真实浏览器证据来自自己的Full和j/k，不能相互替代。

完成后只清理h/i/j/k新建隔离输入目录中的8个JAR临时副本，共1,434,319,238B（约1.34GiB）；删除前逐个核对真实路径、所有权、单链接、只读模式、完整SHA及自有运行清单为空，原件/只读备份/两个CI下载前后SHA未变。这些副本可由保留原件/CI字节重新生成；没有删除其他cache目录或任何私人数据。清理回执SHA `7404a2d6a03627c3820fc76ab4c1bd6bde284ed864beaf21a3c3793181be0050`。

四条自己的CI和Native六作业均已独立接受，接受器源码SHA `f6f3e0f9ba96218f0ee3eacb71fdf31eb8a5bc0afecfe8f5bc2bed865ed805ad`。验收后结果先保存本机，待后续有效增量提交，避免以文档回填触发另一轮重复大构建；代码bb6afa4e已推送。仍待最新整镜像本机/长测、跨站及真实认证、所有字段/字符集、OPDS、其他日志/错误响应的信息暴露与生产验收。准确原件+当前JAR的生成默认form修正已验证，但不能据此宣称项目总体完成。

随后增加的[认证/跳转/子资源三方诊断](HEADER-SECURITY-CHARACTERIZATION-2026-10-10.md)发现跨源Authorization外溢：原版历史链路同样存在，Camoufox还会在POST303跟随时带过去。12份成功完整JSON相同不能替代安全接受；当前尚未修复，strict安全保持false，不发版部署该认证路径。此次不修改上述a873业务字节或把父绿灯扩称安全通过。
