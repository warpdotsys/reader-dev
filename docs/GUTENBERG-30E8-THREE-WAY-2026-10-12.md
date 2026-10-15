# 原件 / 历史 WebView / 新30e8：同一公开页面快照三方

更新日期：2026-10-12。实际执行，不是从后端class未改变推断兼容；仅固定公开Gutenberg 84页面，没有真实凭据、私人书籍或生产改动。

## 准确输入与维护入口

原件SHA b26fb476、当前恢复JAR完整SHA `30e8cafcea6b07781aae8039c5ae0230340c189884f61eba445bab72e5b440af`，后者确属产品6a133dfe / 受测08df81bf自己的双clean及下载字节。历史引擎使用已独立核对的manifest `88c25004…573`；Camoufox库/浏览器来自缓存runtime401d，不更改其二进制，但实际运行新的30e8 JAR及包内50497908 worker。这不是本机最新完整镜像证明。

采集的真实公开HTML固定SHA `23403a6a…58f`，468,370B。三个路径使用同一只读HTML、同一HTTP loopback别名和同一源规则；无WAN，因此不称真实站点HTTPS或真实认证三方。当前镜像的真实HTTPS及实际网络UI由[独立实站专项](GUTENBERG-CURRENT-ARTIFACT-2026-10-12.md)另证，两个测试范围不能拼成一个未执行的过程。

维护比较器增加`--expected-restored-sha256`及同值的报告验证入口；原件b26和公开快照SHA仍固定，未知恢复身份不能从报告自身猜测。默认保留旧f5调用用于历史重验。新增2项身份回归，相关79项及本机完整731项/35.855秒通过（1项既有Windows/POSIX跳过），不把这些单元测试当实际原件执行。

## 已实际通过的完整 ReturnData 比较

每侧真正执行元数据、完整28目录、第一和第二章四个业务API，不重试。HTTP200与isSuccess/空errorMsg验证，完整JSON类型与字段比较，没有只比标题或摘要。

- 元数据完整非时钟ReturnData相等；三个原默认时间字段各在自己的实际请求窗口，字面完整JSON仍不相等，原样记录，不抹平所有动态字段。
- 28条目录的顺序、索引、标题、相对URL、options及完整ReturnData相等。
- 两章完整ReturnData相等，正文分别6,822与7,332字符，无替换字符；原正文只在内存中比较，公开摘要不包含正文。
- 三侧源规则保留、退出会话、原件执行和实例停止均确认。旧失败/旧成功回执不反写。

运行55.936秒，systemd59.260秒/CPU73.390秒；UID/GID10001、空附加组、零caps/no-new-privileges，私有仅lo网络，独立mount/pid/ipc/uts及只读输入由外部宿主核验。历史引擎必须先实际stopped/PID0，才允许当前Camoufox阶段；只定向回收该已停止自有叶节点，不清全局、不改Reader预算。

总预算2CPU/2GiB、high1.5GiB、256PID、配置及实际swap0保持。峰1,611,595,776B/PID243，soft-high3292，hard/OOM/PID触限0。仍接近高水位、PID余量13，不能宣布真实大书或容量修复。两个自有容器实际stopped/PID0/delete，inventory0，单元另外确认inactive/dead/MainPID0/Result=success。

[有限完整比较证据](evidence/gutenberg-30e8-snapshot-three-way-2026-10-12.json)SHA `ff2ba7ba…e47a`，绑定原始回执SHA `a64d2e32…a4b4`、执行源码指纹、当前JAR/worker、原件及采集输入。根代理逐项重验后投影并精确比较，未由导出的哈希再次冒称逐字重放。原件/只读备份/授权单书及58份用户报告哈希保护不变。

## 尚未证明与回退

此处没有实站HTTPS/真实账号的三方、最新完整镜像的本机重导入、严格请求头安全三方或真实单书合计压力。旧POST类型失败的准确根因仍未定位，原其他缺口不关闭，goal仍active，不合并/发版/部署。

回退仅在独立干净工作树恢复比较器的历史固定f5入口；保留新回归及本轮原始证据。不得reset用户工作区、放宽源规则/时钟窗口/资源或延长超时来伪造兼容。
