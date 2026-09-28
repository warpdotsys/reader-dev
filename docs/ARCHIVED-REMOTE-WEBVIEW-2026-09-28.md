# 旧远程 WebView 镜像的隔离参考探针

日期：2026-09-28。此探针使用 Docker Hub 官方 `hectorqin/remote-webview:3.2.0` 的固定多架构摘要 `sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`，在 [GitHub 托管 runner](https://github.com/warpdotsys/reader-dev/actions/runs/36420960778) 的临时容器中运行。服务只映射到 runner 的 `127.0.0.1:18050`，限制 2 GiB 内存、2 CPU、256 PIDs，并给浏览器 1 GiB `/dev/shm`；没有挂载生产数据或用户凭据。该镜像是可追溯的**旧实现参考**，不是“此前生产远程 WebView 实例”的版本证明。

请求体按 `RemoteWebview.kt` 的 `/render.html` JSON 字段构造，合成站点只返回固定 HTML 和无意义的测试 Cookie。探针不打印页面正文或 Cookie 值。

| 样本 | 已观测结果 | 边界 |
| --- | --- | --- |
| 合成 GET | HTTP 200；正文 193 字节，SHA-256 `00001ef0489b189d67f3253a2ce0ffb4cd652fb5374b3dc00541f5d0a136cba5`；固定标记与页面内联脚本结果存在 | 只证明该参考镜像可取回此合成页面 |
| 合成 POST | HTTP 200；正文 194 字节，SHA-256 `a5188edc4aa1af1e3383a7c2230b8ff4b9d341841a0708f3574631ac5beaf9d6`；合成服务器记录 POST | 未验证代理、复杂请求头或重定向 |
| 公开目录 `https://m.jjjxsw.com/txt/` | HTTP 200；正文 16,403 字节，SHA-256 `38c911694a9d98532883a34913a9c6bb68b11b368c635fbd005ffb6cdbc045e3` | 只记录渲染结果摘要；未与 Camoufox 的原始 HTML 同条件逐字节比较 |
| 额外 `js_source` 请求 | 合成服务器收到第三次 GET，但 `/render.html` 在 20 秒内未返回；探针失败 | 只证明本次脚本形态和运行条件下的超时，不能推断所有真实书源脚本都失败 |

合成 GET/POST 的响应都未向调用方暴露 `Set-Cookie` 响应头；合成页面确实发送了测试 Cookie。这是此参考镜像在该协议路径的观察值，不推断它的内部 Cookie 状态或原生产实例的行为。

探针最初只等 TCP 端口开放，首次请求偶发连接重置；在浏览器启动后等待并对**第一笔**请求做最多三次有界重试后，GET/POST 与公开页稳定完成。早期作业中单独调整 `/dev/shm` 或额外的 `no-new-privileges` 后仍出现重置；最终脚本超时期间容器仍在运行且 `OOMKilled=false`，不能把这些失败误报为内存溢出。

这份结果可作为后续请求形状、正文和 Cookie 差分的参考输入，但**不是三方验收完成**：本地原 JAR 没有连到这个临时服务；此前生产远程实例不存在可核验容器；公开镜像站同名 Reader JAR 也已证实与本地原件不同（见[来源核验](ORIGINAL-JAR-PROVENANCE-2026-09-28.md)）。下一步需在可追溯的隔离网络中，用同一书源规则同时驱动本地原 JAR、该旧参考服务和内置 Camoufox，并分别记录脚本/超时差异。
