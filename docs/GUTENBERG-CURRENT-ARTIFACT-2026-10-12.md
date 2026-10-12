# 真实网络书验证：绑定受测产物，不固定旧版本

更新日期：2026-10-12。仅固定Gutenberg 84匿名书源；不搜索其他站点、不导入真实凭据/本地正文、不改生产。修正的是验证工具的产物身份传递，不是通过放宽规则解决站点故障。

## 已从代码确认的阻断

原Reader API、实际UI和UI报告检查均固定历史f5c9 JAR，native wrapper也直接拒绝其他SHA。6a133dfe锚点修复后的正式构建已经产生30e8 JAR，因此不能让旧f5验证结果充当新界面交付证明。

新的维护入口使用`--expected-jar-sha256`，要求小写64位十六进制，并在任何业务请求/浏览器启动前核对运行镜像里的`/app/reader.jar`完整SHA。身份来源是先通过`release-native-artifacts.mjs check`验证的导入metadata；后续loaded image检查、实际服务端JAR散列仍保留，API探针、独立UI客户端和报告消费者接收同一值。报告消费者还把revision/JAR/network/publishedPorts与独立运行身份文件精确核对，拒绝混用或布尔数值别名。

旧脚本调用默认仍固定f5，便于重验历史证据；它不会从新报告自身推断expected SHA。固定书URL/28目录规则/首章、HTTP和ReturnData断言、API与界面逐字比对、Cookie及命名空间、TLS和私网限制、2CPU/2GiB/256PID/零swap、UID10001、客户端清理和不得输出正文等门槛不变。UI依然是强制旅程，不降级为API绿灯。

## 已成功本机检查，实站仍须实际执行

77项Gutenberg相关测试通过；完整Python729项/33.721秒通过，1项既有Windows/POSIX平台跳过，Bash语法检查通过。新增4项回归包含无效expected身份、旧/新SHA混用、revision/network/端口错配和workflow传递；它们是工具回归，不是实际网络阅读。

实际专项应消费已经终态成功的Native运行，明确记录两种身份：探针源码提交，以及被测试镜像的source/merge SHA。相同镜像真正重新载入运行并执行注册、登录、读章、翻章、刷新、返回和退出之后，才接受新网络书UI结果。未下载到或原生发布演练未完成时不得另起一个构建来代替等待。

## 6a 产品的正式 JAR：已有有限证据

产品源码`6a133dfe81e9397415a5d7dd8264ecc5e034013c` / 合并快照`08df81bfea5ae6c35f2475172db002d7e643ecbb`。

- [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/38194621952)、[Vue](https://github.com/warpdotsys/reader-dev/actions/runs/38194621922)、[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/38194621981)各自的原始小报告已独立接受；[Native](https://github.com/warpdotsys/reader-dev/actions/runs/38194621977)捕获时仍在自己的重导入/发布导入流程，不能提前接受整轮。
- 本轮双clean完整字节相等，准确JAR SHA `30e8cafcea6b07781aae8039c5ae0230340c189884f61eba445bab72e5b440af`，285,673,034B/1570条目；共享JAR真正下载并在本机重新散列，包内worker仍为50497908。
- 逐条流式比较旧6fee和新30e8，差异仅Vue资源；业务class、依赖、MANIFEST以及worker内容不变，不是原始3.2.14完全兼容证明。
- 自己的前端313/实际生成UI25、Python725/零跳过、JVM207/35既有环境跳过、Full Cam24均核验。新增锚点完整阅读旅程在正式JAR入口实际执行22.386秒/零跳过；完整镜像默认TXT/两章EPUB旅程12.623秒。Full合成服务端子集峰876,380,160B/PID201、普通swap允许1GiB但观察0，不代替合计压力或实站。

这些三条成功与新JAR字节不能替代参数化工具自身的CI，也不能直接宣布新镜像网络书、原件/历史/current三方或真实认证已完成。当前goal仍active，不合并/发版/推registry/部署。原件、授权书和58份用户报告哈希保护不变。

## 本轮最新镜像真实网络 UI 已独立接受

探针源码d814aaed / [实际专项38196263456](https://github.com/warpdotsys/reader-dev/actions/runs/38196263456)，消费产品6a / 受测08df自己的Native运行38194621977，AMD64 image ID `1960671e…dc9cf`、准确JAR30e8。四条6a正常工作流及Native六个作业现已各自独立接受，32publisher文件和两架构真正重导入核验；这不替代探针d814自身仍需分别验收的普通CI。

专项在GitHub托管原生runner真正载入既有受测镜像，运行原Guarded Reader API和强制默认Vue3 UI，没有mock响应、重建镜像或依赖宿主Chrome。实际注册、登录、入架，阅读0→1→刷新1→返回0并退出，21.058秒；6,822/7,332字符四次画面与实际API在仅空白归一后相同，28目录、替换字符/错误/阻断请求/观察错误0、会话撤销及两测试容器移除均验证。

15份有限文件重新取得原ZIP，SHA `c9b1f3ad…6aeda`与GitHub artifact digest一致，所有解压字节逐项相等。两张登录/退出图经过PNG像素护栏，根代理也实际查看，中文可读、不空白；正文画面未导出。消费者使用明确30e8而非历史默认f5，旧失败与旧成功记录不修改。

服务端同生命周期峰1,029,738,496B/PID207，独立客户端峰791,740,416B/PID168，各自2CPU/2GiB/256PID、配置及实际swap0，hard/OOM/PID事件0。客户端使用同镜像中的打包浏览器，但只是临时外部UI测试客户端，不是部署所需的WebView服务。两个预算不能宣称合计2CPU/2GiB/256PID通过，也不证明真实单书压力或生产容量。

[本轮有限公开摘要](evidence/gutenberg-current-image-ui-2026-10-12.json)SHA `862d77ba…ad94d`，绑定原独立回执`3760d546…24a8b`并精确投影/比较，分别记录探针与镜像源码。公开报告无正文、Cookie或真实账号，原件和58份用户数据报告未改。[最新原件/历史/当前快照三方](GUTENBERG-30E8-THREE-WAY-2026-10-12.md)为另一次无WAN实测，不将两个成功拼为实站HTTPS三方或真实认证。

## 回退与已知限制

在独立干净工作树恢复固定f5要求即可回到旧专项入口；保留新回归、全部旧失败和原报告，不reset用户工作区。回退会再次拒绝最新JAR，不能把它称为真实源兼容修复。现有真实单书合计预算失败、原POST类型失败根因未定、起点/真实认证、最新三方及OPDS等未完成项继续保留。
