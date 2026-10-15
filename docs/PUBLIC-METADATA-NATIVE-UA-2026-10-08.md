# 匿名详情的单变量 User-Agent 观察

本增量是有界排查入口，不是站点修复或认证通过。默认产品和默认探针的 HTTP 请求不变，不修改 Java/Kotlin 业务类、worker、全局代理、真实用户数据或生产服务。

## 已验证与源码推断分开

旧结构专项 `37722140916` 已实际失败：原 Native `37718718336`／runtime `7e4b9e509822473c6d98d17c8bb0b922fe28a406`／JAR `61fdc23d71c8ad0c68e120500223562f94186190d595fc6ec172657c9dcf2dde`。快照非空、head 有 script、body 无子元素，元数据为空；不能唯一认定验证码或页面脚本失败。[完整原失败](PUBLIC-METADATA-SNAPSHOT-STRUCTURE-2026-10-07.md#2026-10-08实际结构观察仍是业务红灯)。

已从当前源码确认：`BaseSource.getHeaderMap` 注入 `AppConst.userAgent` 的旧 Windows Chrome 75 字符串；`AnalyzeUrl.UrlOption.headers` 会覆盖同名项，空字符串不会触发 null 默认值；Camoufox renderer 将 UA 从普通 headers 分离至 `userAgent`，worker 仅在其非空时给 NewContext 覆盖值。因此**源码推断**一次请求带固定空 UA 会使用现有浏览器默认值。没有实际采集 HTTP UA 或 navigator UA，也未证明它是空元数据的原因；报告明确 `browserUserAgentActuallyObserved=false`。

## 行为边界

新增显式 `--browser-native-user-agent`／托管 `public_metadata_user_agent=browser-native`；默认仍 `reader-default`。只在这次固定公开书籍请求的 URL options 中加入 `headers: {"User-Agent":""}`，不是伪造 Chrome/Firefox 字符串、安装扩展、改 navigator、重放请求或自动操作验证码。可以与原四种 capture 模式组合；实际选用 `snapshot-only-details`，无额外源等待脚本，网站自身 JS 未被禁用。

Shell 与 workflow 都在加载归档或容器前拒绝非固定枚举；参数只经引号保护的环境变量传入，不做表达式执行。原三个／四个 wrapper 参数调用仍兼容；第五参数默认为 `reader-default`。并发组包含 UA 模式，互不取消不同条件的观察。默认规则散列、固定 URL、源保存读回、一次详情、完整元数据、8＋9 有限布尔值／1 KiB 上限、Cookie 清除／退出、2 CPU／2 GiB／256 PID／零 swap 和私网拒绝全部保留。无真实 Cookie、账号或章节正文。

同步把 no-wait details 的最终结构检查显式绑定 `details_requested`，不再只检查旧 `args.snapshot_details`。原解析器通常将 page/structure 一起拒绝；新增模拟缺结构但有效 page summary 的守卫测试，证明最终门禁也不能默许缺结构。这不是已观察到的实站漏洞或业务修复，不放宽旧检查。

新入口缺失时，三项实际新增用例在旧 CLI／调用接口出现 errors（55 项／3 errors）；早期 helper 名称错误及静态测试误匹配其他 job 也已修正，不把测试脚手架错误冒充产品故障。最终 59 项在 Windows／WSL 全执行通过；WSL JS 仍经既有固定参数桥调用 Windows Node，不冒充原生 Linux Node或浏览器。Windows 全套 358 项：357 通过／1 POSIX 跳过／19.703 秒；现有 Node 发布及源脚本语言契约 119 项无跳过通过，Shell／YAML和发布静态守卫通过。

## 运行身份与复现

为只改变请求条件，将继续消费上述原 `37718718336` Native 归档，不换成新 JAR；探针提交另记。新的 Native 构建和匿名原归档的结果分别核对，不混用身份。

```sh
gh workflow run browser-image.yml --repo warpdotsys/reader-dev --ref ci/full-reader-20260926 \
  -f native_arch=amd64 -f public_metadata_native_run=37718718336 \
  -f public_metadata_revision=7e4b9e509822473c6d98d17c8bb0b922fe28a406 \
  -f public_metadata_capture=snapshot-only-details \
  -f public_metadata_user_agent=browser-native
```

## 新条件已实际执行，业务仍失败

提交 `ce12b21bd7ca521eb918608896da7b6a7c42979d` 的[单次作业 37728643627](https://github.com/warpdotsys/reader-dev/actions/runs/37728643627)实际失败；只执行了一个匿名详情 job，六个互斥 job 跳过不计通过。继续使用原 Native `37718718336`／runtime `7e4b9e509822473c6d98d17c8bb0b922fe28a406`，不是新构建或原件三方。

实际 5.85 秒／HTTP 200／`isSuccess=true`／空 errorMsg，但书名、作者、封面全空。八项页面布尔值均 false，九项结构和上一默认 UA 对照逐项相同：快照非空、head 有 script、body 无子元素、没有 title／iframe。**这是有限快照结构，不是上游原始 HTML、可见验证码或 JS 错误的证明**。`browserNativeUserAgentRequested=true`、`browserUserAgentActuallyObserved=false` 保留，不能称已采集真实 UA。

九份小 JSON、入场／加载／实际 JAR 身份、生成源读回、Cookie 0→0、生成会话撤销、2 CPU／2 GiB／256 PID／零 swap 和有限 worker 诊断均已独立接受；内存峰值 812,363,776 B／PID 183，触限及 OOM 0。worker 固定错误记录为空只说明该类别没有记录，不说明网页 JS／网络全部正常。自有容器已移除为报告值，没有独立宿主残留容器计数。本机仅下载小报告，未另下载／复算原大归档。

[独立机器证据](evidence/public-metadata-native-ua-hosted-ce12b21b-2026-10-08.json)记录完整有限结果和九文件散列；本机独立回执 SHA-256 `bc147be863a33c3ce8cacb221ecad0c5dd9ae9ba77a282590088b67c72b0c69b`。业务红灯保留，不自动重试、不改 worker 或安全门禁，不导入真实凭据、不请求正文、不操作验证码／生产。网站状态和时间也是变量，既未证明 UA 是唯一原因，也未证明它完全无影响。

该提交自己的四条普通托管工作流及 Native 六实际作业成功，自己的小报告已逐项核对，不能抵消上述红灯或借给后续代码。Java CI 两次 JAR 字节相同仅是同一 runner 范围；Full 与 Native 的 JAR 仍不同。下一项源码增量排除已观察到的本机 Python 缓存污染，结果须另验。

回退只移除这项 UA 输入、CLI flag 和请求选项／相关测试，恢复默认对照不需要改产品、用户配置或数据。no-wait details 的显式结构守卫应保留；原失败和原 JAR不删除。最新 JAR 的[位级复建结果](JAR-REPRODUCIBILITY-GATE-2026-10-08.md#修正后的真实重建已通过范围仍有限)仍仅证明其自身 runner 范围。
