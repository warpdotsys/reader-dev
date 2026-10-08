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

当前新条件实站未执行；只发一次不同条件请求，不自动重试。即使通过，站点状态和时间也是变量，仍不能唯一归因 UA、证明真实登录、原件／旧远程三方或已部署。失败同样完整保留，不能靠已通过的普通构建抵消。

回退只移除这项 UA 输入、CLI flag 和请求选项／相关测试，恢复默认对照不需要改产品、用户配置或数据。no-wait details 的显式结构守卫应保留；原失败和原 JAR不删除。最新 JAR 的[位级复建结果](JAR-REPRODUCIBILITY-GATE-2026-10-08.md#修正后的真实重建已通过范围仍有限)仍仅证明其自身 runner 范围。
