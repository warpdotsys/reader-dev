# Kotlin 原版 vs Rust 转录 差异清单（20 个 agent 排查结果汇总）

> 目标：与 Kotlin 原版**包含细节的完全一致重新实现**。每个问题必须完全解决后才能从本文件删除。包括"待确认"问题——不确定的也要验证并修复到一致。
> 状态标记：`[未修]`（待修复）/ `[修复中]` / `[已修]`（修复并验证后删除条目）。已修复验证的条目直接从本文件删除。

## 修复进度汇总（v6.0.20 已包含）

**已修复并验证**（e2e 42/42 + secure 7/7 + cargo test 全绿）：
- P0：16.4/4.1/6.2/6.3/7.1/12.1（handler panic 兜底 500）、16.3（body 10MB）、16.2（原始字节）、10.1（路径穿越 normalize/starts_with 真实实现）、10.2/6.7（READER_APP_SECURE 认证）
- 网络层：15.1（form body+Content-Type）、15.2（单次字符集解码）、15.3（二进制原始字节）、15.4（Set-Cookie 大小写/多值）、15.5（URL 编码真实）、15.6（网络错误异常传播）、15.7（postJson Content-Type）、15.8（socks4 修正）、15.10（read timeout 15s）
- 1.7/7.2（PDF 闭区间+页序）、7.3/18.1/18.2/18.3（GBK 解码链+utf8ToGbk）、11.1/11.2/11.3（封面 clone 引用/UA+Referer/redirectUrl 基准）、3.1/3.2/3.8（lastIndex 读回+SSE isEnd）、19.2（Any::downcast_ref 真实实现）、17.1-17.4（定时任务 await/去重/守护）、16.5（路由链 next() 执行）、16.1/10.3（headers_end 真实执行）、16.6/16.7（multipart 文件名+表单字段）、6.1（logout token_map）、5.1（负数 index 保护）、3.3/3.4（{{bookName}} 自引用+@put: 变量）、1.1/1.6/2.1/2.3/9.2（header map JSON 解析+@js 求值+登录头）、1.4（POST 顶层 chapterUrl）、2.2（分页 ruleData）、9.1/9.5/9.7（RSS baseUrl/key/ruleData 参数）、8B.1/8B.2/19.1（@js: 贪婪+@result 对齐 Kotlin）、8B.3/8B.5/8B.8（JS cookie/cache 对象绑定+Set-Cookie 写回）、8B.4（JsExtensions 补齐约 28 个方法）、13.1（text 规范化）、13.2（result Elements 数组）、13.4（中间伪类 tail）、13.5（id.xxx 前置）、13.6（@html strip script/style）、13.7/13.8（ownText/textNodes 直接子文本）、13.10（:contains 大小写）、13.11（Entities.unescape 全表）
- **生产崩溃（malloc_consolidate ABRT）三个根因**：RoutingContext::get_user ptr::read 双 free、coroutine_handler 悬垂闭包 transmute、YueduApi self_ref 悬垂
- **Release 包 web 目录**：web/ 与 src/main/resources/web 双路径
- **normalize 盘符 bug**（C:C:/... 双重盘符）——WebDAV 400/404 修复

**剩余待处理**（P1 低危 + P2）：
- 1.3（useWebView）、1.8/5.5（chapterUrl 匹配块——Rust 增强，需决策）、2.5/3.12（分页 redirectUrl/JS_PATTERN 决策项）、4.5（SourceAnalyzer login 字段——Rust 增强，需决策）、13.3（:has 条件剥离——已部分改善保留 tail）、13.9/13.12-13.15（低危）、14.1-14.10（缓存落盘/多用户隔离）、16.8（SSE 流式）、16.10（session 续期）、16.11-16.15（CORS/405/静态头/bodyAsJson/多值）、18.4-18.12（CacheManager/临时文件/hashCode/getBaseUrl panic/zip 注释）、19.3-19.7、20.1-20.6（前端 bug）、序列化字段 userNameSpace 等 P2 打磨

---

## 修复进度汇总（v6.0.23 已部署验证）

**v6.0.23 生产验证（192.168.18.75:4396）全部通过**：
- 登录（secure 模式）✓、getUserInfo ✓（堆损坏双 free 修复——不再崩溃）、书架 169 本 ✓、正文（诡秘之主 567ms）✓、封面 200 ✓、书源 429 ✓、WebDAV PROPFIND 207 + DAV/MS-Author-Via/Allow 头 ✓、换源 ✓
- 本地回归：front_flow 42/42 + secure_mode 7/7 + cargo test 全绿

**v6.0.20-23 新增修复**（在之前汇总基础上）：
- 生产崩溃三根因（get_user ptr::read 双 free / coroutine_handler 悬垂闭包 / self_ref 悬垂）——v6.0.20
- Release web 目录双路径——v6.0.20
- normalize 盘符 bug——v6.0.20
- getResultLast 裸 CSS 规则支持（保留增强）——v6.0.21
- useWebView 对齐 Kotlin 硬失败——v6.0.21
- 失效书源缓存落盘恢复、session 续期、序列化 insert_opt 省略 null、bookSourceExploreList 序列化、checker 缺 key、密码字符数、缓存章节解析报错、JS 失败抛错、getUserList 保序、preserve_order——v6.0.21/22
- 前端：滚动模式 __API_ROOT__ 全局替换 / cacheBookSSE token / HttpTTS loadHttpTTS / renderEpub 兼容 /assets——v6.0.22
- 非书架章节缓存落盘+用户隔离——v6.0.22
- headers-end 回调先执行再读响应头（DAV 头注入）——v6.0.23

**v6.0.24 新增修复**（书架刷新并发16/send_file Content-Type+404/IDN 字节切片/bodyAsJson 校验/HttpTTS deleteMulti/users 全局锁/宽松相对URL/checkNotEmpty/时间字段默认/CORS/405/静态缓存头/isSuccessful/header 大小写/limitConcurrent 空批/zip 内存读取/text.xxx/:has 伪类/定时任务丢轮/重定向上限 20/exploreBook 校验）——本地全绿（42/42 + 7/7 + unit）

**已知限制（架构/边缘，标记保留）**：
- SSE 流式：单线程 tokio 架构下缓冲发送（text/event-stream 已设，功能可用，渐进显示降级）
- gb18030 4 字节生僻字：encoding_rs GBK 覆盖 2 字节（罕见）
- 跨域重定向 reqwest 剥离 cookie/Authorization（okhttp 保留；罕见）
- 查询参数多值 last-wins（现有接口单值）
- 定时任务串行执行（Kotlin IO 池并发——Rust 串行等效结果，更安全）
- 决策项（Rust 增强保留）：chapterUrl 匹配块/分页 redirectUrl/SourceAnalyzer login 字段/PDF 文本渲染/裸 CSS 规则

---

# 详细差异清单

---

## 1. 正文链差异（agent 1）

### 1.1 【功能 bug】多页请求头：Rust 把整个 header JSON 当单个 "header" 头发送
- Kotlin: `BookContent.kt:64-70` → `BaseSource.kt:51-71`（UA + 解析 header JSON 成独立头）
- Rust: `BookContent.rs:73-86` → `stubs.rs:8740-8752`（占位：`hm.insert("header", h)`，原始 JSON 字符串原样塞进名为 "header" 的单个头）
- 影响：分页正文（nextContentUrl）场景下，依赖自定义头的书源下一页请求头错误
- 验证：header 非空书源 + 分页章节，抓包对比请求头
- 状态：[未修]

### 1.2 【功能差异】AnalyzeByJSoup.getResultLast 的 else 分支：Rust 改成"CSS 选择器优先"
- Kotlin: `AnalyzeByJSoup.kt:249-256`（else 只读 `element.attr(lastRule)`）
- Rust: `AnalyzeByJSoup.rs:351-369`（先 `element.select(last_rule)`，命中返回元素 text，未命中才 attr）
- 影响：`@title`（内容含 `<title>` 时）、`@img`、`@style` 等规则结果不同
- 验证：构造末段规则为 title/img 的书源对比
- 状态：[未修]

### 1.3 【待确认】useWebView 书源：Kotlin 硬失败 vs Rust 静默降级且丢弃 sourceRegex/webJs
- Kotlin: `AnalyzeUrl.kt:355-379` + `DefaultAdpater.kt:37-52`（webview 适配器 throw Exception）
- Rust: `AnalyzeUrl.rs:610-642` + `ReaderAdapterHelper.rs:96-100`（适配器恒 None → 回退 js_http_request，不执行 webJs、不应用 sourceRegex；POST 用带 query 的完整 url）
- 影响：useWebView=true 书源 Kotlin 报错、Rust 返回未渲染 HTML（静默错内容）
- 验证：useWebView+webJs 书源对比
- 状态：[未修]

### 1.4 【功能 bug】POST 顶层 "chapterUrl" 字段支持缺失
- Kotlin: `BookController.kt:523`（`bodyAsJson.getString("chapterUrl") ?: bookChapter.url`）
- Rust: `BookController.rs:765-769`（只读 `bookChapter.url`）
- 影响：任何直接 POST 顶层 chapterUrl 的客户端（如原版 Android 端）拿到空 chapterUrl
- 验证：POST /getBookContent body `{url, chapterUrl}` 对比
- 状态：[未修]

### 1.5 【待确认】分页边界检测：getAbsoluteURL 基准 URL 不同（base_url vs redirectUrl）
- Kotlin: `BookContent.kt:58-61`（`getAbsoluteURL(redirectUrl, nextUrl)`）
- Rust: `BookContent.rs:64-71`（`get_absolute_url(URL::parse(base_url))`）
- 影响：首页重定向时 "下一页 == 下一章" 判定可能误判
- 验证：首页 301/302 + 多页正文书源
- 状态：[未修]

### 1.6 【待确认】AnalyzeUrl::new 的 header fallback 缺 @js:/<js> 求值与登录头合并
- Kotlin: `AnalyzeUrl.kt:72-82` → `BaseSource.kt:51-71`（含 @js: 求值 + hasLoginHeader 合并）
- Rust: `AnalyzeUrl.rs:180-202`（手工 fallback：UA + serde 解析，无 @js: 求值、无登录头；@js: header 解析失败塞进 "header" 头）
- 影响：WebBook 全部调用传 None 走此 fallback；JS 计算请求头/登录头的书源头错误
- 验证：header 以 @js: 开头的书源对比
- 状态：[未修]

### 1.7 【PDF 正文】区间 exclusive + 页向量乱序索引
- Kotlin: `BookController.kt:665-684`（`for (page in start..end)` 含端）
- Rust: `BookController.rs:993-1022`（`for page in start..end` 不含端 → 末页丢失；`pages` 由 HashMap 迭代构建后 1 基索引 → 页错位/越界）
- 验证：3 页以上 PDF 逐页比对
- 状态：[未修]

### 1.8 【增强型差异】Rust 新增"按 chapterUrl 匹配章节"块（Kotlin 无）
- Kotlin: 无（web 书只发 chapterUrl 时报"获取章节链接失败"）
- Rust: `BookController.rs:856-879`（chapter_info 空且 chapterUrl 非空时拉目录按 url 匹配）
- 判断：属于 Rust 端修复而非回归；为保持一致需评估是否保留（目标：完全一致 → 需要决策）
- 状态：[未修]

### 1.9 【JS 失败语义】Kotlin 抛异常 vs Rust 静默 None
- Kotlin: `AnalyzeUrl.kt:240-254`（SCRIPT_ENGINE.eval 抛 ScriptException → 请求失败）
- Rust: `AnalyzeUrl.rs:440-477`（eval_downcast_any 返回 None → URL 中 <js>/@js: 段被替换成空串）；`AnalyzeUrl.rs:445-446` cookie/cache 绑定为占位字符串
- 影响：JS 语法错误时 Kotlin 报错、Rust 产出截断 URL/空值
- 状态：[未修]

### 1.10 【无害】loginCheckJs eval 结果处理不同
- Kotlin: `WebBook.kt:238`（`as StrResponse` 强制 cast，非字符串 ClassCastException）
- Rust: `WebBook.rs:254-262`（仅 as_string 时重建，否则保留）
- 判断：无害（Rust 更宽容）——为完全一致需评估
- 状态：[未修]

### 1.11 【无害】get_book_content 中规则来源混用 prepared_source / book_source（prepare_source 只是 clone，等价）
- 状态：[未修]

### 1.12 【微差异】getAbsoluteURL(String) 重载的逗号截断
- Kotlin: `NetworkUtils.kt:82`（`baseURL.substringBefore(",")`）
- Rust: `NetworkUtils.rs:90`（splitn(2,",") 等价）；但 `BookContent.rs:66-67` 走 `URL::parse(base_url)` 无逗号截断
- 判断：极端场景
- 状态：[未修]

---

## 2. 章节列表链差异（agent 2）

### 2.1 【功能 bug】主目录网络请求丢失登录头（getHeaderMap(true) → None）
- Kotlin: `WebBook.kt:226-233`（`getHeaderMap(true)`：UA + header JSON + loginHeader 缓存）
- Rust: `WebBook.rs:318-330`（header_map_f = None → 回退分支只有 UA + header JSON，无登录头）
- 影响：已登录书源目录页请求丢失登录头
- 状态：[未修]

### 2.2 【功能 bug】分页目录请求丢失 ruleData → book 变量失效 + Cookie 命名空间变 "unknow"
- Kotlin: `BookChapterList.kt:61-67,84-90`（AnalyzeUrl ruleData=book）
- Rust: `BookChapterList.rs:71-72,96-97` → `analyze_url_new_placeholder`（`stubs.rs:8526-8544`，rule_data=None、debug_log=None）
- 影响：分页 URL 含 {{bookName}} 时失败；cookie 存到 "unknow" 命名空间 → 分页登录态断裂
- 状态：[未修]

### 2.3 【功能 bug】分页请求 header 字段 JSON 不解析（同 1.1，Rust 端 `stubs.rs:8740-8752`）
- Kotlin: `BookChapterList.kt:65` getHeaderMap() 解析 JSON
- Rust: `BookChapterList.rs:71,96` → 占位塞 "header" 键
- 状态：[未修]

### 2.4 【高影响】本地书错误分支 panic! 完全无保护
- Kotlin: `BookController.kt:1741-1749` throw Exception → vert.x failure handler → 500
- Rust: `BookController.rs:2378-2387` panic!（在 catch_unwind 之外）、`:2438` 再次 panic、`LocalBook.rs:66` panic、`VertExt.rs:373` panic
- 影响：epub 解压失败/目录为空崩溃整个 HTTP 服务器（配合基础设施层 panic 无兜底，见 16.4）
- 状态：[未修]

### 2.5 【待确认】分页循环分支 redirectUrl 用 res.url() 而非请求 URL
- Kotlin: `BookChapterList.kt:68-71`（循环分支 baseUrl=redirectUrl=nextUrl；并发分支 redirectUrl=res.url）
- Rust: `BookChapterList.rs:75-78`（循环分支也 redirectUrl=&res_url——对齐了 Kotlin 并发分支）
- 判断：Rust 是"改进"还是"偏差"取决于原版意图（目标完全一致：需决策）
- 状态：[未修]

### 2.6 【功能差异】缓存章节解析失败：Rust 静默空章节 vs Kotlin 整请求报错
- Kotlin: `BookController.kt:1794-1797`（mapTo 失败抛异常 → 500）
- Rust: `BookController.rs:2462-2466`（`unwrap_or_default()` 返回全默认 BookChapter 加入列表）
- 影响：书架缓存损坏时前端目录出现空白条目
- 状态：[未修]

### 2.7 【无害】响应 JSON 多出 "userNameSpace" 字段（json_conv.rs:295；Kotlin @JsonIgnoreProperties）
- 状态：[未修]（为一致需移除）

### 2.8 【待确认·低】JS NativeObject 分支 Debug 格式化代替 toString（AnalyzeRule.rs:262-264；当前 JS 引擎占位不可达）
- 状态：[未修]

### 2.9 【无害】GET refresh 参数解析容错（Rust to_int().unwrap_or(0) vs Kotlin toInt() 抛异常）
- 状态：[未修]（Rust 更健壮，为一致需决策）

### 2.10 【无害】getChapterListByRule body 解析失败处理不同（unwrap_or_default vs 抛异常）
- 状态：[未修]

### 2.11 【说明】前端组件是 PopCatalog.vue 不是 ChapterList.vue（无差异）
- 状态：无需修复

### 2.12 【无害】POST body 缺 "book" 对象时 Kotlin NPE vs Rust 安全返回（Rust 更健壮）
- 状态：[未修]

---

## 3. 搜索链差异（agent 3）

### 3.1 【功能 bug·高】searchBookMulti 响应返回的 lastIndex 永远是请求时的旧值（分页失效）
- Kotlin: `BookController.kt:851`（闭包内 `lastIndex = Math.max(lastIndex, it)`）+ `:891`（返回更新后值）
- Rust: `BookController.rs:1240`（拷贝到 cell）、`:1246`（只更新 cell）、`:1295`（返回外层旧值——i32 Copy 不随 cell 更新）
- 佐证：同文件 SSE 版 `BookController.rs:1452` 正确读回 cell
- 影响：前端"加载更多"重复同一批书源 → 新结果被去重为空 → 误报"没有更多啦"
- 状态：[未修]

### 3.2 【功能 bug·高】searchBookSource 非 SSE 版同样返回旧 lastIndex
- Kotlin: `BookController.kt:1074,1109`（返回更新后）
- Rust: `BookController.rs:1534`（只更新 cell）+ `:1576`（返回旧值）
- 影响：换源页"加载更多书籍来源"反复拉同一批源（SSE 路径正确 rs:1732）
- 状态：[未修]

### 3.3 【功能 bug·中高】搜索条目规则中 {{bookName}}/@get:{bookName} 自引用在 Rust 解析为空
- Kotlin: `BookList.kt:164`（ruleData = searchBook 本体；bookUrl 在 :220 解析，能取到书名）
- Rust: `BookList.rs:267-271`（rule_data 设为只含 variable+userNameSpace 的快照副本）；`AnalyzeRule.rs:79-83` book() 只 downcast_ref::<Book>()（SearchBook 与 Book 无关）恒 None；get()/eval_js 的 book 绑定只能从 book_variables 重建
- 影响：bookUrl 含 {{bookName}} 的书源链接为空
- 状态：[未修]

### 3.4 【功能 bug·中】搜索条目中的 @put: 变量不写入返回的 SearchBook（不持久化）
- Kotlin: `AnalyzeRule.kt:625-630`（put 写入 ruleData=searchBook 本体 → SearchBook.variable）
- Rust: `AnalyzeRule.rs:488-497`（写入 self.book_variables + rule_data 快照副本）；`BookList.rs:260-271` 返回的 search_book.variable 仍 None
- 影响：依赖 @put: 传递变量到详情页/下一请求的书源失效
- 状态：[未修]

### 3.5 【错误分支】get_search_item 逐字段 try/catch 被注释掉：单字段失败升级为整源失败
- Kotlin: `BookList.kt:177-217`（kind/wordCount/latestChapterTitle/intro/coverUrl 各自 try/catch）
- Rust: `BookList.rs:304-382`（try/catch 全部注释 → panic 冒泡 → 整源标记失败丢失）
- 状态：[未修]

### 3.6 【待确认·中】loginCheckJs 执行结果处理不同（Kotlin 无条件强转 as StrResponse + result 绑定整个响应；Rust 仅字符串重建 + result 只传 res.body()）
- Kotlin: `WebBook.kt:82-85` vs Rust: `WebBook.rs:126-137`
- 影响：依赖 result.url 等属性的 login JS 在 Rust 拿不到
- 状态：[未修]

### 3.7 【待确认·低】AnalyzeUrl 的 analyze_js：Rust 缺少 @result 替换、@js: 段结束符处理不同（同 19.1）
- Kotlin: `AnalyzeUrl.kt:106-124`（前缀 @result 替换；@js: 贪婪吃串尾）
- Rust: `AnalyzeUrl.rs:242-280`（手工扫描：前缀不替换 @result；@js: 吃到下一个 @）
- 状态：[未修]

### 3.8 【待确认·低】searchBookSourceSSE isEnd 计算：Rust 用未收缩的 max_size
- Kotlin: `BookController.kt:1201`（maxSize 收缩）+ `:1232`（isEnd 用收缩后）
- Rust: `BookController.rs:1695`（收缩 cell）+ `:1736`（用外层原值）
- 影响：提前终止时 isEnd 恒 false（前端 end 处理器不读 isEnd，影响小，但需一致）
- 状态：[未修]

### 3.9 【无差异】SearchBook 字段映射已核对一致（coverUrl/originName/latestChapterTitle/intro/time/origin/type/bookUrl）
- 状态：无需修复

### 3.10 【待确认·低】limit_concurrent_with 与 Kotlin limitConcurrent 循环语义差异（空结果批次是否触发 needContinue；线程 panic 静默丢弃）
- Kotlin: `BaseController.kt:319-391` vs Rust: `BookController.rs:71-105`
- 状态：[未修]

### 3.11 【无害】searchBookWithSource 非精确过滤 null 作者处理（Rust 不抛错不标记失败——Rust 更健壮）
- 状态：[未修]（为一致需决策）

### 3.12 【待确认·低】Kotlin 原版分页/JS 模式疑似失效（Rust 重写修复）——需实测确认哪边行为真实
- Rust `AnalyzeUrl.rs:243,317` 注释："原 JS_PATTERN 匹配恒空"、"原 pagePattern 匹配恒空"
- 状态：[未修]

---

## 4. 书源管理链差异（agent 4）

### 4.1 【功能 bug·高】Rust 控制器 panic 无 HTTP 兜底 → 请求挂起；Kotlin 有 JSON 错误响应
- Kotlin: `RestVerticle.kt:145-160` coroutineHandler 捕获异常 → onHandlerError → 500 JSON
- Rust: `stubs.rs:7799-7810` coroutine_handler 直接执行无 catch_unwind；`runtime/server.rs:395-445` dispatch 无兜底
- 触发点：`VertExt.rs:824` parse_json_string_list panic、`VertExt.rs:356` 文件锁超时 panic、`BookSourceController.rs:386/445/675/756` body_as_json().unwrap()、`BookSourceController.rs:853` panic!
- 状态：[未修]

### 4.2 【功能 bug·中高】saveFromRemoteSource 同步阻塞 → 单线程运行时全站冻结
- Kotlin: `BookSourceController.kt:470-479`（launch(Dispatchers.IO) 异步）
- Rust: `BookSourceController.rs:887-914`（同步 async_get_text_in_thread 最多 3s；单线程 current_thread runtime 阻塞全站）
- 状态：[未修]

### 4.3 【待确认·中】bookSourceExploreList 被序列化为 null
- Kotlin: `BookSourceController.kt:503`（arrayListOf<Map> → GSON 数组）
- Rust: `BookSourceController.rs:962`（Vec<HashMap<String,Option<String>>> 无 any_to_json_value 分支 → Value::Null 落盘）
- 当前无读取方（死数据），但为完全一致需修
- 状态：[未修]

### 4.4 【待确认·中】Rust 持久化 JSON 与 Kotlin 格式差异（多 userNameSpace 字段 + 显式 null）
- Kotlin: `BookSource.kt:27-30` @JsonIgnoreProperties 排除 _userNameSpace；NON_NULL 省略 null
- Rust: `json_conv.rs:101-141` 显式输出 `"userNameSpace":""` + `map_opt(None)` 显式 null（bookSourceGroup/header/ruleSearch 等 null）
- 状态：[未修]

### 4.5 【待确认·低中】SourceAnalyzer 旧格式书源导入差异（Rust 恢复 login_url/login_ui/login_check_js 读取——标注"fix"，与 Kotlin 不一致但方向增强）
- 状态：[未修]（为完全一致需决策：目标完全一致 → 应回退到 Kotlin 行为）

### 4.6 【无害】parseJsonStringList checkNotEmpty：字段缺失时 Kotlin 缺键 vs Rust 显式 false（exploreUrl 为 null 字符串时行为不同）
- 状态：[未修]（为一致需修）

### 4.7 【无害】getBookSources GET simple 参数解析健壮性（Rust 更健壮）
- 状态：[未修]

### 4.8 【无害】deleteBookSources 单条解析失败：Kotlin 500 vs Rust 静默跳过
- 状态：[未修]

### 4.9 【无害】getBookSourceMap 数值解析（等价）
- 状态：无需修复

### 4.10 【无害】readSourceFile 返回类型（等价）
- 状态：无需修复

### 4.11 【无害】updateRemoteSourceSub 异常语义（panic 被 job 层捕获，等价）
- 状态：无需修复

### 4.12 【待确认·低】BookSource 手工 serde 反序列化默认值（respondTime=0 vs 180000L、enabledCookieJar=None vs false；仅用于 URL 比较路径，无实际影响但语义偏差）
- 状态：[未修]

---

## 5. 书架/进度/书签链差异（agent 5）

### 5.1 【功能 bug】saveBookProgress 负数 index：Rust 静默覆盖阅读进度，Kotlin 报错
- Kotlin: `BookController.kt:468-471`（-1 抛 IndexOutOfBoundsException → 500，进度不变）
- Rust: `BookController.rs:689-697`（`-1 as usize` 溢出 → unwrap_or_default 得到默认 BookChapter → 覆盖 durChapterIndex=0、清空标题）
- 验证：POST {url, index:-1} 对比
- 状态：[未修]

### 5.2 【功能差异】getBookContent POST 丢失顶层 chapterUrl 参数（同 1.4）
- 状态：[未修]

### 5.3 【待确认·跨用户】失效书源缓存按用户隔离丢失：Rust 静态共享
- Kotlin: `BookController.kt:128-133,160-168`（每用户目录 + 读取按用户 listFiles）
- Rust: `BookController.rs:224-252`（静态 OnceLock + 落盘路径无 userNameSpace）
- 状态：[未修]

### 5.4 【待确认·跨用户】章节缓存静态共享，忽略 userNameSpace
- Kotlin: `BookController.kt:144-147`（按用户建目录）
- Rust: `BookController.rs:254-258`（get_book_chapters_cache 忽略参数）
- 状态：[未修]

### 5.5 【无害·增强】getBookContent 新增 chapterUrl 兜底匹配分支（同 1.8）
- 状态：[未修]

### 5.6 【无害】Book 序列化多 4 字段（infoHtml/tocHtml/rootDir/userNameSpace；json_conv.rs:90-93 vs Book.kt:22 @JsonIgnoreProperties）
- 状态：[未修]

### 5.7 【无害·性能】getBookshelf 刷新：Kotlin 并发 16 vs Rust 串行
- Kotlin: `BookController.kt:1687-1720` vs Rust: `BookController.rs:2322-2351`
- 状态：[未修]

### 5.8 【无害】缺失时间字段反序列化默认值不同：Rust=0 vs Kotlin=now（latestChapterTime/lastCheckTime 显示 1970）
- Kotlin: `Book.kt:40-41` vs Rust: `Book.rs:495-496`
- 状态：[未修]

### 5.9 【无害】getBookInfo POST 缺 searchBook 时 Kotlin NPE（Rust 已修复——Rust 更健壮）
- 状态：[未修]（为一致需决策）

### 5.10 【无害】book_info_cache 容量与过期语义不同（Kotlin ACache 2MB/10000 上限 LRU vs Rust 无限 HashMap 永不过期）
- Kotlin: `BookController.kt:119` vs Rust: `BookController.rs:195-207`
- 状态：[未修]

### 5.11 【待确认】bookshelf.json 读后写回的自洽性依赖 JsonArray 内部字符串存储设计（当前全部走 to_string 路径自洽；脆弱设计非现有 bug）
- 状态：[未修]（验证：加书→getBookshelf→saveBookProgress→getBookshelf 字段逐键不变）

### 5.12 【无害】getShelfBookWithCacheInfo 返回结构差异（同 5.6 多 4 键）
- 状态：[未修]

---

## 6. 用户/登录/配置链差异（agent 6）

### 6.1 【功能 bug·高】logout 无法清除 token_map → 退出后旧 accessToken 仍可自动登录
- Kotlin: `UserController.kt:195-200`（tokenMapVal as? MutableMap 泛型擦除恒成功，remove 生效）
- Rust: `UserController.rs:379-384`（downcast_ref::<HashMap<String, i64>> 精确匹配失败——实际类型是 HashMap<String, Box<dyn Any>>（stubs.rs:8873-8893）→ 恒 None → 删除被跳过）
- 影响：退出登录后 accessToken 依然有效（check_auth 还会续期）
- 状态：[未修]

### 6.2 【功能 bug·中高】storage 读写/校验失败：Rust panic（无响应/连接重置）vs Kotlin 干净 500 JSON
- Kotlin: `VertExt.kt:189-195,243-250,267-279`（throw → onHandlerError）
- Rust: `VertExt.rs:289,356,373,392`（panic!）；RestVerticle.rs:194-204 coroutine_handler 无 catch；server.rs execute_rules 无 catch_unwind
- 触发：文件锁超时（10s）、.users.key 篡改校验失败、IO 错误
- 状态：[未修]

### 6.3 【功能 bug·中】deleteUsers 对空/非数组 body 直接 .unwrap() panic
- Kotlin: `UserController.kt:368`（bodyAsJsonArray 抛 DecodeException → 500）
- Rust: `UserController.rs:728`（body_as_json_array().unwrap()——空 body 返回 None → panic）
- 状态：[未修]

### 6.4 【待确认·中低】users.json 损坏且 .users.key 缺失时：Kotlin 拒写 vs Rust 静默清空全部用户
- Kotlin: `VertExt.kt:267-279`（JsonObject 构造抛异常 → 500，文件不动）
- Rust: `stubs.rs:1683`（new_parsed 不校验合法性）→ `user_map_nested`（stubs.rs:8854-8871）失败返回空表 → login 视为无用户 → 注册新用户重写 users.json 仅剩新用户
- 状态：[未修]

### 6.5 【待确认·中低】密码长度校验：Kotlin 字符数 vs Rust 字节数（str.len()）
- Kotlin: `UserController.kt:113,258,323`（String.length UTF-16 码元）vs Rust: `UserController.rs:239,529,641`（str.len() UTF-8 字节）
- 例：3 个汉字 = 9 字节，Rust 通过 Kotlin 拒绝
- 状态：[未修]

### 6.6 【待确认·中】logout 与 saveUserSession 使用两个不同互斥锁 → 并发丢失更新
- Kotlin: `BaseController.kt:83`（单一 userMutex）
- Rust: `BaseController.rs:50`（save_user_session 用之）与 `UserController.rs:100`（logout 用之）各自独立 Mutex
- 状态：[未修]

### 6.7 【待确认·中】getUserInfo 的 secure/secureKey 配置源：Spring 配置 vs OS 环境变量（部署差异）
- Kotlin: `UserController.kt:457-458`（Spring Environment）vs Rust: `UserController.rs:909-910` + `stubs.rs:6231-6254`（READER_APP_SECURE/READER_APP_SECUREKEY 环境变量）
- 注意：WebDAV 认证依赖 secure 值（见 10.2）
- 状态：[未修]

### 6.8 【无害】输入健壮性：空 body（Rust 更健壮）、非字符串字段（数字 username Rust 可注册）
- 状态：[未修]

### 6.9 【无害】getUserList 返回顺序：插入序 vs 哈希序（HashMap 无序）
- 状态：[未修]（为一致需改用有序 map）

### 6.10 【无害】写出的 JSON 对象 key 被排序（serde_json 未启用 preserve_order → BTreeMap 字母序）
- 状态：[未修]

### 6.11 【无害】getUserInfo 的 username 读取路径（含兜底，行为一致）
- 状态：无需修复

### 6.12 【无害】死代码 userMaxCount=15（未使用）
- 状态：无需修复

---

## 7. 本地书/文件管理链差异（agent 7）

### 7.1 【功能 bug·严重】本地书解析 panic 会杀死整个 HTTP 服务器
- Kotlin: `LocalBook.kt:56`、`BookController.kt:1741-1750`（异常 → 500，服务存活）
- Rust: `LocalBook.rs:66,49`、`BookController.rs:2378-2388`（panic 无 catch_unwind）；调用点：`BookController.rs:519,638-640,836/870/906,690`
- 执行环境：`server.rs:348-355` execute_rules 直接 f(&mut ctx) 无 panic 拦截；单线程 current_thread runtime 在 thread::spawn 线程——handler panic → 线程死亡 → 整个 HTTP 服务停止响应（main 在 block_forever 无感知）
- 修复方向：handler 外层统一 catch_unwind 兜底（核心）
- 状态：[未修]

### 7.2 【功能 bug·高】PDF 正文整章空白：Rust 区间 start..end 排他，Kotlin 闭区间
- Kotlin: `BookController.kt:673` `for (page in start..end)`（含 end）
- Rust: `BookController.rs:1001` `for page in start..end`（不含 end）
- PDF 章节 start==end（每页一章，toc="page"）：Rust 循环零次 → content=""，所有按页分章的 PDF 正文为空
- 状态：[未修]

### 7.3 【功能 bug·高】txt 章节解析/正文忽略已检测的字符集（GBK 等乱码 + 偏移错乱）
- Kotlin: `TextFile.kt:130,40`（String(buffer, ..., charset) 按 EncodingDetect 结果解码）
- Rust: `TextFile.rs:162,80,104`（String::from_utf8_lossy；book.charset 检测了(:100)却从不使用）
- 后果：GBK txt 目录/正文乱码；lossy 后长度膨胀 → 章节 start/end 偏移漂移
- 同类：`FilesUtil.rs:476-485` readText_charset 忽略 charset
- 状态：[未修]

### 7.4 【功能差异·中】PDF 阅读管线：Kotlin 图片渲染（stub 产出空 png）vs Rust 文本提取（有意修复，Rust 可读）——为完全一致需决策（Kotlin 实际不可用，保持 Rust 文本渲染？）
- 状态：[未修]

### 7.5 【功能差异·低危】importBookPreview 异常语义变宽容（Rust catch_unwind 一切 panic → 空章节列表入库；Kotlin 只 catch TocEmptyException，损坏 epub 500 拒绝导入）
- 状态：[未修]

### 7.6 【无害】download 的 Content-Disposition 文件名未 URL 编码（url_encode_charset no-op 相关，见 15.5）
- 状态：[未修]

### 7.7 【无害】EPUB 正文解码：Rust 按资源声明编码（改进）；但标题回退提取两处固定 from_utf8_lossy（EpubFile.rs:358,394 vs EpubFile.kt:280,313 用 mCharset）——非 UTF-8 epub 目录标题可能乱码
- 状态：[未修]

### 7.8 【无害】resolveSecurePath 前缀剥离 removePrefix vs trim_start_matches（最终都受 starts_with 校验，无穿越）
- 状态：无需修复（但见 10.1 路径穿越——基础校验本身是字符串前缀比较）
- 状态：[未修]（与 10.1 合并处理）

### 7.9 【无害】request_path POST 分支取参差异（等价）
- 状态：无需修复

### 7.10 【待确认】import_book_preview 临时文件名（中文截断 50 字符不追加哈希，同名覆盖——两版一致）
- 状态：无需修复

### 7.11 【一致】getBookContent epub 章节文件路径拼装一致
- 状态：无需修复

### 7.12 【一致】章节缓存文件名/路径一致
- 状态：无需修复

---

## 8. 前端契约 + JS/变量/Cookie/java 扩展（agent 8）

### 8A. 前端 API 契约

### 8A.1 【唯一非规范】App.ts:666 调 GET /getTxtTocRules 不带 api 前缀（依赖 axios baseURL 拼接；若 api_prefix 配置变化会 404）
- 状态：[未修]

### 8A.2 【信息】getSystemInfo/getShelfBook/bookSourceDebugSSE/httpTTS/list/backupToMongodb/restoreFromMongodb 后端注册前端不用（无前端调用缺失）
- 状态：无需修复

### 8B. JS/变量/Cookie/java 扩展

### 8B.1 【严重】URL 末尾裸 `@js:code`（无闭合 @）在 Rust 不执行；Kotlin 正常执行（JS_PATTERN 贪婪吃串尾）
- Kotlin: `AnalyzeUrl.kt:103-124`、`AppPattern.kt:6-7` vs Rust: `AnalyzeUrl.rs:242-280`（要求 @js:...@ 有闭合 @；无闭合时整个段原样留在 URL）
- 影响：所有以 @js: 收尾的 search/detail/toc 规则 URL
- 状态：[未修]

### 8B.2 【严重】JS 段前文本含 `@result` 时替换为 ruleUrl（Kotlin AnalyzeUrl.kt:112,121）——Rust 无此逻辑，输出字面 @result
- 状态：[未修]

### 8B.3 【严重】JS 中 `cookie`/`cache` 绑定对象：Kotlin CookieStore/CacheManager 对象；Rust 绑定字符串/null（cookie.getCookie()/cache.put() 全部 TypeError）
- Kotlin: `AnalyzeRule.kt:653-654`/`AnalyzeUrl.kt:244-245` vs Rust: `AnalyzeRule.rs:571-572`/`AnalyzeUrl.rs:445-446`
- 影响：登录态判断、cookie 读写类书源（大量）
- 状态：[未修]

### 8B.4 【严重】JsExtensions 缺约 34 个方法：connect、webView、downloadFile、utf8ToGbk、readTxtFile、deleteFile、unzipFile、getTxtInFolder、queryBase64TTF、queryTTF、replaceFont、longToast、logType、androidId、digestBase64Str、全部 ByteArray 系 AES/3DES/DES（aesDecodeToByteArray、tripleDESDecodeStr、desEncodeToString 等约 12 个）
- Kotlin: JsExtensions.kt 全表（945 行约 60 方法）vs Rust: runtime/js.rs:778-808（约 26 个）
- 状态：[未修]

### 8B.5 【高】java.get/head/post 返回 Jsoup Response 且自动把 Set-Cookie 写回 cookie jar（Kotlin JsExtensions.kt:208-212）——Rust 返回普通对象不写回
- 影响：先 GET 登录页再取正文的书源后续请求无 cookie
- 状态：[未修]

### 8B.6 【中高】JS 异常静默 vs 抛错（Kotlin evalJS 无 try/catch 异常向上抛；Rust eval 失败静默返回 None → 空内容，无错误反馈）
- 状态：[未修]

### 8B.7 【中】book 绑定：Kotlin 绑定真实 Book/SearchBook；Rust book() 恒 None 时用 @put: 重建子集 JSON，缺 customCoverUrl/latestChapterTime/readConfig 等字段
- 状态：[未修]

### 8B.8 【高】JS 侧 cookie 写入通道：Kotlin `cookie.setCookie(url, cookie)`；Rust 无入口（核心 CookieStore 本体转录忠实，缺 JS 绑定）
- 状态：[未修]

### 8B.9 【中】encodeURI(str, enc) 支持指定字符集（Kotlin JsExtensions.kt:331-337）；Rust 只取 arg0 恒 UTF-8（runtime/js.rs:658-663）
- 状态：[未修]

### 8B.10 【中】JS_PATTERN 正则 vs 手写扫描器（转录作者承认原正则不工作，手写重写引入 8B.1/8B.2 偏差）——需统一到 Kotlin 语义
- 状态：[未修]

### 8B.11 【低】RuleAnalyzer innerRule fr 返回 null 时 Kotlin 拼字面 "null" vs Rust 拼 ""（RuleAnalyzer.rs:467-468）
- 状态：[未修]

### 8B.12 【中】引擎差异：Rhino 1.7.13（ES5+部分 ES6，保留 Packages.* Java 互操作）vs BoA 0.21（ES2021，无 Java 互操作；async 结果 Promise 悬置）
- 影响：用 Packages.javax.crypto 的书源 Rust 全灭；async/await 书源结果不 resolve
- 状态：[未修]

---

## 9. RSS 链差异（agent 9）

### 9.1 【功能 bug】getContent 中 AnalyzeUrl 参数错位：origin 被塞进 key，baseUrl 传空串
- Kotlin: `Rss.kt:40-47`（命名参数 baseUrl=rssArticle.origin）
- Rust: `Rss.rs:60-72`（第 2 位 key=origin，第 6 位 base_url=空串）
- 影响：link 相对/协议相对时（RSS 默认解析器不绝对化 link）正文抓取失败；{{key}}/{{baseUrl}} 绑定错位
- 状态：[未修]

### 9.2 【功能 bug】RssSource::get_header_map 是占位实现，header JSON 未解析（同 1.1/2.3）
- Kotlin: `BaseSource.kt:51-71` vs Rust: `stubs.rs:9643-9653`
- 状态：[未修]

### 9.3 【功能 bug】空 body 时 Rust panic! vs Kotlin 抛异常返回 500 JSON（同 16.4 基础设施）
- Kotlin: `RssParserByRule.kt:25-29` vs Rust: `RssParserByRule.rs:30-32`
- 状态：[未修]

### 9.4 【功能 bug】RssSource::from_json_doc 用 .expect() 无保护 → panic（saveRssSource 缺 sourceUrl 断连；get_rss_source_by_url 遍历坏条目 panic）
- Kotlin: `RssSource.kt:131-132`（runCatching 内） vs Rust: `RssSource.rs:253-254`
- 状态：[未修]

### 9.5 【功能 bug】get_articles 中 source/ruleData 传 None（JS source 绑定丢失、{{put:}} 变量丢失）
- Kotlin: `Rss.kt:20-27` vs Rust: `Rss.rs:30-42`（注释自认占位）
- 影响：sortUrl 用 source.xxx / {{put:}} 的 RSS 源失效；RssParserByRule.rs:80-84 variable 恒 None
- 状态：[未修]

### 9.6 【功能 bug】RssParserByRule 中 AnalyzeRule 的 source 传 None
- Kotlin: `RssParserByRule.kt:38` vs Rust: `RssParserByRule.rs:43`（绑定 source_book_source 而非 source）
- 状态：[未修]

### 9.7 【功能 bug】getContent 的 AnalyzeRule.setBaseUrl 基准 URL 不同
- Kotlin: `Rss.kt:53`（getAbsoluteURL(origin, link)）vs Rust: `Rss.rs:81-82`（get_absolute_url(None, link)）
- 状态：[未修]

### 9.8 【无害】RssArticle 序列化多出 userNameSpace（键名 Kotlin 是 _userNameSpace）
- 状态：[未修]

### 9.9 【无害】getRssArticles 返回 second：null vs 空串
- 状态：[未修]

### 9.10 【无害】GET page 参数解析（Rust 更健壮）
- 状态：[未修]

### 9.11 【待确认】set_cookie 的 tag 来源不同（Kotlin rssSource.getKey() vs Rust source=None 落 self.url；跨域 cookie 差异）
- 状态：[未修]

### 9.12 【待确认】KXmlParser::next_text() 多 END_TAG 检查（RssParserDefault 全部 unwrap_or_default 吞错——需对照 kxml2 行为）
- 状态：[未修]

---

## 10. WebDAV 链差异（agent 10）

### 10.1 【安全·高危】路径穿越：normalize() 为 no-op + starts_with 纯字符串前缀比较 → 可读写删除 home 外任意文件
- Kotlin: `WebdavController.kt:85-90`（java.nio.Path.normalize 真实解析 ..，Path.startsWith 分段比较）
- Rust: `WebdavController.rs:67-75` + `stubs.rs:5220-5226`（normalize 原样返回；starts_with 字符串前缀）
- 结果：`GET /reader3/webdav/../../../../etc/passwd` 可读任意文件；兄弟目录 `webdav2` 误放行
- 同病：`FileController.rs:36-43` resolve_secure_path vs `FileController.kt:22-26`
- 状态：[未修]

### 10.2 【安全·高危】secure 模式下 WebDAV 认证被 stub 完全旁路（且回落 "default" 用户目录）
- Kotlin: `WebdavController.kt:205-248`（appConfig.secure 真实值 + Basic 认证 + enable_webdav 校验）
- Rust: `WebdavController.rs:385-432`（get_app_config_secure → `stubs.rs:9456-9460` 硬编码 false）→ check_authorization 恒 true；put("username") 永不执行 → get_user_name_space 回落 "default" → 所有用户 WebDAV 文件落 default 目录
- 状态：[未修]

### 10.3 【客户端兼容·高】WebDAV 特征响应头全部缺失（DAV/MS-Author-Via/Allow/WWW-Authenticate/Expose-Headers）
- Kotlin: `WebdavController.kt:101-112`（addHeadersEndHandler 注入）
- Rust: `WebdavController.rs:210-221` 注册了闭包但 `runtime/vertx.rs:249-251` add_headers_end_handler 是 no-op
- 影响：Windows 资源管理器/RaiDrive/Nutstore 依赖 DAV 头协商，缺失可能拒绝挂载
- 状态：[未修]

### 10.4 【客户端兼容·中高】PROPFIND href 编码（url_encode_charset no-op，中文/空格未转义）与基准 URL 双差异（absolute_uri 取 axum Uri 只有 path 无 scheme/host）
- Kotlin: `WebdavController.kt:326` URLEncoder.encode + `:302` absoluteURI() 完整绝对 URL
- Rust: `WebdavController.rs:602`（no-op）+ `:567` absolute_uri（relative）
- 状态：[未修]

### 10.5 【行为·中】WebDAV 根路径（无尾斜杠）PROPFIND：Kotlin 404 vs Rust 207（Rust 标注修复——为一致需决策）
- 状态：[未修]

### 10.6 【行为·中低】MOVE/COPY 的 Destination 解析失败处理不同
- Kotlin: `WebdavController.kt:92-96`（URL 解析失败 → null → 400）
- Rust: `WebdavController.rs:82-92`（unwrap_or_default 得到 "" → resolve_webdav_path("") → home 目录 → 412 或 Overwrite 时删 home）
- 状态：[未修]

### 10.7 【行为·低】Basic 认证 scheme 大小写（Kotlin replace("Basic ", true) 忽略大小写）与 +/% 解码差异
- 状态：[未修]

### 10.8 【行为·低】syncFromWebdav 成功后 tmp 目录泄漏（finally 语义丢失）
- Kotlin: `BookController.kt:2046-2087`（finally deleteRecursively）
- Rust: `BookController.rs:2734-2786`（return true/false 直接退出，收尾 delete 不可达）
- 状态：[未修]

### 10.9 【行为·低】createUserBackup 显式 staging_dir.mkdirs()（Kotlin 无，大概率等价）
- 状态：[未修]

### 10.10 【性能·低】WebDAV 同步阻塞 + send_file 整读内存（同 16.9）
- 状态：[未修]

### 10.11 【行为·低】GET 文件读取失败：Rust 200 空 body vs Kotlin 错误通道
- Rust: `WebdavController.rs:746-748` + `server.rs:363-371`（read 失败落 r.body=None → 200 空）
- 状态：[未修]

### 10.12 【兼容性·待确认】OPTIONS CORS 预检服务器层提前短路，Allow-Methods 缺 PROPFIND/MKCOL/MOVE/COPY/LOCK/UNLOCK（浏览器内 WebDAV 客户端失败；与 Kotlin 一致地缺 PROPFIND）
- 状态：[未修]

---

## 11. 详情页/封面/图片链差异（agent 11）

### 11.1 【严重】saveBook 时封面本地化失效——Rust 传 clone 导致 coverUrl 未被改写
- Kotlin: `BookController.kt:1398-1400`（saveBookCover/saveLocalBookCover 操作同一 Book 对象，coverUrl 改写反映到 saveBookToShelf 持久化）
- Rust: `BookController.rs:1951-1953`（save_book_cover(book.clone()) 对 clone 的修改丢失）→ 书架存远程 URL → 前端走 /cover 代理而非本地 /assets/
- 状态：[未修]

### 11.2 【严重】/reader3/cover 代理缺 User-Agent 与 Referer 头
- Kotlin: `BookController.kt:237-239`（putHeader UA + Referer=coverUrl 前缀）
- Rust: `BookController.rs:384-388`（web_client.get_abs 无任何头）
- 影响：防盗链图源 403 → 封面裂
- 状态：[未修]

### 11.3 【严重】详情页封面相对 URL 解析基准不同（redirectUrl vs 书源 URL）
- Kotlin: `webBook/BookInfo.kt:122-124`（getAbsoluteURL(redirectUrl, it)）
- Rust: `webBook/BookInfo.rs:160`（get_absolute_url(URL::parse(book_source_url), cover_url)）
- 影响：详情页跨域重定向 + 相对封面规则 → 封面 404
- 状态：[未修]

### 11.4 【中】Book JSON 序列化多 infoHtml/tocHtml/rootDir/userNameSpace（同 5.6）
- 状态：[未修]

### 11.5 【中】BookHelp.saveImage 失败路径：Rust 写空文件导致该图永久跳过（且 panic 会死锁）
- Kotlin: `help/BookHelp.kt:159-171`（try/catch/finally，网络异常不落盘；downloadImages.remove 必执行）
- Rust: `help/BookHelp.rs:258-267`（get_byte_array_await 失败返回空 Vec 仍 write_bytes 写空文件；remove 只在成功后执行 → 下载 panic 时 while contains 永等死锁）
- 影响：正文插图一次失败后永久空白
- 状态：[未修]

### 11.6 【中】getBookShelfBooks 刷新：Kotlin 16 并发 vs Rust 串行（同 5.7）
- 状态：[未修]

### 11.7 【低】bookInfoCache 实现差异：磁盘 LRU vs 内存无界（同 5.10）
- 状态：[未修]

### 11.8 【低】loginCheckJs 结果类型处理（同 1.10/3.6）
- 状态：[未修]

### 11.9 【低】saveLocalBookCover：超时 3s vs 30s + 空字节处理相反
- Kotlin: `BookController.kt:3571-3578` vs Rust: `BookController.rs:4509-4524`
- 状态：[未修]

### 11.10 【低】详情页请求 headerMap：Rust 无登录头且 header @js: 不求值（同 1.6/2.1）
- 状态：[未修]

### 11.11 【低】saveBookCover 失败时写空封面文件（Kotlin catch 后不落盘）
- Kotlin: `BookController.kt:3546-3556` vs Rust: `BookController.rs:4462-4484`
- 状态：[未修]

### 11.12 【低】详情页 fetch 的 header 回退与 JS 语义（合并到 1.6/3.6）
- 状态：[未修]

---

## 12. 辅助控制器链差异（agent 12）

### 12.1 【高危】Rust 请求处理路径存在未捕获 panic → 服务器进程崩溃
- 位置：`CURD.rs:139,226`（body_as_json().unwrap()）、`BookGroupController.rs:156`、`HttpTTSController.rs:59,66`、`HttpTTS.rs:121-122`（expect("name")/expect("url")）
- Kotlin: CURD.kt:74/113、HttpTTSController.kt:37/41（NPE → 500）
- 触发：POST /reader3/httpTTS/save {} 、/saveBookmark 空 body、/saveBookGroup 无 body
- 状态：[未修]

### 12.2 【中危·待确认】HttpTTS /deleteMulti 路由两侧都错误指向单删（前端发数组）——Kotlin 500 vs Rust 崩溃（既有缺陷被 panic 放大）
- YueduApi.kt:411 / YueduApi.rs:462（deleteMulti → delete）
- 状态：[未修]

### 12.3 【低危】Bookmark checker time 缺省值语义不同（Kotlin getLong("time") null 恒不匹配 vs Rust get_long("time",0)）
- Kotlin: `BookmarkController.kt:26` vs Rust: `BookmarkController.rs:34`
- 状态：[未修]

### 12.4 【低危】get_string 缺 key 返回 ""（Rust）vs null（Kotlin）（影响 ReplaceRule/HttpTTS 的 name checker）
- 状态：[未修]

### 12.5 【无害】批量接口畸形 body：Kotlin 500 vs Rust 优雅"参数错误"
- 状态：[未修]

### 12.6 【无害】HttpTTS 序列化多 userNameSpace 字段
- 状态：[未修]

### 12.7 【前端 bug】loadHttpTTS 未定义 + setHttpTTS mutation 无调用方 → HttpTTS 列表前端永不加载（见 20.1）
- 状态：[未修]

### 12.8 【无差异】getBookContent 正文替换链两侧均被注释禁用（替换规则由前端 filterContent 应用）
- 状态：无需修复（但注意：为完全一致需决策——Kotlin 注释掉了，Rust 也注释掉，一致）

---

## 13. jsoup 引擎深层差异（agent 13）

### 13.1 【功能 bug】text() 提取缺失空白规范化与块级/br 分隔
- Kotlin: `AnalyzeByJSoup.kt:215-219`（jsoup text()：appendNormalisedText 折叠空白，块级元素和 br 后追加空格，trim）
- Rust: `stubs.rs:1236-1238`（Element::text() 直接返回存储 text 字段）；text 字段由 `runtime/html.rs:85-88,215-228,254-263` scraper `e.text()` 原始拼接（无空白处理、无分隔、br 不换行）；scraper 把 <script> 内容混入文本（jsoup 不含）
- 影响：所有 @text 规则（目录/简介/正文）
- 修复参考：`JsoupExtensions.rs:47-54` appendNormalisedWhitespace
- 状态：[未修]

### 13.2 【功能 bug】JS 绑定 result（Elements）降级为拼接字符串，集合方法全失效
- Kotlin: `AnalyzeRule.kt:650-664`（JS 可调 result.eachText()/result.text()/result.attr()/result.hasClass()/result[0]）
- Rust: `AnalyzeRule.rs:573` + `stubs.rs:10361`（Any::Elements → Value::String 无分隔拼接）+ `runtime/js.rs:39-96`（bind_value 仅支持 JSON 标量/数组/对象）
- 影响：@js:result.eachText().join('|') 等全部失效
- 状态：[未修]

### 13.3 【功能 bug】:has / :matches / :containsWholeText / :matchesOwn 被整体剥离、条件忽略
- Kotlin: jsoup 原生支持
- Rust: `runtime/html.rs:151-165`（注释明言"条件忽略仅保留基础选择器"）
- 影响：@CSS:li:has(a) 返回所有 li 过度匹配，索引错位
- 状态：[未修]

### 13.4 【功能 bug】伪类不在末尾时选择器截断 + 组合伪类应用顺序颠倒
- Rust: `runtime/html.rs:126-212`（rfind 从尾部剥离：div:eq(0) span 的 span 被丢弃；li:first a 解析失败空结果；div:eq(2):contains(x) 先 contains 后 eq）
- Kotlin: jsoup 完整解析任意位置伪类
- 状态：[未修]

### 13.5 【功能 bug】id.xxx 前置规则恒空（Collector::collect 空操作）
- Kotlin: `AnalyzeByJSoup.kt:296`（Collector.collect(Evaluator.Id) 真实搜索）
- Rust: `AnalyzeByJSoup.rs:428` → `stubs.rs:7406-7412`（Elements::default() 空实现）
- 状态：[未修]

### 13.6 【功能 bug】@html 规则不再剔除 script/style（Elements.remove() 空操作）
- Kotlin: `AnalyzeByJSoup.kt:240-247`（select("script").remove() 后再 outerHtml）
- Rust: `AnalyzeByJSoup.rs:342-349`（`stubs.rs:7630-7632` remove 只清临时列表，原始 DOM 不变）
- 状态：[未修]

### 13.7 【功能 bug】ownText 返回整棵子树文本而非直接文本节点
- Kotlin: `AnalyzeByJSoup.kt:234-239`（jsoup ownText()）
- Rust: `stubs.rs:1275-1277`（own_text = text_of(&self.html) 全文拼接）
- 状态：[未修]

### 13.8 【功能 bug】textNodes 含所有后代的文本节点（scraper select("*") 展开）
- Kotlin: `AnalyzeByJSoup.kt:221-233`（jsoup textNodes() 仅直接子文本节点）
- Rust: `stubs.rs:9688-9707`（select("*") 遍历所有后代）
- 状态：[未修]

### 13.9 【待确认】getResultLast 属性分支先试 CSS 选择器再回退 attr（同 1.2）
- 状态：[未修]

### 13.10 【功能 bug·轻】:contains 大小写敏感且基于未规范化文本（jsoup :contains 大小写不敏感、对规范化文本匹配；:containsOwn 大小写敏感）
- Rust: `runtime/html.rs:62-67`（原始文本、大小写敏感）
- 状态：[未修]

### 13.11 【功能 bug·轻】Entities.unescape 仅 5 个命名实体，数字引用不解码
- Kotlin: `AnalyzeRule.kt:254-260`（jsoup 完整实体表 + &#123; 数字引用）
- Rust: `stubs.rs:473-479`（仅 &amp; &lt; &gt; &quot; &#39;）
- 状态：[未修]

### 13.12 【待确认】相对 URL 解析：url::Url 严格 vs java.net.URL 宽松（空格/非 ASCII 相对路径解析失败返回原样相对路径）
- 状态：[未修]

### 13.13 【待确认】无效选择器：Rust 静默空 vs Kotlin 抛异常（行为都是无结果）
- 状态：[未修]

### 13.14 【无害】空规则分支 element.data() 恒为空
- 状态：[未修]

### 13.15 【无害】text.xxx 前置规则不含根元素自身
- 状态：[未修]

---

## 14. 章节更新/缓存链差异（agent 14）

### 14.1 【待确认·多用户】非书架章节列表缓存：Kotlin 落盘 vs Rust 纯内存 + 无用户隔离（重启后缓存全失、跨用户串缓存）
- Kotlin: `BookController.kt:144-146,1727-1733,1787-1788`（ACache 文件落盘 3600s，按用户目录）
- Rust: `BookController.rs:254-258,2359-2370,2452-2457`（static OnceLock 内存 HashMap，无用户隔离）
- 状态：[未修]

### 14.2 【待确认·低】bookInfoCache：Kotlin 落盘 vs Rust 内存（重启丢失；当前前端路径影响小）
- 状态：[未修]

### 14.3 【待确认·低】invalidBookSourceCache：目录结构、键名、重启恢复三者不一致（重启后失效标记丢失 → 坏源被反复请求；无用户隔离；键名 hashCode vs md5Encode16）
- Kotlin: `BookController.kt:128-131,159-169` vs Rust: `BookController.rs:224-228,239-252,264-287`
- 状态：[未修]

### 14.4 【无害】getBookContent POST 顶层 chapterUrl 参数丢失 + Rust 独有按 URL 匹配块（同 1.4/1.8）
- 状态：[未修]

### 14.5 【无害·性能】getBookShelfBooks 并发 16 vs 串行（同 5.7）
- 状态：[未修]

### 14.6 【功能差异】PDF 阅读：Kotlin 图片渲染（stub 空 png 实际不可用）vs Rust 文本提取（有意修复可读）——决策项
- 状态：[未修]

### 14.7 【无害】JSON 序列化格式与键集差异（互读兼容）
- 状态：[未修]

### 14.8 【待确认·部署】storage 根路径 READER_APP_WORKDIR 覆盖（部署配置差异，已部署验证正常）
- 状态：[未修]（部署已用该变量，保持）

### 14.9 【无害】getLocalChapterList 异常路径：Kotlin 抛异常 vs Rust panic+catch_unwind（语义等价；若调用点未包 catch_unwind 则中断线程）
- 状态：[未修]

### 14.10 【无害】非书架章节缓存过期语义（TTL 同 3600s，载体不同）
- 状态：[未修]

---

## 15. 网络层差异（agent 15）

### 15.1 【严重】POST form 请求 body 完全丢失
- Kotlin: `OkHttpUtils.kt:135-145`（真 FormBody urlencoded）+ `AnalyzeUrl.kt:388-396`
- Rust: `OkHttpUtils.rs:304-323` → FormBodyBuilder 空壳（`stubs.rs:7433-7450` add/add_encoded 空操作，build() 返回空 RequestBody）→ `stubs.rs:663-668` post 只设 body=Some("")；form_fields（stubs.rs:631）从不填充
- 影响：POST form 书源（多数搜索/目录）发出空 body 且无 Content-Type
- 状态：[未修]

### 15.2 【严重】字符集双重解码 → 中文 GBK 站点乱码
- Kotlin: `OkHttpUtils.kt:99-115`（原始字节 → Content-Type charset → EncodingDetect 嗅探，只解码一次）
- Rust: `OkHttpUtils.rs:213-232` + `runtime/okhttp.rs:91`（resp.text() 已按 UTF-8 lossy 解码一次）→ `stubs.rs:7556-7560` content_type() 恒 None → 之后 EncodingDetect 在"重编码回 UTF-8 的字节"上嗅探 meta=gbk → 二次解码 → 乱码
- 状态：[未修]

### 15.3 【严重】二进制 body 损坏（封面/图片/音频下载全坏）
- Kotlin: `AnalyzeUrl.kt:464-499`（bytes() 原始字节）
- Rust: `AnalyzeUrl.rs:742-784` → `OkHttpUtils.rs:77-83` → `runtime/okhttp.rs:91` resp.text()（UTF-8 lossy 解码二进制）→ `stubs.rs:5266-5268` bytes() = 解码后 String 重编码
- 受害：EPUB 封面（BookController.rs:3390）、BookHelp.rs:259 图片保存、AnalyzeUrl.rs:602-605 hex
- 状态：[未修]

### 15.4 【中高】Set-Cookie 永不保存 → cookie jar 恒空
- Kotlin: `AnalyzeUrl.kt:406-412`（okhttp headers("Set-Cookie") 大小写不敏感、返回全部值）
- Rust: `AnalyzeUrl.rs:675-686` → `stubs.rs:752-757` headers("Set-Cookie") 大小写敏感（runtime/okhttp.rs:85-88 存的是小写 "set-cookie"）→ 查找恒空；多 Set-Cookie 只留最后一个
- 影响：enabledCookieJar 书源登录态丢失
- 状态：[未修]

### 15.5 【中】URL 参数未 percent-encode（url_encode_charset 空实现）
- Kotlin: `AnalyzeUrl.kt:217-235`（URLEncoder.encode）
- Rust: `AnalyzeUrl.rs:417-435` → `stubs.rs:4147-4149`（no-op）；HttpUrlBuilder add_query_parameter（stubs.rs:7503-7526）也不编码
- 影响：含中文/空格/&/=/+ 的搜索关键词破坏参数结构
- 状态：[未修]

### 15.6 【中】网络错误被静默吞掉 → 返回空 Response
- Kotlin: `OkHttpUtils.kt:81-97`（onFailure → resumeWithException 传播异常）
- Rust: `OkHttpUtils.rs:177-210`（Err → tx.send(Err) 后 `_ => Response::default()` status=0 空响应）；`stubs.rs:5563-5583` Chain::proceed 同
- 影响：连接失败/DNS 失败/超时时无任何错误信息
- 状态：[未修]

### 15.7 【中】postJson 请求不带 Content-Type: application/json
- Kotlin: `OkHttpUtils.kt:180-185`（toRequestBody("application/json; charset=UTF-8")）
- Rust: `OkHttpUtils.rs:422-431` → `stubs.rs:663-668`（只存 body 文本不存 mediaType）
- 状态：[未修]

### 15.8 【低-中】socks4 代理路径是死代码，实际按 socks5 发送
- Kotlin: `HttpHelper.kt:91-104`（java.net Proxy.Type.SOCKS 原生支持 socks4/5）
- Rust: `HttpHelper.rs:160-185` → `stubs.rs:5630-5638`（ProxyType::SOCKS 改写成 socks5://）→ `runtime/okhttp.rs:24-30` socks4 手写分支永不命中
- 状态：[未修]

### 15.9 【低】isSuccessful 把 3xx 当成功（200..400 vs 200..299）
- Kotlin: okhttp isSuccessful=200..299（AnalyzeUrl.kt:32-36 retry 依赖）
- Rust: `stubs.rs:740-742` = 200..400
- 状态：[未修]

### 15.10 【低】超时：read 15s → 总超时 45s（挂起服务器 Kotlin 15s 报错 Rust 拖 45s；慢 drip 响应差异）
- Kotlin: `HttpHelper.kt:35-37`（connect/write/read 各 15s）vs Rust: `HttpHelper.rs:74-84`（被 stubs.rs:5594-5602 忽略）+ runtime/okhttp.rs:33-34（connect 15s + 总 45s）
- 状态：[未修]

### 15.11 【待确认】跨域重定向时 reqwest 剥离 Cookie/Authorization（okhttp 保留全部头）
- Rust: `runtime/okhttp.rs:32` Policy::limited(10) 文档行为
- 状态：[未修]

### 15.12 【低】书源 header 的 @js 求值与 loginHeader 未合并（同 1.6）
- 状态：[未修]

### 15.13 【低】header 大小写敏感：Request.header HashMap 精确匹配 → 书源写小写 "user-agent" 时补双 UA
- 状态：[未修]

### 15.14 【低】StrResponse.headers() 恒空（stubs.rs:8044-8047）
- 状态：[未修]

### 15.15 【低】重定向上限 20 vs 10（无害）
- 状态：[未修]

### 15.16 【低】WebClient options 被忽略（vertx.rs:905 wrap() 丢弃 is_trust_all）→ 自签证书站点 WebClient 路径失败
- 状态：[未修]

---

## 16. vert.x 基础设施层差异（agent 16）

### 16.1 【P0】add_headers_end_handler 是 no-op → WebDAV 全部响应头缺失（同 10.3）
- Kotlin: vert.x 真实执行 addHeadersEndHandler
- Rust: `runtime/vertx.rs:249-251` 丢弃闭包
- 状态：[未修]

### 16.2 【P0】WebDAV PUT 等原始 body 接口二进制损坏（UTF-8 lossy）
- Kotlin: `context.getBody()` 原始字节（WebdavController.kt:374）
- Rust: `server.rs:171` 非 multipart 一律 String::from_utf8_lossy；`server.rs:344-345` raw_body = lossy 后字节；`vertx.rs:309-312` get_body 返回损坏数据
- 影响：WebDAV 上传 epub/zip 二进制静默损坏
- 状态：[未修]

### 16.3 【P0】body 大小限制：axum 2MB vs vert.x BodyHandler 10MB
- Kotlin: `RestVerticle.kt:77`（BodyHandler.create() 默认 10MB）
- Rust: `server.rs:468` axum Bytes 提取器受 DefaultBodyLimit 2MB
- 影响：2MB~10MB 上传/WebDAV PUT 直接 413
- 状态：[未修]

### 16.4 【P0】控制器 panic 无兜底 → 连接断开，无 500 JSON
- Kotlin: `RestVerticle.kt:145-160`（try/catch → onHandlerError）+ `:97-99` failureHandler
- Rust: `stubs.rs:7795-7810`（无 try/catch）；`vertx.rs:767-773` failure_handler no-op；`server.rs:300-374,395-446,463-478` 无 catch_unwind
- 影响：所有 API 错误路径（非法 JSON、缺字段、IO 失败）连接重置而非 500 JSON
- 状态：[未修]

### 16.5 【P1】路由选择只取"最高分单规则" → 全局 handler 链全部失效 + 同分双规则只执行第一条
- Kotlin: vert.x 按注册顺序匹配，handler next() 后继续下一个匹配路由（SessionHandler/CORS/BodyHandler/日志对每个请求执行）
- Rust: `server.rs:312-321`（> 严格比较只留一条）+ `vertx.rs:232-235` next() no-op
- 后果：Session 续期 handler（RestVerticle.rs:71-78）、CORS headers-end（:84-95）、/reader3/* 日志（:112-118）从不执行；`/book-assets/*` 与 `/epub/*` 各两条同分规则（JS 注入 + static）→ 只执行第一条 inject handler → static 永不执行 → 书籍资源全部 404
- 状态：[未修]

### 16.6 【P1】multipart 文件名：Rust 保存路径被当作文件名（字段名前缀 + 路径）
- Kotlin: vert.x uploadedFileName()=临时路径、fileName()=客户端原始文件名
- Rust: `server.rs:159-166`（保存为 storage/file-uploads/{字段名}_{文件名}，uploaded_file_name 与 file_name 都返回该路径）；普通文本字段也因 data.is_empty() 为 false 被写成文件
- 影响：上传资源 URL 与文件名不符（file0_ 前缀）；home/path 等文本字段落盘垃圾
- 状态：[未修]

### 16.7 【P1】multipart 普通表单字段未合并进 request params → /file/upload 的 home/path 字段丢失
- Kotlin: vert.x BodyHandler 默认 setMergeFormAttributes(true)
- Rust: `server.rs:146-169` 丢弃非文件字段；get_param 只查 query/path_params（vertx.rs:47-49）→ FileController.rs:53 requested_home() 恒空 → 上传落错目录且绕过 enable_webdav/enable_local_store 检查
- 状态：[未修]

### 16.8 【中】SSE 不流式（write 全缓冲到 end）
- Kotlin: setChunked(true)+write() 实时推送
- Rust: `stubs.rs:10193-10205`（write 往 body Vec 追加，end 一次性发送）
- 影响：搜索结果/进度条不再渐进出现
- 状态：[未修]

### 16.9 【中-高】send_file 整读内存 + 无 Content-Type 推断 + 无 Range
- Kotlin: vert.x sendFile 流式 + 扩展名推断 Content-Type + Range/206
- Rust: `server.rs:366-370`（std::fs::read 整读）；`vertx.rs:99-102` 只记路径；调用方未设 Content-Type 时缺失；read 失败 200 空 body
- 状态：[未修]

### 16.10 【中】Session cookie 不再续期 + store 无过期
- Kotlin: `RestVerticle.kt:44-54`（每次请求 addHeadersEndHandler 续期 2 天）+ LocalSessionStore 7 天清理
- Rust: 续期闭包因 no-op 失效（RestVerticle.rs:71-78）；session() 只在创建时写 Max-Age=604800（vertx.rs:367-370）；session_store（vertx.rs:492-495）无 TTL
- 影响：活跃用户 7 天后必然过期重登
- 状态：[未修]

### 16.11 【低】CORS 方法列表与预检响应体差异（Allow-Methods 多 OPTIONS；预检 200 空 body vs Kotlin JSON；两侧都不含 PROPFIND）
- 状态：[未修]

### 16.12 【低】方法不匹配：405 vs 404
- Kotlin: vert.x 405 vs Rust: 落 /* 静态 → 404
- 状态：[未修]

### 16.13 【低】静态文件：无缓存头/ETag/Last-Modified/Range；.js MIME 差异（text/javascript vs application/javascript）
- 状态：[未修]

### 16.14 【低】bodyAsJson 非法 JSON：Kotlin 抛 DecodeException → 500 vs Rust 原样保存静默按默认参数执行
- 状态：[未修]

### 16.15 【低】查询参数多值：last-wins vs first（现有接口均单值，无害）
- 状态：[未修]

---

## 17. 定时任务/后台任务差异（agent 17）

### 17.1 【严重】Rust auto_backup 静默完全不执行（async fn 未 await）
- Kotlin: `YueduApi.kt:584/590`（saveToWebdav suspend await）
- Rust: `YueduApi.rs:740/746`（save_to_webdav 是 async fn 但此处没 .await/block_on，future 被 drop，一行不执行）；日志照常打印像成功
- 状态：[未修]

### 17.2 【严重】Rust shelf_update_job 静默完全不执行（async fn 未 await）
- Kotlin: `YueduApi.kt:530/536`（getBookShelfBooks suspend）
- Rust: `YueduApi.rs:686/692`（get_book_shelf_books async 未 await）
- 状态：[未修]

### 17.3 【中高】Rust 每日任务每窗口执行 2 次（Kotlin cron 秒级 1 次）
- Rust: `server.rs:553/557/561` 用 minute 窗口 + 30s 轮询 → 23:50:00 和 23:50:30 各触发一次（无 last_run_min 去重）
- 影响：auto_backup/clear_user/auto_gc 每天各跑 2 遍
- 状态：[未修]

### 17.4 【高】Rust 定时线程单点故障：panic 使所有定时任务永久停止
- Rust: `server.rs:536-565` 全部任务一个循环线程；catch_unwind 只包 4 个调用点，`YueduApi::new()`（L543）和 chrono 代码不在 catch 内；`server.rs:546-549` 书架+书源共用一个 catch_unwind（shelf panic 连带跳过书源）
- Kotlin: 每任务独立 Spring 调度 + try/catch
- 状态：[未修]

### 17.5 【中】Rust 10 分钟任务可能跳过周期（30s 轮询 % 10 == 0 检查，跨边界丢轮）
- Kotlin: cron 延迟不丢周期
- 状态：[未修]

### 17.6 【中】并发模型：Kotlin 任务并发（Dispatchers.IO）vs Rust 全部串行（单循环线程）
- 状态：[未修]

### 17.7 【无害】auto_gc 是空操作（System::gc stub 空实现）
- 状态：[未修]

### 17.8 【无害】触发精度：秒级 vs 30 秒粒度（分钟级 gate 不影响结果）
- 状态：[未修]

### 17.9 【一致】配置门控逻辑完全一致
- 状态：无需修复

### 17.10 【一致】执行链与活跃用户判定一致
- 状态：无需修复

---

## 18. 工具层差异（agent 18）

### 18.1 【严重】本地 TXT 书籍解码完全忽略检测到的字符集（同 7.3）
- Kotlin: `TextFile.kt:40,74,130`（String(buffer, charset)）
- Rust: `TextFile.rs:80,104,162`（from_utf8_lossy；检测了 charset 结果丢弃）
- 状态：[未修]

### 18.2 【严重】JS 文件读取 API 系列同样丢弃检测结果
- Kotlin: `JsExtensions.kt:371-372,380,422-423,441`（String(file.readBytes(), charset(charsetName))）
- Rust: `JsExtensions.rs:627,636,701,725,735`（readTxtFile/readTxtFileWithCharset/getTxtInFolder/getZipStringContent/getZipStringContentWithCharset 都 from_utf8_lossy）
- 状态：[未修]

### 18.3 【严重】utf8ToGbk 变成恒等变换
- Kotlin: `JsExtensions.kt:317-321`（真实 UTF-8→GBK 字节后按 UTF-8 重解释）
- Rust: `JsExtensions.rs:551-558`（三段 from_utf8_lossy 等于原样返回）
- 状态：[未修]

### 18.4 【中】CacheManager.put 的 downcast_ref 恒为 None → queryTTF 内存缓存永不命中
- Kotlin: `JsExtensions.kt:499-510` + `CacheManager.kt:27,57-63`
- Rust: `JsExtensions.rs:859` → `CacheManager.rs:60-65` 依赖 `stubs.rs:1946-1948`（Any::downcast_ref 恒 None）
- 影响：queryTTF 每次都重新下载+解析字体
- 状态：[未修]

### 18.5 【中】失效书源缓存：重启后读不到 + 跨用户串扰（同 14.3）
- 状态：[未修]

### 18.6 【待确认】ZipFile.getInputStream 解压到临时文件（资源泄漏+并发竞争；损坏条目静默空）
- Kotlin: `ZipUtils.kt:283`（zip.getInputStream(entry) 流式）
- Rust: `stubs.rs:4704-4718`（写 %TEMP%/reader_zip_{path}_{name} 再读回；`let _ =` 吞错误）
- 状态：[未修]

### 18.7 【待确认】String.hashCode 用 SipHash 而非 Java 31-hash（ACache 文件名不同；内部自洽，跨版本缓存文件孤儿）
- Kotlin: `ACache.kt:722` vs Rust: `stubs.rs:991-995`
- 状态：[未修]

### 18.8 【待确认】CharsetMatch.getString 解码 = UTF-8 lossy（当前无调用者，悬挂炸弹）
- Kotlin: `CharsetMatch.java:270`（真实字符集解码）
- Rust: `CharsetMatch.rs:296` → `stubs.rs:7698`（退化 UTF-8 lossy）
- 状态：[未修]

### 18.9 【待确认】getBaseUrl 字节索引切片对 IDN 域名可 panic
- Kotlin: `NetworkUtils.kt:111`（char 索引）vs Rust: `NetworkUtils.rs:124-128` + `stubs.rs:980-988`（字节偏移 s[9..]，多字节字符中间切片 panic）
- 状态：[未修]

### 18.10 【待确认】decode_bytes_with_charset 把 gb18030 一律当 GBK 解（4 字节序列生僻字变 U+FFFD；latin1 映射 WINDOWS_1252）
- Kotlin: `OkHttpUtils.kt:113`（Charset.forName 支持 gb18030 4 字节）
- Rust: `OkHttpUtils.rs:239-242`（"gbk"|"gb2312"|"gb18030" 全部 encoding_rs::GBK）
- 状态：[未修]

### 18.11 【待确认】ZipUtils.getComments 恒返回空串（当前无调用者）
- 状态：[未修]

### 18.12 【无害】EncodingDetect.getEncode 增加 UTF-8 先验检查（单方面修复，更正确）
- 状态：[未修]（为一致需决策）

---

## 19. 探索/搜索配置链差异（agent 19）

### 19.1 【功能 bug】@js:/<js> URL 规则执行语义不同（exploreUrl 执行差异核心，同 8B.1/8B.2）
- Kotlin: `AppPattern.kt:6-7`（JS_PATTERN = "<js>([\w\W]*?)</js>|@js:([\w\W]*)" CASE_INSENSITIVE 贪婪到串尾）+ `AnalyzeUrl.kt:103-124`（@result 前缀替换、每次传完整 ruleUrl 作为 result 绑定）
- Rust: `AnalyzeUrl.rs:242-280`（@js: 只吃到下一个 @；无尾随 @ 不执行原始文本残留；不支持 @result；大小写敏感）
- 影响：主流写法 `"url": "@js:...计算整个URL..."`（无尾随 @）探索/搜索直接失败；Rust 自带测试断言的是自定义语义（tests/analyze_url_test.rs:48-70 与 Kotlin 不兼容）
- 状态：[未修]

### 19.2 【功能 bug】get_book_source_map 读取时所有书源索引恒为 0 → getBookSource 返回错误书源
- Kotlin: `BookSourceController.kt:507-528,244`（真实解析 JSON 数字索引）
- Rust: `BookSourceController.rs:988-1002`（Any::downcast_ref 占位恒 None → unwrap_or(0)）→ 所有书源索引 0
- 触发：bookSourceMap.json 有效时（Kotlin 写/备份恢复）；getBookSource 返回第 0 个书源、delete_book_source 误删索引 0
- 状态：[未修]

### 19.3 【无害】bookSourceExploreList 存储 null（同 4.3）
- 状态：[未修]

### 19.4 【待确认】/exploreBook 缺 ruleFindUrl：Kotlin 500 vs Rust 空串发请求（前端恒传参）
- 状态：[未修]

### 19.5 【无害】探索响应字段集合不同（infoHtml/tocHtml/userNameSpace 多出）
- 状态：[未修]

### 19.6 【待确认】parseJsonStringList fields 分支值类型（Kotlin 一律转字符串 vs Rust 保留原始）
- 状态：[未修]

### 19.7 【待确认】多源搜索并发批次终止细节（limitConcurrent 轮数差异）
- 状态：[未修]

### 19.8 【信息】搜索历史与搜索配置两版均无（非转录回归）
- 状态：无需修复

---

## 20. 前端阅读交互深链差异（agent 20）

### 20.1 【高】HttpTTS 管理功能整体失效（前端 bug：loadHttpTTS 未定义 + 列表永不加载）
- `web\src\components\HttpTTS.ts:136,176,262`（调用 this.$root.$children[0].loadHttpTTS(true)——不存在）；vuex.ts:459-460 setHttpTTS 无 commit 调用；无 /httpTTS/list 请求
- 后端路由完好（YueduApi.rs:458-462）
- 影响：HttpTTS 列表恒空、朗读不可用
- 状态：[未修]

### 20.2 【高】searchBookMulti 非 SSE 分页 lastIndex 不前进（同 3.1/3.2，后端 bug）
- 状态：[未修]

### 20.3 【高】secure 模式下 cacheBookSSE 缺 accessToken（前端 bug）
- `web\src\BookManage.ts:632-636`（params 仅 {url, refresh} 未带 accessToken；对比 :766-770 exportBook 带了）
- 后端 `BookController.rs:2901` 先 check_auth
- 影响：secure 模式"缓存到服务器"必然 NEED_LOGIN
- 状态：[未修]

### 20.4 【中】滚动模式下含图章节多图裂图（前端 bug）
- `Content.vue:290`（renderScrollChapterList 用 .replace("__API_ROOT__") 只替换第一处；分页路径 :85 用 replace(/__API_ROOT__/g)）
- 状态：[未修]

### 20.5 【中】SSE 错误事件前端读不到 errorMsg（前后端契约不匹配）
- 后端 SSE 端点写 `event: error` + data；前端 addEventListener("error", e => e.data)——EventSource error 事件不含 data
- 状态：[未修]

### 20.6 【中】上传导入的本地 epub 正文空白（存疑，继承 JAR）
- 前端 `Content.vue:485-495` renderEpub 要求 content.startsWith("/book-assets")；上传导入书 bookUrl=/assets/... → 后端返回 /assets/... 开头 → renderEpub return null
- 后端构造：`BookController.rs:941-962`（bookUrl.replace("storage/data/", "/book-assets/")——上传书不含 storage/data/）
- 状态：[未修]

### 20.7 【低】阅读进度本地键不含 bookUrl（与 JAR 一致，仅提示）
- 状态：无需修复

### 20.8 【低】编辑保存内容的 localforage 键不一致（saveBookContent 用章节 url；saveContent 用 bookUrl——仅缓存清理命中率）
- 状态：[未修]

### 20.9 【低】getContent 章节越界后 tryRefresh 死循环风险（与 JAR 一致）
- 状态：无需修复

### 20.10 【低】搜索结果直接阅读不保存进度（契约一致，与 JAR 一致）
- 状态：无需修复

---

## 修复优先级规划（P0 基础设施 → P1 核心功能 → P2 一致性）

### P0（服务器可用性/数据安全，先修）
- 16.4 / 4.1 / 6.2 / 6.3 / 7.1 / 12.1：控制器 panic 无兜底 → handler 外层统一 catch_unwind + 500 JSON
- 16.3：body 限制 2MB → 10MB（axum DefaultBodyLimit）
- 16.2：原始 body 二进制损坏 → 保留原始字节
- 10.1：WebDAV/FileController 路径穿越
- 10.2：WebDAV 认证 stub 旁路（secure 配置读环境变量）

### P1（核心功能）
- 15.1/15.2/15.3/15.4/15.5/15.6/15.7：网络层（form body/字符集/二进制/cookie/encode/错误/Content-Type）
- 7.2：PDF 末页 off-by-one
- 7.3/18.1/18.2/18.3：txt/JS 文件 GBK 解码、utf8ToGbk
- 11.1/11.2/11.3：封面（clone/UA+Referer/基准 URL）
- 3.1/3.2/19.2/20.2：lastIndex、bookSourceMap 索引
- 17.1/17.2：定时任务 async 未 await
- 16.5：路由链（/book-assets、/epub 404）
- 16.1/10.3：add_headers_end_handler（WebDAV 头）
- 16.7：multipart 表单字段合并
- 16.6：multipart 文件名
- 13.1/13.3/13.4/13.5/13.6/13.7/13.8/13.10/13.11：jsoup 引擎
- 8B.1/8B.2/19.1：@js: 语义（对齐 Kotlin 贪婪+@result）
- 8B.3/8B.5/8B.8：JS cookie/java 绑定
- 8B.4：JsExtensions 缺失方法
- 6.1：logout token_map
- 3.3/3.4：{{bookName}} 自引用、@put: 变量
- 1.1/1.4/1.6/2.1/2.2/2.3/9.1/9.5/9.6/9.7：header/ruleData/chapterUrl
- 1.7：PDF 页向量
- 5.1：saveBookProgress 负数 index

### P2（一致性打磨）
- 序列化字段（userNameSpace/infoHtml/tocHtml/rootDir/null）、默认值、顺序、404 vs 405、SSE 流式、缓存落盘、Cookie 续期、CORS、静态文件头、URL 编码、getBaseUrl panic、IDN、hashCode、临时文件泄漏、定时任务 2 次/跳轮/单点、HttpTTS 前端、cacheBookSSE token、__API_ROOT__ 滚动、SSE error 事件、epub 上传空白、前端杂项
