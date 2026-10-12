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

## 回退与已知限制

在独立干净工作树恢复固定f5要求即可回到旧专项入口；保留新回归、全部旧失败和原报告，不reset用户工作区。回退会再次拒绝最新JAR，不能把它称为真实源兼容修复。现有真实单书合计预算失败、原POST类型失败根因未定、起点/真实认证、最新三方及OPDS等未完成项继续保留。
