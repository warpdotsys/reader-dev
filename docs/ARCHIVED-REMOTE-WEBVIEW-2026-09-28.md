# 旧远程 WebView 镜像的隔离参考探针

日期：2026-09-28。此探针使用 Docker Hub `hectorqin/remote-webview:3.2.0` 的固定多架构摘要 `sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`，在 [GitHub 托管 runner 首轮](https://github.com/warpdotsys/reader-dev/actions/runs/36420960778)及[修正脚本返回值后的复验](https://github.com/warpdotsys/reader-dev/actions/runs/36437640038)的临时容器中运行。服务只映射到 runner 的 `127.0.0.1:18050`，限制 2 GiB 内存、2 CPU、256 PIDs，并给浏览器 1 GiB `/dev/shm`；没有挂载生产数据或用户凭据。该镜像是可追溯的**旧实现参考**，不是“此前生产远程 WebView 实例”的版本证明。

请求体按 `RemoteWebview.kt` 的 `/render.html` JSON 字段构造，合成站点只返回固定 HTML 和无意义的测试 Cookie。探针不打印页面正文或 Cookie 值。

| 样本 | 已观测结果 | 边界 |
| --- | --- | --- |
| 合成 GET | HTTP 200；正文 193 字节，SHA-256 `00001ef0489b189d67f3253a2ce0ffb4cd652fb5374b3dc00541f5d0a136cba5`；固定标记与页面内联脚本结果存在 | 只证明该参考镜像可取回此合成页面 |
| 合成 POST | HTTP 200；正文 194 字节，SHA-256 `a5188edc4aa1af1e3383a7c2230b8ff4b9d341841a0708f3574631ac5beaf9d6`；合成服务器记录 POST | 未验证代理、复杂请求头或重定向 |
| 公开目录 `https://m.jjjxsw.com/txt/` | HTTP 200；正文 16,403 字节，SHA-256 `38c911694a9d98532883a34913a9c6bb68b11b368c635fbd005ffb6cdbc045e3` | 只记录渲染结果摘要；未与 Camoufox 的原始 HTML 同条件逐字节比较 |
| `js_source` 返回字符串 | HTTP 200；正文 `script-result-ok`，16 字节，SHA-256 `fa3669b1c8bce6ac9a8d9df19da388dad91377d7345399afd762012c05f9939e` | 只证明返回值型脚本；页面副作用也执行了 |
| `js_source` 返回对象/数组/数字 | 均 HTTP 200；对象 JSON 正文 26 字节、SHA-256 `f40d271486d8bc12ae170e30de191f09149ac8cd2c4208ace1a2464c93fba67c`；数组 JSON 正文 11 字节、SHA-256 `0fb5609573d0a6bc52f19b42db1f52a022b2baa2ee9899e38184b8337494ee94`；数字正文 `42`，SHA-256 `73475cb40a568e8da8a045ced110137e159f890ac4da883b6b17dc651b3a8049` | 对象/数组使用紧凑 JSON；这些是旧参考实现的实测值，不是原 JAR 的结果 |
| `sourceRegex` 命中资源 | [单独探针](https://github.com/warpdotsys/reader-dev/actions/runs/36439381876)中合成服务收到了 `/regex-page` 与 `/regex-resource`，但 `/render.html` 20 秒未返回，容器未 OOM；[复验作业](https://github.com/warpdotsys/reader-dev/actions/runs/36439959686)将此预期缺陷明确记录为 `response-listener-hangs` | 镜像源码监听器写的是 `response.request().url.match(...)`；[Playwright JavaScript API](https://playwright.dev/docs/api/class-request#url)要求 `request.url()`。这是该固定参考镜像的已知缺陷，不能外推至原生产实例 |

合成 GET/POST 的响应都未向调用方暴露 `Set-Cookie` 响应头；合成页面确实发送了测试 Cookie。这是此参考镜像在该协议路径的观察值，不推断它的内部 Cookie 状态或原生产实例的行为。

2026-09-29 对同一固定摘要镜像在[另一条 GitHub 托管 runner 作业](https://github.com/warpdotsys/reader-dev/actions/runs/36515280090)复验：合成 GET/POST 正文长度和 SHA-256 与上表一致，脚本字符串、对象、数组、数字也一致；`sourceRegex` 再次在实际请求到 `/regex-resource` 后进入已知的监听卡住分支。公开目录仍为 HTTP 200，但正文为 16,279 字节、SHA-256 `694505e1c5f83069f6c83ba7631c372b55d19f253a73d64b7ef868f1968b7144`，与 2026-09-28 的 16,403 字节不同。公开页会变化，跨日期 HTML 摘要不构成候选与参考实现的同条件差分；本次也未连接原生产远程实例。

2026-09-29 在[同一次托管运行](https://github.com/warpdotsys/reader-dev/actions/runs/36529322389)中并行执行本固定镜像探针和完整 Reader/Camoufox 镜像。旧镜像直接渲染公开目录为 HTTP 200、16,279 字节、SHA-256 `1771f681b205d0d383db2fc92d95699e68a7f3e7cf152c8f620b2522583201fe`；再以 `document.querySelectorAll('.booklist_a .list_a').length` 读取到 **10** 个 DOM 节点。后者于 UTC 06:06:22 观测；Reader 的搜索结果于 UTC 06:13:36 观测，两次相隔 433.381 秒，且请求阶段不同。比较制品 `public-webview-comparison` 仅记录状态、数量、摘要和时间；两侧数量相同，但未逐字段比较书目，更未证明 HTML 字节一致。原 JAR 与原生产远程实例均未参加本轮。

[下一轮托管作业](https://github.com/warpdotsys/reader-dev/actions/runs/36531083960)显示：旧镜像的独立“返回纯数字”脚本有一次只得到不可解析的结果，因此比较作业按设计报错；同一旧镜像的书目数组投影和 Reader 搜索却都产生 10 项、相同的书名/URL SHA-256。不能因两个子作业成功而把整次红灯记成绿灯。`a53cc7be` 随后去掉冗余的数字脚本，直接用数组长度与摘要比较；[修正后的完整配对作业](https://github.com/warpdotsys/reader-dev/actions/runs/36531865466)四个适用作业全部通过：参考与 Reader 的投影均为 10 项，SHA-256 均为 `46c56e39a396824f6cc2d7ff920791f5f6810419c3d60d2e271740d7817ce561`，两次观测相隔 237.914 秒。原页面 HTML 摘要在不同请求间可变，所以这仍只证明此时段、此解析投影一致，不证明原生产服务或字节级页面一致。上传制品不包含书名、正文、凭据或 Cookie 值。

探针最初只等 TCP 端口开放，首次请求偶发连接重置；在浏览器启动后等待并对**第一笔**请求做最多三次有界重试后，GET/POST 与公开页稳定完成。早期作业中单独调整 `/dev/shm` 或额外的 `no-new-privileges` 后仍出现重置，不能把这些失败误报为内存溢出。首轮 `js_source` 只执行页面副作用，没有返回值；镜像 `/app/index.js` 实际读取 `page.evaluate(js_source)` 的返回值，空值会重试，所以那次 20 秒超时是**无效的接口形态探针**，不是已证实的服务故障。改为返回字符串后通过，随后对象、数组和数字返回值也通过。该镜像使用 WebKit/Safari UA，内置候选使用 Camoufox/Firefox；这仍是潜在兼容差异。

这份结果可作为后续请求形状、正文和 Cookie 差分的参考输入，但**不是三方验收完成**：本地原 JAR 没有连到这个临时服务；此前生产远程实例不存在可核验容器；公开镜像站同名 Reader JAR 也已证实与本地原件不同（见[来源核验](ORIGINAL-JAR-PROVENANCE-2026-09-28.md)）。下一步需在可追溯的隔离网络中，用同一书源规则同时驱动本地原 JAR、该旧参考服务和内置 Camoufox，并分别记录脚本/超时差异。
