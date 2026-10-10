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
