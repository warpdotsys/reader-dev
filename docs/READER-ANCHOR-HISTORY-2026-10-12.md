# EPUB 锚点历史：真实复现与可读源码修正

更新日期：2026-10-12。仅生成账号和生成 TXT/EPUB，未读取真实正文或浏览器会话，未改生产。当前产品基线为 `605cf80de7a1e4baeccb6fcc2f4f9344161f689b`；其四条托管运行状态已活验success，但不是下述新候选的验收结论。

## 已从恢复 JAR 实际复现

使用真正下载并核对的6fee共享JAR（285,672,774B），独立Windows实例只绑定127.0.0.1:18966。工程原有真实Chrome旅程先注册/登录、导入生成书并正常读到第二个书内小节；随后调用安装在实际Vue应用上的Router，从同一个backend index=0的中文锚点push到`intro`。地址已成功变化，但iframe仍停留旧位置，原30秒定位断言超时。

原始JUnit报告：1方法、1失败、errors/skipped0、47.292秒，SHA `882e62bb…4241`；堆栈明确落在新`verifySameChapterFragmentHistory`的第一次`intro`定位。源码快照及编译类保留，失败没有被下一次成功覆盖。这里证明的是恢复版Vue3问题，不是原始3.2.14 Vue2也有同一问题。

## 已从维护源码确认并修正

原ReaderView只监听`route.query.chapter`。同章只改`epubAnchor`不会触发定位；跨章历史入口也可能让旧滚动位置覆盖显式锚点。现有书内点击能跳转，是因为它另有直接更新fragment/navigationId的处理，并不能证明浏览器历史入口正常。

新增可读`readerRouteNavigation.ts`把已有章节与锚点选择分开，ReaderView同时监听两个参数：

- 只选择已有、非卷标、安全整数backend index；多值/无效章节拒绝，同章锚点不伪造新章节。
- 同章只改变当前iframe的定位，不重取正文、不重新下载整书或重建文档；中文、旧式name锚点、缺锚点警告仍交给原安全iframe处理。
- 无`chapter`参数时沿用当前真实index；没有强迫续读入口改成第0章。
- 跨章显式锚点作为一次pending目标，仍用原startFromTop跳过旧保存位置；常规reload继续保持原“相同锚点保存位置优先”行为。
- 自己翻章后同步query/删除旧锚点不重复覆盖pending；已有书内点击已选同目标不二次恢复。旧A实例仍受`isCurrentReaderRoute`保护。

没有改Java/Kotlin接口、`{url,index}`进度传输、命名空间、存储、iframe sandbox、脚本净化、Worker/下载超时、资源上限或内置浏览器worker。没有字节码补丁或业务重试。

## 已成功本机执行，正式产物尚待自己的托管构建

| 实际旅程 | 范围 | JUnit结果 |
| --- | --- | --- |
| 原6fee完整JAR | 新同章回归，在第一次intro定位真实失败 | 47.292秒，1失败/0跳过 |
| 候选A本机预览 + 准确6fee后端 | 同章push/back/forward、命名锚点、缺锚点及返回 | 26.565秒，1方法通过/0跳过 |
| 候选B独立新实例 | 上述全部，加无chapter入口、跨资源back/forward、显式锚点覆盖旧位置；原TXT/EPUB多模式、换书、刷新、CSS/预算/取消/期限回归仍执行 | 26.882秒，1方法通过/0跳过 |

同章实际断言证实getBookContent新增请求0、顶层document未reload、原iframe document仍为同一对象；未用mock响应替代。两张最终生成NCX/EPUB3画面经根代理实际查看，中文可读；PNG解码、非单色及SHA检查通过。它们是原旅程截图，其中NCX图处于保存滚动位置，不把截图冒称每一次history动作的逐帧记录。

两个新helper测试实际覆盖8个正向选择及无效值/卷标/缺条目/多值/普通文本/自同步等负例；本机全前端313、类型检查与两次Vite构建通过。使用本机既有依赖，不是新的npm ci；候选前端通过独立Vite端口18967代理后端18968，不是新候选完整JAR或最新Linux镜像。

Windows服务端Job维持2GiB、2核配额/亲和性、256进程，三次观测的Job提交内存峰分别450,281,472 / 502,726,656 / 483,614,720B；实际测试服务器及预览均已停止。这里的进程数不是Linux线程PID计量，Chrome客户端在服务端Job之外，不能接受合计2GiB压力、UID10001、零swap或生产容量。

[有限本机证据](evidence/reader-anchor-history-local-2026-10-12.json)SHA `c0979b17…c5c1`，原XML和Job结果逐项检查后投影，公开文件又精确比较。工作区源码指纹在执行后采集，不伪装为开始时原始manifest；记录明确不接受正式托管候选、完整镜像、原件三方、真实账号/书或压力整轮。公开摘要无用户名、token、书内私有名称、正文或原异常文本，58份用户报告/原JAR/只读备份/授权书哈希仍不变。

## 尚未验证与回退

新候选必须由自己的GitHub托管运行重新构建JAR/镜像并实际执行新增浏览器断言，不能借605或9e的绿灯。真实单书合计压力、实站认证、准确原件/历史/current三方、旧请求头/正文类型故障的根因定位、OPDS服务和生产切换等既有缺口仍保留，当前goal不完成。

回退在独立干净工作树定向恢复ReaderView旧章节watcher并移除helper import；保留新增回归，回退将重新暴露已复现锚点错误。不得reset用户工作区、删除原失败、改Cookie/storage或用延长定位等待判绿。当前不合并、发版、推registry或部署。
