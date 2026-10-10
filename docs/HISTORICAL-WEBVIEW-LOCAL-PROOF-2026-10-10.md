# 固定历史 WebView 的本机运行证据

后续独立轮次已实际运行准确原件／当前 JAR／内置 Camoufox 的生成三方，两轮业务比较一致但 PID 资源门禁失败，见[三方原始失败及当前结论](EXACT-JAR-GENERATED-THREE-WAY-2026-10-10.md)。下文仍是先前仅参考接口阶段的原快照，不倒填其“未启动 Reader”或机器记录。

## 结论与边界

2026-10-10 已完成两个非 root OCI 容器共享无外网网络的生成验证，并从官方 Docker Hub 取得固定 AMD64 历史 WebView、独立复算 manifest/config/四个压缩层及四个展开层的 SHA，随后实际运行它的生成接口。未使用 Docker 守护进程、安装新浏览器、改系统代理、停止/重启 WSL、复用私人浏览器会话或改生产服务。

这不是原件＋历史 WebView、当前 JAR＋历史 WebView、当前 JAR＋Camoufox的三方验收。准确原件基础九项对照仍以[独立基础记录](EXACT-JAR-BASELINE-2026-10-10.md)为准；本次没有启动 Reader、读取/上传原件、真实 Cookie 或书籍正文。也不能证明固定公开镜像曾是用户生产使用的远程实例。

## 实际输入身份

- 先前读取的 index：`sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`；本次没有独立复算 index blob，不将描述符观察改写成已复算。
- 独立复算的 AMD64 manifest：`sha256:88c250043bd33715ca372b749c1dca055c14e4550882caf968bd7af933d53573`。
- 独立复算的 config：`sha256:5d85fb04eb6a8900d56956f5598b50487209b72c736441bc954221ae8ac4e6bb`。
- 四压缩层379,489,069 B，四展开tar流1,056,643,584 B；全部压缩SHA及config给定diff-ID实际流式校验通过，没有重复下载大层。
- 配置声明Node/index.js、8050、版本3.2.0、默认root；实际诊断强制UID/GID10001、附加组空、能力全零、禁止提权。旧`/root`目录权限700没有修改，原缓存以只读挂载接入独立可访问路径，并通过PLAYWRIGHT_BROWSERS_PATH定位；未添加禁用沙箱的环境变量或参数，不据此宣称WebKit默认沙箱已独立验收。
- `app/index.js`5,391 B／SHA `963b7365504cc19a0345aa9cd83b3ea52583da11cbc5ae44d9b64d48512af030`，文件未改。

容器只共享user/net，mount/pid/ipc/uts分别独立。两个实际进程在宿主核对映射身份、能力、CPU集合、cgroup、namespace inode及只读挂载后才获准执行；只存在loopback。官方uidmap现已实际查询为`1:4.17.4-2ubuntu3`，既有subuid/subgid文件没有修改。

## 真实失败与执行结果

共享网络a/b失败保持原始失败记录：运行器把生成stdout文件所有者改为映射UID110000而保留GID1000，0600使原启动者无法读取回执；b有限诊断定位至读取生成回执，errno13，后续宿主stat确认文件所有者与模式。c只将新一轮生成stdout设为0640、核对GID1000，不增加世界读取权限或放宽运行身份/预算，实际共享网络收发成功。没有修改a/b源文件或报告。

| 阶段 | 秒 | 父级峰值 B | PID峰值 | max/OOM/PID触限 | swap | memory.high事件 |
| --- | ---: | ---: | ---: | --- | --- | ---: |
| 共享网络a（失败） | 4.935 | 41,549,824 | 15 | 0 | 0 | 0 |
| 共享网络b（失败） | 4.895 | 41,508,864 | 14 | 0 | 0 | 0 |
| 共享网络c（通过） | 1.550 | 56,573,952 | 15 | 0 | 0 | 0 |
| 固定参考下载/校验/解包 | 63.167 | 1,611,988,992 | 7 | 0 | 0 | 55 |
| 固定参考生成接口运行 | 35.973 | 888,135,680 | 217 | 0 | 0 | 0 |

各阶段总限都是2 CPU／2GiB／256PID／零swap，memory.high=1.5GiB，没有提高硬上限。解包末尾anon约13MB、file约1.46GB，父级峰值包含文件缓存，不是只计算程序RSS；55次high事件是实际触发观察，但不证明单一改动独自导致全部内存改善。只对新下载的自有层给出DONTNEED建议，没有清全局缓存。

旧服务在非root环境实际启动，生成GET/POST均HTTP200且各自标记正确；页面脚本执行；字符串、对象、数组、数字脚本结果的实际返回字节SHA符合既定生成值。这六项正向用例的探针实际退出0，没有用启动驱动0替代业务观察。

**仍然存在的旧实现缺陷**：sourceRegex用例确实访问`/regex-page`与`/regex-resource`，但监听不完成，20秒超时；保持`response-listener-hangs`记录，不把它改成通过。公开源码中`response.request().url.match(...)`与`url()`调用的差异提供原因证据，但本轮没有修改参考实现。本轮是带明确已知缺陷的有限正向验收，不能宣称该引擎所有功能可用或无泄漏。

两个测试容器随后均独立观察stopped/pid0、精确删除、自有容器剩余0；宿主最终查询相关五个单元均inactive/MainPID0。参考Node没有采到优雅退出回执，因此不宣称它自然退出或应用退出码0；回收证明只到容器/进程边界。

## 当前提交的独立托管回执

源码`c40f27f507f4010a1ad0887a08c7352ed5e4934d`，实际测试merge `116790904e470178564f1fb29f51c5df75bda9af`，四条流水线全部完成成功，已独立下载小型XML/PNG/JSON并核对来源；不是借父版绿灯。

- [Java38011300587](https://github.com/warpdotsys/reader-dev/actions/runs/38011300587)：43个JVM suite／181测试／31项既有skip，11项TTS编辑全执行；429项Python实际执行、无skip；两次clean JAR逐字节一致。
- [Vue38011300581](https://github.com/warpdotsys/reader-dev/actions/runs/38011300581)：25条实际浏览器流程、无skip；296项前端及20项截图守卫；11张生成设置截图，另人工查看编辑、过期、重名冲突、OPDS未实现四张。人工查看不是生产界面验收。
- [Full38011300626](https://github.com/warpdotsys/reader-dev/actions/runs/38011300626)：Camoufox20／Chromium23／helper3实际执行，默认UI、异步及metadata回执通过，峰值835,575,808 B／PID198。
- [Native38011300576](https://github.com/warpdotsys/reader-dev/actions/runs/38011300576)：六个实际作业含两架构构建、重载、publisher导入，24份publisher JSON独立通过；峰值892,162,048 B／PID207。普通短测仍允许1GiB swap、实际0，不能称其零swap长测。

全部当前JAR都为e59／285,665,012 B／1,569 entries；本轮没有重新下载大Native归档或本机运行最新完整Native镜像，归档SHA是经来源核对的publisher回执声明，不是本机大文件复算。固定参考运行不能代替当前镜像600秒长测、真实认证书源、SSE/下载、存储全范围、生产界面或正式部署验收。

[有限机器记录与本机原始回执SHA](evidence/historical-webview-local-proof-2026-10-10.json)。新测试缓存保留用于下一步三方对照；回退只停止/停用精确自有测试单元，不删除原报告、原件、浏览器缓存或用户工程。58份受保护记录与原b26 SHA末尾再次一致。
