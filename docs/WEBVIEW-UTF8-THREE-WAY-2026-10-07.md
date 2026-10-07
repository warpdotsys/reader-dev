# 生成 WebView UTF-8 六用例三方复验

**后续 G 轮已收齐三侧六例，严格等价仍红灯**：未包装 API 或修改历史引擎；原件／恢复远程／内置路径各实际 6 搜索／6 目标请求，完整生成 JSON 一致。历史 UTF-8 POST 两侧真实 44 B 截断，内置真实 60 B；显式已知缺陷诊断非零退出，默认严格门禁仍拒绝，不把差异改成通过。原预算、已停止历史引擎交接、两容器清理／独立剩余 0、两个 JAR／58 报告未变均已独立核对；详见[完整观察、限制及回退](WEBVIEW-UTF8-CHARACTERIZATION-2026-10-07.md)。下文早先失败保留原范围，不因 G 追认为修复。

## 范围与输入身份

本轮扩展既有无外网生成三方工具，不修改业务源码、存储格式或前端。旧 [2026-10-04 五用例](WEBVIEW-THREE-WAY-2026-10-04.md)继续按原规则验证，不能代表当前 worker，也不能冒充新六用例验收。

- 原件：`reader-pro-3.2.14.jar` SHA-256 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`，沿用服务器授权临时目录内的只读副本，未重传或覆盖原件。
- 新恢复输入：托管 native run `37580306817`／源码 `09163efe`／受测 `3aec4f9e0102626c9f47b74b6130a872ae541b27` 的 `reader-release-jar-3aec4f9e0102626c9f47b74b6130a872ae541b27`（artifact `11465045291`）。实际解包 JAR 285,662,082 B，SHA-256 `221d41efbab68fa61e2117a4db888c1d8996d671c99f4a034c233c55002d1a43`；包内 worker SHA-256 `01863dcd45eab836bc0e76e2f0512d97656112a43c8f7dd1ffa3e3e8845087be`。上传后再次核对 JAR 哈希；不是本机 F9 构建、旧 74 恢复件或随后 594 的独立 JAR。
- 运行时：服务器既存只读 `sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf`，amd64／UID 10001，**当前业务 JAR 与 worker＋旧锁定运行时**，不是当前最终完整镜像运行证明。旧运行时自带 JAR 不执行。
- 历史引擎：`hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`；仍不能证明原作者生产服务当年的真实版本。

输入、新 probe 和 run 放在原授权根目录内的新 `webview-threeway-utf8-20261007-a` 子目录。旧输入／旧报告保留；不联网，不读正文，不复制 Cookie／真实账号，不改生产挂载、代理、映射范围或用户 18931 Reader。新输入和脚本设为只读，容器 bind 只读。

## 新增断言与复跑方式

工具 `--exercise-encoding` 默认关闭。显式六用例模式必须同时启用隔离原件、实际历史引擎、Camoufox、原脚本和 POST 五用例；不允许 synthetic `/render.html` 冒充浏览器执行。原有四 GET＋一个 POST 和 Cookie 回放／删除约束保留，第六次为独立 `/search-utf8` POST：

- 明确规则 `charset: UTF-8`／JSON Content-Type，正文含中文、`𠮷`、`😀` 及 `+ & %`；保存书源后逐字符读回 URL 规则。
- 目标服务直接对收到的原字节计数／SHA-256，再严格 UTF-8 解码；60 B，SHA-256 `8d0383070028f4f457194a194e8e9133df2487261e341c1da121171d59eeeef1`。不从 Reader 摘要重编码后冒充原字节，不用替换字符吞掉错误。
- HTML 原书名故意为 `WebView编码原始书`；必须由 webJs 改为 `WebView编码书𠮷😀 + & %`。三方全部相同乱码也会判失败。
- 完整 ReturnData／书目字段按严格 JSON 类型比较；缺响应、额外目标请求、旧五次、错误 byte count／digest、布尔伪装整数、丢失 Cookie 删除或原 POST 不接受。
- 失败可保留生成阶段报告（完成的搜索／最后观测搜索响应／实际目标请求），不填补缺失数据。原件或历史配对未通过时不许可 Camoufox 交接。

只在已授权 Linux Docker/systemd 主机、精确已有运行时下执行，输出必须是全新 `/var/tmp` 子目录；示例中的输入和目录需先落实为授权目标，不能把原件直接在 Windows 上运行：

```bash
sudo python3 -B scripts/run-three-way-webview-in-docker.py \
  --original /var/tmp/authorized-test/original.jar \
  --restored /var/tmp/authorized-test/hosted-restored.jar \
  --runtime-image sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf \
  --output /var/tmp/authorized-test/new-utf8-run \
  --exercise-encoding
```

实际两容器及其所有 Java／浏览器／probe 子进程合计仍为 2 CPU／2 GiB／零 swap／slice PID 512（单容器 PID 256），没有放行私网出口或发布端口。外侧 host PID 1 独立验证私有 loopback netns，UID 10001 probe 再核对 inode；历史两侧完成、原预算守卫通过、旧引擎实际停止和 8050 拒绝连接后才运行 Camoufox。保留累计峰值和触限事件，不重置、扣缓存或升预算。内部 360 秒 deadline，外侧自有服务 480 秒上限；只按本次随机 ownership label 清理自有容器。

## 本轮实际状态与限制

新工具纯测试全量 249 项：248 实际通过、1 Windows POSIX symlink 环境跳过，0 失败／错误；定向 33 项中 32 通过、1 同一环境跳过，包含新增 11 项。发布结构及 69 项 pipeline／身份／manifest 纯测试通过。纯 fixture／mock 验证不等于真实浏览器执行。

服务器已成功启动 `reader-threeway-utf8-20261007-a.service`，invocation `15dfdadf8378416895272a7681308b9e`。间歇 SSH banner 超时后，一次只读查询先确认终态 failed／Result exit-code／ExecMainStatus 1／MainPID 0。当时尚未读到具体原因；随后连接恢复，三个原报告已逐字节复制并核对 SHA-256，原报告保留，没有覆盖或重复启动该 unit。[原件失败与清理证据](evidence/webview-utf8-original-failure-2026-10-07.json)。

**准确失败位置是原件第一次 `/reader3/searchBook` 读取响应超时，不是原 JAR 无法启动，也不是第六个 UTF-8 用例失败。** 日志有 `ReaderApplication Started`，结合实际执行脚本的栈位置，之前的 getSystemInfo、生成账号注册／登录、书源保存及逐字符读回守卫已通过；这些 API 的完整原返回没有单独保存，不能补造。已完成搜索 0、目标实际请求 0、最后搜索返回 null；恢复及 Camoufox 侧尚未完成，无完整 `three-way.json`。

共同 2 CPU／2 GiB／零 swap／PID 512 实际上限成立，累计峰值 562,421,760 B，内存 max／OOM／oom_kill、PID 触限均为 0。业务失败，三个接受标志仍为 false，不因无触限改成整体通过。两只自有容器已记录移除，随后宿主按本轮标签独立查询剩余 0；两个 JAR 哈希不变。采样存在较长间隔，但旧报告没有 CPU 计数或历史引擎日志，不能唯一归因为宿主负载、渲染器或 Reader。

本机另三次原件 readiness 启动器尝试在 systemd `217/USER` 或 `setresuid EPERM` 前置步骤停止，三个 unit 均 MainPID 0，**Java 没有执行**；不是原 JAR 缺陷，未新增系统用户／改映射或放松权限。官方 uidmap 和映射工具已实际确认安装，旧受限浏览器 helper 的成功也不等于这三个启动器或当前完整 Reader 镜像验收。服务器原件已实际启动，因此不重复本机原件启动诊断。

## 本增量自身的托管验收

源码 `86c5047d`／受测合并快照 `3250682951e22698e992c3a835aba6370288ce95` 的 [Java](https://github.com/warpdotsys/reader-dev/actions/runs/37586778326)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37586778116)、[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37586778218)、[双架构演练](https://github.com/warpdotsys/reader-dev/actions/runs/37586778072)均终态 success；演练六项实际作业全部成功。仅下载小制品，在本机重新执行原身份／UI／异步／资源守卫，未下载新大镜像或上传新的 7c55 JAR。[独立托管证据](evidence/webview-utf8-hosted-86c5047d-2026-10-07.json)。

- Linux 日志 `Ran 249 tests in 8.793s`，随后 `OK`；这是纯 Python 测试，不能替代服务器六用例。
- 两份真实 Camoufox 各 19 项无跳过／失败／错误；两个 helper 各 3 项。共同 Java 162 项中 132 执行、30 门控跳过；Vue 18 核心＋1 子目录实际执行，书架几何 18 组和长分组 3 组通过；只目视本轮 320px 长分组生成图，不称全部截图检查。
- 24 份 publisher JSON 精确核对、5 组 UI／异步／资源通过；短时最高 940,109,824 B／PID 195，触限／OOM／实际 swap 0，正常 CI 仍允许 1 GiB swap，与服务器零 swap 不混用。
- 原生共同 JAR 为 `7c55ba75...`，Full 独立构建 JAR 为 `2c34ee04...`；两者不是服务器本轮输入 `221d41ef...`。58 份用户已有报告散列不变，无发布、registry 写入或生产部署。

## 保持原门禁的诊断增量

只改三方测试工具：每次采样可保存 slice 实际 `cpu.stat`／有限 `cpu.pressure`，不存在时为 null；不从当前内存分类扣缓存，不改原预算或接受条件。清理前先核对随机 ownership label，再保存该次历史引擎状态及各最多 32 KiB 的 stdout／stderr 尾部（每流最多 80 行，0600、新文件独占创建）。原始生成日志只保留授权临时 run 与本机 ignored 隔离证据目录，不上传正文／账号或将日志内容提交 GitHub；诊断失败不跳过容器 ownership 核对及清理。

新增 6 项纯测试，本机全量 255 中 254 通过、1 Windows POSIX symlink 环境跳过；WSL 定向 18 项全部通过。69 项发布结构／原生身份／manifest 纯测试通过。新日志、CPU 字段只提供定位材料，不把失败诊断当兼容成功，也不能追补 a 轮缺失数据。诊断后的 b 轮在全新 `webview-threeway-utf8-20261007-b/run`、相同 221 JAR／原件／运行时、原共同限额下执行；unit `reader-threeway-utf8-20261007-b.service`，invocation `183b062ef0794bcc9af9fbb7dd896c46`。启动时先确认 a 的 MainPID 0、原标签容器剩余 0；不与前一份重叠。

**b 轮仍在原件首次搜索超时**，48.097 秒最后采样，累计峰值 755,159,040 B、触限／OOM 0，两个 JAR 不变，两只自有容器已移除并独立查询剩余 0。32 份实际 CPU 观测保留；末次累计使用 22,553,350 us、节流 59 次／630,607 us，不据此唯一判断超时原因。历史服务已收到正确的生成 GET URL，并输出 contextOptions，清理前 Running true／OOMKilled false；目标请求仍为 0。固定镜像内真实源码在 `webkit.launch` 返回后才打印 contextOptions，说明这一处浏览器启动已完成；不能由日志猜测 newContext／newPage 哪一步卡住。[本次准确诊断和字节哈希](evidence/webview-utf8-diagnostics-2026-10-07.json)。

## 单独历史引擎诊断及已确认旧缺陷

只从未启动的专用 reference 容器读取固定镜像的 `/app/index.js`／`package.json`，文件共约 5.5 KiB，读取后按精确标签移除容器，未改原镜像。声明的 Playwright 范围是 `^1.25.1`，实际固定镜像安装的是 **1.36.0**；不把范围当准确运行版本。源码记录 POST `Content-Length: body.length`，这是 UTF-8 长度风险，先作为推断而非 Reader 故障结论。

在同授权目录、独立 only-lo netns／原 2 CPU 2 GiB 零 swap／能力全部为 0／NNP 下，单独 WebKit 的 launch、newContext、newPage、生成页 goto/content/close **实际 6.898 秒成功**，目标请求 1，峰值 780,365,824 B、触限 0、自有容器剩余 0。它只使用历史镜像既有 root UID／零 capabilities，**不是最终非 root Reader 验收**，没有启动任何 JAR。只能排除这一份 standalone 样本无法启动，不能替代跨容器／Reader 集成复验或解释先前超时。

随后另一个新隔离容器直接执行固定镜像原 `index.js`，发送同一生成 JSON 至真实 `/render.html`；不改其 POST 实现，不用伪响应冒充旧引擎。目标实际收到 **44 B／应有 60 B**，原字节 SHA-256 为 `c61ca3e5...`／应有 `8d038307...`，JSON 解析失败，而历史渲染 HTTP 仍返回 200；2.493 秒完成观测，峰值 726,691,840 B、触限 0、容器清理。**此为已实测的旧历史引擎 UTF-8 请求截断缺陷，不是六用例 Reader 三方通过**。诊断脚本退出 0／原报告 passed 只表示观测完成，明确 `postBodyLengthCorrect=false`；业务正确性仍红灯。没有将正确请求故意截断以追随旧缺陷，也没有修改三方守卫制造绿灯。

当前完整三方仍未完成：原件首次搜索超时需要继续定位；即使它恢复，新六用例必须明确记录历史旧缺陷与内置引擎的实际差异，不能预先声称所有侧正确或用旧五用例拼接第六结果。上述生成协议观测没有走 Reader，不能推测原件／恢复版 searchBook 的具体 ReturnData；新 diagnostic 工具自己的 hosted 状态与原先 86 提交分开。

即使该轮生成对照通过，仍不代表非 UTF-8 全部编码、真实网站认证／章节、当前完整镜像、长期容量或生产验收。此前本机起点解析／旧 worker 资源红灯仍保留。`594cec9c` collector 自己的 [hosted 分类验收](BROWSER-MEMORY-CATEGORIES-2026-10-07.md)单独记录，不拿它代替此业务差分。

回滚工具时不传新增 flag，可继续严格使用旧五用例；在干净检出撤销本测试工具增量即可，不影响产品 JAR／数据库。旧原件、输入、报告和失败结果保留，禁止 reset 用户工作区或忽略触限取绿灯。

## 后续控制诊断与未加包装器的失败

新独立双容器 GET 使用同一固定历史服务和原 Python 生成夹具，但不启动 JAR：UID 10001 目标端／capabilities 0／NNP／only-lo 守卫通过，实际 2.638 秒、HTTP 200／196 B／目标一次且正确书名，峰值 803,049,472 B、触限 0、两个容器移除并独立剩余 0。不能再由此前失败唯一归因为基础跨容器回环不通，也不能据此证明含 JVM 时稳定。

接着新的五例 Reader 诊断只在原镜像启动命令添加 logging-only 包装器，转发原 Playwright 方法及结果、记录阶段名，不改源码／镜像或造浏览器返回。原件、恢复历史路径、同 221 JAR 内置引擎各 **5 次**实际搜索／目标请求；完整生成 ReturnData、字段、POST 和 Cookie 规则再次独立通过。累计峰值 1,432,326,144 B、93 采样／末次 98.666 秒、原共同预算／零触限、历史停机交接及两个自有容器清理通过。wrapper 可能影响时序，不能用它追认未加包装器六例或解释 a／b 超时。

随后 c 轮真正去掉 wrapper，保留原严格六例及同输入／限额。两侧搜索循环完成后，**在历史配对 `validate_generated_searches` 的目标 GET／POST 字段比较处失败**，不是再次首次搜索超时；Camoufox 尚未获准执行，overallAccepted false。峰值 1,192,341,504 B、44 采样／触限 0、两个 JAR 不变、两容器清理／独立剩余 0。原 API 完整响应未在 handoff 前保存，不能根据控制流重建原数据或声称完整六例通过。

因此新增只读观察保留：六例历史两侧完成后、严格验证前，将完整生成返回及目标原字节字段独占写入 observation 文件；旧文件／符号链接在启动前拒绝。这个文件明确不代表接受，不放行 handoff，不伪填 Camoufox 结果；后续可精确区分旧截断与其他差异。[控制诊断及 c 失败哈希](evidence/webview-generated-controls-2026-10-07.json)、[第 20 契约与观察保留](CAMOUFOX-UTF8-POST-2026-10-07.md)。

## 原生 API 日志定位与不同前置失败

观察保留的 d 轮使用新只读 compare 输入 SHA-256 `47c2ebf4...`，三个隔离／预算脚本保持原字节身份；仍为同原件／221 JAR／190 运行时／原共同限额。d 轮原件首次搜索读取超时、完成搜索及目标请求 0，未到达观察保存，峰值 724,553,728 B、34 采样／触限 0，两 JAR 不变、两容器清理并独立剩余 0。不能将新增保存代码的纯测试或未到达的保存点说成实际六例通过。

e 轮只给历史容器添加 `DEBUG=pw:api`，直接执行固定镜像原入口，不包装 Playwright 方法或修改源码。真实 188 B stderr 顺序为 **browserType.launch started → succeeded → browser.newContext started**，没有该 newContext 的 succeeded／failed；原件首次搜索响应仍超时、目标 0。因此卡住位置已从 launch 之后缩小到 newContext，但没有唯一根因，不能以更多日志宣称修复。峰值 543,961,088 B、触限 0、两 JAR 不变、两自有容器清理／独立剩余 0。[原日志哈希与严格范围](evidence/webview-api-context-2026-10-07.json)。

f 轮只增加原生 `pw:browser` 日志和精确自有 cgroup／进程状态观测，不读 argv、环境、用户文件或真实账号。它停在 **原件 readiness／getSystemInfo 前置等待**：Java 已执行，有 Starting args 行，但没有证明启动完成；没有浏览器 API 调用、搜索或目标请求，不是“原 JAR 无法启动”的证明。峰值 259,117,056 B、触限 0、两容器清理／独立剩余 0。probe 退出后、清理前 runtime 只剩两进程／两线程，历史服务一进程／七线程；该次两个 leaf pids.max 事件均 0。**这是 f 的非浏览器样本，Java 此时已退出，不能用它解释或排除 e 的 newContext 原因。**

SSH 间歇 banner／连接错误没有被当成 unit 终止，也没有重复启动 e／f；恢复后核对 f MainPID 0、终态 exit 1 及独立 ownership 剩余 0。通过单个只读复制会话取回自有生成 stage 的小报告，再核对字节哈希；不复制真实数据、JAR 新副本或生产文件。a／b／d 首次搜索、c 历史字段拒绝、e newContext 定位、f readiness 前置分别保留，不能合并原因或拼接成三方成功。
