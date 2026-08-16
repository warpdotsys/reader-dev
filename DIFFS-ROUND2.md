# Kotlin 原版 vs Rust 转录 — 第二轮深度差异排查报告（ROUND 2）

> 7 个 agent 于 2026-08-17 完成。本轮覆盖：配置/启动/部署链、正则/字符串/日期工具、存储与并发安全、规则引擎剩余方法、前端契约剩余、媒体/下载/字体链、实体序列化 round-trip。
> 状态标记：[未修] 待修复 / [决策] 需决策 / [架构] 架构级限制 / [已修] 已处理。

---

## A. 配置/启动/部署链（agent 1）

### A1 [未修·严重] MongoManager 假初始化——mongoUri 配置后不真实连接、文件静默丢弃
- Kotlin: `MongoManager.kt:21-29` 真实建连（失败保持 null）
- Rust: `MongoManager.rs:29-41` → `stubs.rs:7558-7564` MongoClients::create 空壳（恒返回 default，is_init 恒 true，file_storage 假 collection）
- 影响: 配 READER_APP_MONGOURI 的文件写入静默丢弃；无效 URI 不报错
- 验证: `READER_APP_MONGOURI=mongodb://127.0.0.1:1/x` 启动，is_init() 仍 true

### A2 [未修·严重] RemoteWebview 桥接死代码——remoteWebviewApi 配置后 webView 书源仍全部失败
- Kotlin: `ReaderAdapter.kt:31-59` 真实请求 `${api}/render.html`
- Rust: `ReaderAdapterHelper.rs:81-100` get_str_response_by_remote_webview 恒 `None`（注释自认占位）→ AnalyzeUrl.rs:624-648 panic("不支持webview") → 500
- 影响: 配 READER_APP_REMOTEWEBVIEWAPI 后 webView=true 书源必失败（Kotlin 正常）
- 验证: 配 REMOTEWEBVIEWAPI 搜索 webview 书源——Kotlin 渲染 HTML、Rust 500

### A3 [已修·部署] Dockerfile WORKDIR /app vs compose 挂载 /data/storage 失联（git 812a8cd 重构引入）
- 当前生产部署已用 READER_APP_WORKDIR=/ 绕开（相对路径 storage = /storage 挂载 ✓）；docker-compose.yml:11 仍挂 /data/storage（与 /app 不符）——若用户按 compose 部署会丢数据
- 建议: compose 补 `-e READER_APP_WORKDIR=/data` 或改挂载

### A4 [未修·高] 配置默认值不一致（application.yml 生效值 vs AppConfig.rs default）
| 字段 | Kotlin 生效默认 (application.yml) | Rust 默认 | 影响 |
|---|---|---|---|
| userLimit | 15 | 500000 | 第 16 个用户注册被拒 vs 放行 |
| cacheChapterContent | true | false | 章节缓存默认关 |
| shelfUpdateInteval | 30 | 10 | 书架刷新频率 3 倍 |
| defaultUserEnableWebdav | true | false | 新用户 webdav 默认关 |
| defaultUserEnableLocalStore | true | false | 同上 |
| defaultUserBookSourceLimit | 100 | 200 | 书源上限 |
- 验证: 不设 env 启动两端，注册第 16 用户对比

### A5 [未修·高] READER_SERVER_PORT / READER_SERVER_CONTEXTPATH 无效
- Kotlin: `YueduApi.kt:414-421` setupPort 读 env 作为监听端口
- Rust: `YueduApi.rs:474-483` setup_port 读 env 但 `runtime/server.rs` 用 main 参数绑定端口——env 覆盖被丢弃；contextPath 同样只认 --contextPath=
- 验证: `READER_SERVER_PORT=9999 cargo run` → 仍监听 8080

### A6 [未修·中] 7 个配置字段无 READER_APP_* 绑定
- 未绑定: showUI、debug、packaged、exportUseReplace、exportCharset、exportNoChapterName、exportPictureFile
- 影响: 导出编码固定 UTF-8（Kotlin 设 READER_APP_EXPORTCHARSET=GBK 可生效）；exportToTxt 经 FilesUtil.rs:598 appendText 固定 UTF-8
- 验证: 设 EXPORTCHARSET=GBK 导出 txt——Kotlin GBK、Rust UTF-8

### A7 [决策] 文档承诺不符
- README.md:75 称 cacheChapterContent 默认关闭"与原版 Kotlin 一致"——与 application.yml:16 (true) 矛盾
- doc.md:295-301 remote-webview 部署流程失效（A2）；doc.md:280 `java -jar /app/bin/reader.jar` 为 Kotlin 时代命令
- ROADMAP.md:128 的 500000 默认差异未在任何文档说明

### A8 [架构] SpringContextUtils APPLICATION_CONTEXT 恒 None
- `SpringContextUtils.rs:18` OnceLock 从未被 set；get_bean_by_name 恒 None；VertExt 被迫直接读 READER_APP_WORKDIR env 绕开
- YueduApi::new() 每 30 秒定时重建（server.rs:642-647）——运行中改 env 会改变配置（Kotlin 单例不会）
- 当前已工作（部署用 WORKDIR=/）——架构性占位

### A9 [未修·中低] --ui 模式 windowConfig.json 支持缺失
- Kotlin: `ReaderUIApplication.kt:245-283` 读 windowConfig.json 覆盖端口/UI 配置 + 窗口记忆
- Rust: main.rs --ui 仅 open_browser
- 影响: 桌面 UI 模式降级为"开浏览器"

### A10 [未修·低] 启动参数形态差异
- Kotlin: Spring 支持 `--reader.app.workDir=/data` 等任意 command-line property（doc.md 部署示例）
- Rust: main.rs 仅识别 --port=/--contextPath=/--workdir=/--ui，其余静默忽略
- 验证: 按 doc.md 参数形式启动不生效且无提示

### A11 [决策] 定时任务门控差异（A4 衍生）
- shelfUpdateInteval 默认 10 → 书架刷新频率为 Kotlin 的 3 倍

---

## B. 正则/字符串/日期工具层（agent 2）——引擎事实：Rust Pattern 用 fancy-regex 0.19（stubs.rs:148-232），替换/分割用纯 regex crate（无 lookaround）；Kotlin 全用 java.util.regex

### B1 [未修·P0 极高] MULTILINE 标志被 compile_with 静默丢弃——TXT 目录解析整体失效
- Kotlin: `TextFile.kt:79,341` toPattern(Pattern.MULTILINE)
- Rust: `stubs.rs:161-167` compile_with 只处理 CASE_INSENSITIVE（=1），MULTILINE（=8）忽略
- 影响: 默认 txtTocRule（全部 `^[ 　\t]{0,4}…章…{0,30}$`）按整块(512KB)锚点匹配——本地 TXT 目录基本失效
- 实测: `^第一章$` 对 `"a\n第一章\nb"` → Rust false、`(?m)` true
- 验证: 导入 txt 书看目录

### B2 [未修·P0 极高] Matcher::group_idx 在匹配子串上重新执行整个正则
- Kotlin: `AnalyzeByRegex.kt:18-19,45-46` resM.group(groupIndex) 直接取已匹配捕获组
- Rust: `stubs.rs:313-317` 对子串重跑 → 依赖上下文的正则（lookbehind/尾部 lookahead/\b）重捕获失败返回 None → `AnalyzeByRegex.rs:23` group_idx(0).unwrap() panic、:53 静默空串；`AnalyzeRule.rs:440` 同（panic 不被 Err 兜底捕获）
- 实测: `(?<=x)(a)` 对 "xa" 正常 find，但 captures("a") 重跑 → None
- 影响: 大量真实书源（带 lookahead/lookbehind 的目录/列表规则）静默空列或崩溃

### B3 [未修·P0 极高] \w \d \s \b Unicode 语义（中文内容行为相反）
- Kotlin: Java 默认 ASCII 类（\w=[a-zA-Z0-9_]、\d=[0-9]、\s 不含 U+3000）
- Rust: regex crate 默认 Unicode（\w 含 CJK、\d=\p{Nd}、\s 含 U+3000）
- 实测: `<title>(\w+)</title>` 对 "第一章" → Rust 捕获"第一章"、Java 无匹配；`\b章` 对 "abc章" → Rust false、Java true
- 影响: 书名/作者/章名/字数规则中 \w+/[\w\W]*/\d/\b 在中文内容上结果全面不同（[\w\W] 本身不受影响）

### B4 [未修·P0 极高] Pattern::compile 吞编译错误 → 空模式（静默垃圾 + 空匹配死循环挂死）
- Kotlin: Pattern.compile 抛 PatternSyntaxException → runCatching 捕获 → 字面量兜底
- Rust: `stubs.rs:156-159` 编译失败 → 空正则；`Matcher::find` 空匹配后 pos 不前进（Java 前进 1 字符）→ 无限循环挂死服务
- 实测: 规则含 \p{XDigit} 或 \Q...\E → Rust 空模式：getElement 返回 [""]、TextFile.rs:172 while matcher.find() 死循环
- 修复方向: compile 返回 Result + find 空匹配前进 + 字面量兜底

### B5 [未修·P1 高] 替换/分割走 regex crate（无 lookaround）vs 提取走 fancy
- Kotlin: `AnalyzeRule.kt:385` replace 用 Java 正则（lookahead/backref 可用）
- Rust: `stubs.rs:1214-1217` replace_regex_all、`:1048-1053` split_with_regex、js.rs:699-709 @replace: 走纯 regex（不支持 lookaround）→ 编译错误 → 字面量兜底（替换不生效）
- 实测: `正文(?!完|结)` — regex crate 编译错误、fancy 正常
- 影响: 替换类规则静默失效
- 修复方向: 统一引擎（全部 fancy）

### B6 [未修·P1 高] $ ^ . 行终止符语义（CRLF 差异）
- Kotlin: Java `$` 无 MULTILINE 也匹配末尾终止符前；`.` 不匹配 \r/\u0085/\u2028/29
- Rust: `abc$` 对 "abc\n" → false；`a.b` 对 "a\rb" → true
- 影响: CRLF 网页/小说：标题规则带 \r 或失配

### B7 [未修·P1 高] 替换串 $ 引用与 \ 转义
- Kotlin: replaceAll("$2" 不存在的组) → IndexOutOfBoundsException → 字面量兜底；`\$1` → 字面量 $1
- Rust: `$2` 不存在 → 静默空串；`\\$1` → 反斜杠保留且 $1 仍展开
- 影响: ruleReplace 引用错误组时行为不同

### B8 [未修·P1 高] SimpleDateFormat 模式翻译缺陷 + parse 时区错误
- Kotlin: Java SimpleDateFormat（引号字面量、MMM 简月、yy 两位年、本地时区）
- Rust: `stubs.rs:1588-1653,6706-6719` 引号未剥离（`yyyy-MM-dd'T'HH:mm:ss` → 输出含 `'`）、MMM→%B 全月名、yy→%Y 全年份、补零/空格差异、SSS/a/h 未处理；parse 用 UTC（中国时区差 8 小时）
- 影响: dateConvert、JS java.formatDate、章节 updateTime 解析全部偏移/乱码

### B9 [未修·P1 高] TextFile 章节字节偏移：UTF-8 字节数 vs 文件 charset 字节数（GBK 书错位）
- Kotlin: `TextFile.kt:143,153,209` toByteArray(charset).size（按文件编码计偏移）
- Rust: `TextFile.rs:177,187,245` as_bytes().len()（UTF-8 字节）——GBK 下每字 2 字节 vs UTF-8 3 字节 → 章节 start/end 逐步漂移错位
- 影响: GBK 编码 TXT 书

### B10 [未修·P2 中高] matches() 语义：is_match（部分匹配）vs Kotlin Regex.matches（全串）
- `stubs.rs:180-186` Pattern::matches = is_match；StringExtensions.rs:80 isTrue 对 "xfalsex" → false（Kotlin true）；`Matcher::matches()` 恒 false（从未 find）→ StringUtils.isNumeric 恒 false → wordCountFormat（BookList/BookInfo 字数）失效（"123" 不转 "123字"）
- 验证: isNumeric("123") 两端

### B11 [未修·P2 中] 命名组 groupCount 不一致
- Kotlin: Java groupCount 只数编号组；(?<name>x)(y) → 1
- Rust: `stubs.rs:319-323` captures_len()-1 → 2 → AnalyzeByRegex.getElements 多收集一列 "" → 列表字段错位

### B12 [未修·P2 中高] substring 越界：Java 异常（可捕获）vs Rust panic（线程终止）
- `stubs.rs:1084-1090` s[begin..end] 越界/非字符边界直接 panic；index_of 返回 -1 再传给 substring → panic
- 影响: 异常书源规则可导致请求级崩溃（有 handler 兜底 500 但不优雅）

### B13 [未修·P2 中] removeUTFCharacters 的 appendReplacement/appendTail 丢前置文本 + 代理对 panic
- Kotlin: `StringUtils.kt:293-302` Java appendReplacement 先复制匹配前文本
- Rust: `stubs.rs:6744-6757` appendReplacement 只追加 replacement（"abc\u4e00def" → Kotlin "abc一def" vs Rust "一def"）；char::from_u32(\uD83D).unwrap() 代理对 panic
- 当前无调用点（潜在 API）

### B14 [未修·低] 日期辅助 Calendar.get(HOUR) 12 小时制 vs 24 小时制（dateConvert_source 相对时间文案差异）

### B15 [未修·低] DecimalFormat/String.format("%.0f")/hashCode 格式层简化（1.5KB vs 1.50KB、2.5 → 2 vs 3 HALF_UP、SipHash vs Java hashCode）

---

## C. 存储层与并发安全（agent 3）

### C1 [未修·Critical] CURD 表存储路径硬编码 CWD 相对路径 + 读空后静默清空全表
- Kotlin: `VertExt.kt:199` getStorage → getStoragePath()（workDir/storage）
- Rust: `DB.rs:87-94` table_file_path() 硬编码 `"storage/data/{ns}/{name}.json"` 相对 CWD——绕过 get_storage_path（READER_APP_WORKDIR）
- 影响: 容器内 cwd≠/ 时（如 compose 默认）CURD 表（bookSource/bookGroup/bookmark/replaceRule/httpTTS/rssSource）读写到错误目录；叠加 load_cached 解析失败静默空数组 → 下次 save 整表覆盖 = 级联数据丢失
- 当前部署 cwd=/ 碰巧一致（相对路径 = /storage）——但 Dockerfile WORKDIR=/app 时（compose 场景）不同
- 修复: DB.rs 改用 work_dir_multi

### C2 [未修·High] 文件锁整体失效
- Kotlin: `VertExt.kt:60,435-440` synchronized(LRUCache<ReadWriteLock>) + tryLock(10s) 真实
- Rust: `stubs.rs:3119-3145` Lock::try_lock 恒 true、unlock 空操作；`VertExt.rs:102-105` STORAGE_LOCKS 是 thread_local（每线程独立）
- 影响: 完全无跨线程互斥；服务器线程 + worker 线程 + 定时任务线程并发写文件 = 丢失更新

### C3 [未修·High] 写回原子性缺失/错误静默吞掉
- Kotlin: `VertExt.kt:166-175` createTempFile（随机名）+ ATOMIC_MOVE + .backup.json；失败 throw（500 可见）
- Rust: `stubs.rs:3074-3095` 临时文件名毫秒时间戳（同毫秒两次保存互覆盖）；move_path 忽略 ATOMIC_MOVE/REPLACE；错误全 `let _ =` 吞掉
- 影响: Windows 上 rename 目标存在（.backup.json 短名残留）必失败且被吞 → 保存静默不生效、客户端却收到成功

### C4 [未修·High] BookGroup onCheckEnd 回写失效（groupId/order 永不落盘）
- Kotlin: `JSONTable.kt:47-53` 先 onCheckEnd（分配唯一 bitmask groupId、order=max+1）再序列化
- Rust: `DB.rs:155-187` 先序列化再 on_check_end（且 trait &T 不可变）——`BookGroupController.rs:244-248` 注释自认"降级忽略"
- 影响: 新分组全部 groupId=0/order=0 → 分组删除、排序、按 id 操作全部错乱
- 验证: 保存两个分组看 bookGroup.json 的 groupId

### C5 [未修·High] CURD 表无锁 read-modify-write → 并发丢失更新（叠加 C2）
- DB.rs:161-239 每次 load_cached 全量读 → 内存改 → flush 全量写；无锁

### C6 [架构] async fn 内嵌套 block_on 阻塞事件循环 + noop waker 自旋
- stubs.rs:4396-4418 block_on 用 noop waker 自旋（yield_now 忙等）——任何返回 Pending 的 future 无限自旋
- BookController.rs:1850-1902 search_book_with_source 内 block_on + get_available_book_source 的 buffer_unordered(16) 路径中每个 poll 同步执行完整 HTTP——刷新源期间全站无响应
- 当前网络层全部 blocking/独立线程幸免（无 Pending future）
- 架构级：单线程 tokio + 同步阻塞

### C7 [架构] 自旋 Mutex 在单线程 runtime 上的死锁风险
- stubs.rs:685-704 Mutex::lock 自旋 + yield_now 非重入；edit_shelf_book 用它；lock_sync/unlock_sync 是 no-op
- 风险: 同一线程持锁期间再 lock（刷新流程嵌套）→ 永久自旋卡死；worker 持锁做慢 IO 时阻塞全站
- 当前无已知触发路径（递归编辑书架场景未确认）

### C8 [未修·中] ACache 文件名 hash 算法不兼容（SipHash vs Java hashCode）
- Kotlin: ACache.kt:721-723 key.hashCode()（31-hash）
- Rust: stubs.rs:1116-1120 DefaultHasher(SipHash)——同名 key 不同文件名
- 影响: Kotlin 写的缓存文件 Rust 找不到（cookie jar/JS cacheFile 跨实现迁移后 miss）
- 注: 日期头格式（13 位毫秒+秒+空格）已确认兼容

### C9 [未修·中] BaseController::limit_concurrent_need_continue 死代码——一旦调用即死循环
- stubs.rs:8293-8306 Deferred::from_future 丢弃 future、is_completed 恒 false → while 永不退出
- 当前无调用方（grep 仅自身引用）——潜在陷阱

### C10 [未修·中] 损坏/非数组 JSON 行为分裂
- stubs.rs:1937-1939 JsonArray::new_parsed 失败回退 vec![原始字符串]（垃圾当记录）
- DB.rs:100 load_cached 失败回退空数组 → 下次 save 覆盖
- Kotlin: asJsonArray 解析失败抛异常 → 500 不动数据

### C11 [架构] &'static mut YueduApi 跨线程共享
- server.rs:682-683 Box::leak + 服务器线程/worker 线程/定时任务线程同时访问同一对象图
- 局部用 Arc<Mutex>（LocalCache/session）安全；非同步字段（Rc<RefCell> 类状态）并发 = 数据竞争
- 已审计：当前控制器无可变非同步字段（大部分只读/克隆）——但脆弱

### C12 [已修·部署] 存储根路径 READER_APP_WORKDIR env vs Kotlin 只认 Spring AppConfig（部署已用 WORKDIR=/）

### C13 [未修·低] JSONTable.rs 死代码 + DB.table("SQL") 分支退化（两套实现并存行为分裂——维护陷阱）

### C14 [未修·低] MongoManager 断线重连失效（OnceLock 一次性——Mongo 服务晚启动则永久失效）

### C15 [已确认] catch_unwind 覆盖审计：Handler/定时任务/worker/headers-end 全部有兜底；残余低风险在 execute_rules 非 handler 段与 start_server 主 future（当前无 unwrap 高危点）

---

## D. 规则引擎剩余方法（agent 4）

### D1 [未修·严重] getStringList 的 XPath/Json 分支走死 stub，恒返回空
- Kotlin: `AnalyzeRule.kt:167-168` Mode.Json/XPath → 真实 getStringList
- Rust: `AnalyzeRule.rs:190-201` → analyze_rule_stub_analyze_by_j_son_path_get_string_list（stubs.rs:9812）/x_path（:9735，注释"一律返回空占位"）——而 AnalyzeByJSonPath.rs:79/AnalyzeByXPath.rs:106 的真实 get_string_list 是 pub 且完整，只是没接线
- 影响: getStringList（含 isUrl=true 取链接列表）对 XPath 恒空；Json 规则绕过 &&/||/%% 切分与 {$...} 内嵌替换（$.a&&$.b 整串 query 失败）
- 验证: getStringList("$.a&&$.b")、getStringList("//div[@class='book']/a/@href")
- 修复: 接线到真实方法（一行级修复）

### D2 [未修·严重] JSONPath 过滤/切片/通配语法缺口（json_path.rs vs Jayway）
- Kotlin: Jayway 支持 [?(@.a=='x' && @.b=='y')]、[?(@.n>1)]、!=、[0:2]、$.*、$["key"]
- Rust: `src/runtime/json_path.rs:16-183` 自研解析器：&&/|| rhs 粘连（'x' && @.b=='y' 整个当期望值→不匹配空）；>、>=、<=、!= 被静默丢弃（find_operator 只找 =）→ 返回全部条目（比 Kotlin 多数据——错误更危险）；[0:2]/$.*/$["key"] token 静默丢弃；类型宽松（'1' 匹配 1）
- 验证: query("{\"a\":[{\"n\":2},{\"n\":1}]}", "$.a[?(@.n>1)]") — Kotlin 1 条、Rust 2 条

### D3 [未修·严重] XPath 引擎语义差异（xpath.rs vs JXPath）
- `//a/@href` 在 select_nodes 中 need_attr 被丢弃 → 返回元素文本而非属性值（规则链内自相矛盾——JS @xpath: 正确取属性）
- `[n]`/position() 索引全局化（//ul/li[1] 只返回全文档第 1 个 li，JXPath 是每个 ul 的第一个）
- text() 返回元素全文本（含后代）vs JXPath 直接文本节点逐个
- 字符串 != 谓词静默丢弃；position()>2/<2 不过滤；.// 相对轴解析失败
- 验证: getString("//a/@href")（对比 href vs 文本）；多 ul 文档 getElements("//ul/li[1]")

### D4 [未修·高] splitNotBlank 缺 trim
- Kotlin: `StringExtensions.kt:55-57` split + trim + filterNot(blank)
- Rust: `stubs.rs:1195-1200` 只 filter 空——"re1 && re2" → ["re1 ", " re2"]（正则带空格语义改变）；analyzeFields 的 "a=1& b=2" key 变 " b"
- 影响: AnalyzeByRegex 路径与 POST 表单字段

### D5 [未修·高] 变量链查找/写入顺序不一致（put 缺 book 分支）
- Kotlin: put = chapter→book→ruleData；get = chapter→book→ruleData
- Rust: put = chapter 或（book_variables + rule_data）——**跳过 Book 实体**（@put: 值不进 Book.variable 不随 DB 持久化）；get 多出 book_variables 层插在 book 与 rule_data 之间（优先级与 Kotlin 不同）
- 验证: 无 chapter 有 book 场景 @put:{x:'1'}@@@get:{x} 链；检查 book.variable 落库

### D6 [未修·高] RuleAnalyzer 字节/字符索引混用（中文规则 panic/错切）
- Kotlin: RuleAnalyzer.kt 全程 UTF-16 码元索引
- Rust: RuleAnalyzer.rs:63-117,122-316 consume_to 用字节偏移（String::find）、consume_to_any 用 chars().nth（字符计数）、queue[pos..end] 又按字节切片——中文在分隔符前时切片错位或 panic!("后未平衡")
- 验证: $.a&&$.b[?(@.title=='中文')] 走 getStringList；innerRule 前有中文

### D7 [未修·中] dataUriRegex 收窄
- Kotlin: `AppPattern.kt:14` data:.*?;base64,(.*)（任意类型）
- Rust: `AnalyzeUrl.rs:75-77` 只 data:image/...;base64（svg+xml 不匹配）；stubs.rs:4381 is_data_url 只认 data:image
- 影响: data:text/html;base64 / data:image/svg+xml 被当普通 HTTP 请求

### D8 [未修·中] fetchStart 并发控制：抛异常 vs 线程 sleep+状态重置
- Kotlin: `AnalyzeUrl.kt:325-327` 等待时 throw ConcurrentException（上层捕获重试）
- Rust: `AnalyzeUrl.rs:581-588` 直接 thread::sleep（阻塞 async 线程）+ 频率清零（限速失效）
- 影响: 并发控制行为不同 + 阻塞

### D9 [架构] eval_js 绑定：真实对象 vs JSON 字符串（方法缺失）
- Kotlin: book/source/chapter 绑定真实对象（source.getKey() 等可调）
- Rust: 绑定 JSON 字符串（属性可用，方法靠 js.rs polyfill——不完整）；rule_data 非 Book 时绑定 namespace 字符串（Kotlin 为 null）

### D10 [未修·中] analyzeJs 尾部 @result 段缺失 + 崩溃行为差异
- Kotlin: `AnalyzeUrl.kt:118-123` 循环后尾部段处理（@result 替换）
- Rust: `AnalyzeUrl.rs:236-283` 循环后直接赋值（尾部 @result 拼接丢失）

### D11 [未修·中] BookHelp.saveImage 无异常保护（下载锁泄漏——配合 F1 修复后死循环）
- Kotlin: `BookHelp.kt:150-172` try/catch/finally（finally 必 remove）
- Rust: `BookHelp.rs:220-268` 成功路径才 remove；请求 panic → download_images 永久残留 → 后续同 src while contains 死循环

### D12 [未修·低] getStringList 尾部空串（split('\n') 保留尾部空元素 vs Java split 丢弃）
- Rust: AnalyzeRule.rs:227-231 "a\nb\n" → ["a","b",""]（isUrl 路径被 getAbsoluteURL 滤掉，非 isUrl 多一项）

### D13 [未修·低] webview POST 分支 url vs urlNoQuery（Rust 端 webview 本就回退 panic——实际影响小）

### D14 [未修·低] 对象叶子 toString 格式（{a=1} vs {"a":1}）+ NativeObject {:?} 调试格式

### D15 [未修·低] 杂项：getImageSuffix 无点时 Kotlin 返回整串 vs Rust 空（回退 jpg）；splitSourceRule trim ASCII vs Unicode（NBSP 差异）；replaceKeyPageJs 全局替换 vs 逐段

---

## E. 前端契约剩余（agent 5）

### E1 [架构] 服务器并发模型：block_on 忙等 + 单线程 tokio → 网络型接口执行期间全站冻结
- 受影响接口: get_book_content 正文、cacheBookSSE、多源搜索/换源（limit_concurrent_with 同步等待）、RSS 抓取、TTS、远程封面
- Kotlin: 协程 + Dispatchers.IO 真并发挂起
- 影响: 搜索/缓存/封面下载期间其他 API 完全串行；多用户互相阻塞
- get_available_book_source 已改 buffer_unordered——其余 5 处未同步

### E2 [未修·中] cacheBookOnServer（批量"缓存到服务器"）静默无效
- 前端: BookManage.ts:748-763 POST /cacheBookOnServer {bookUrlList} → 提示"提交缓存任务成功"
- 后端: BookController.rs:3894-3899 launch(同步执行闭包) 内 async fn 未 poll 即 drop → 任务从未执行
- 影响: 批量缓存假成功（单本 cacheBookSSE 可用）

### E3 [未修·低] saveBookProgress 越界/负数 index 语义（Rust 新增范围检查——换源/目录刷新后 index 未重置时进度静默丢失，错误被前端吞掉）

### E4 [未修·低] 新建书签 time 缺省行为（Rust serde default 需实测——time=0 会互相覆盖）
- BookmarkForm.ts:85-86 不含 time；Bookmark.rs:17-44 serde default（time=now 或 0 取决于实现）

### E5 [决策] logout 行为与 Kotlin 完全一致（点击注销→弹登录框，不刷新）——备忘

### E6 [信息] 前端 checkBookSource（失效书源检测）POST body 传 bookSourceUrl 而 axios 只检查 params——检测不出失效源（与 Kotlin 相同前端，非转录差异）

### E7 [已确认] 其余 90+ API 契约全部对齐（路由/参数/字段/localStorage 键均一致——详见 agent 报告"无差异项"）

---

## F. 媒体/下载/字体链路（agent 6）

### F1 [未修·致命] 正文图片下载链整体失效——saveImages 的 r#async 是 no-op 占位
- Kotlin: `BookHelp.kt:124-172` scope.async 并发 saveImage（真实）
- Rust: `BookHelp.rs:173-268` scope.r#async → `stubs.rs:5990-6000` CoroutineScope::r#async no-op（Deferred 空、闭包从未执行）——3 个调用点全部无效（BookController.rs:1156/3142/3944）
- 影响: **图片从未下载**；正文 <img> 保持远程 URL 直连（防盗链裂图、无本地缓存）；比"写空文件"更彻底——整条链死
- 验证: 抓缓存章节后检查 storage/data/<ns>/<md5>/images/ 是否为空

### F2 [未修·严重] TTS 音频响应二进制损坏（MP3 经 UTF-8 lossy）
- Kotlin: `BookController.kt:3135,3200` response.end(Buffer.buffer(audioBytes)) 二进制安全
- Rust: `BookController.rs:4172/4221` Buffer::new(bytes).to_string() → vertx.rs:887-889 from_utf8_lossy → 无效字节全变 U+FFFD
- 影响: edge 朗读 100% 播放失败；base64=1 预缓存分支不受影响
- 验证: curl /reader3/book/tts?type=edge&text=测试 — 响应含大量 EF BF BD

### F3 [决策] PDF 本地书：页图渲染（Kotlin）vs 文本渲染（Rust）+ 空 png 永久裂图
- Kotlin: convertPdfPageToImage PDFBox 渲染 PNG
- Rust: lopdf 文本提取（扫描版 PDF 全空白）+ convert_pdf_page_to_image 占位写空文件——指向 output-N.png 的请求拿 0 字节且 exists() 检查永久不重新生成
- 决策: 保持文本渲染（服务端无 PDF 渲染器）还是接 pdfium？——当前已决策保留文本渲染

### F4 [未修·高] save_image 的 download_images.remove 不在 finally（配合 F1 修复后 panic 泄漏→永久空转）
- Kotlin: try/catch/finally（finally 必 remove）
- Rust: BookHelp.rs:259-267 移除在写文件后裸调用；网络错误 panic（OkHttpUtils.rs:209）→ catch_unwind 吞掉 → src 永久残留 → while contains 无限空转

### F5 [未修·高] HttpTTS 音源 getSpeakStream 缺 loginCheckJs / Content-Type 校验 / 重试
- Kotlin: `BookController.kt:3230-3289` AnalyzeUrl + loginCheckJs 求值 + contentType 正则校验 + 重试 5 次 + 连续 5 次错误返回 null
- Rust: `BookController.rs:4300-4333` 简单 GET + header + CookieStore 注入——无 loginCheckJs、无校验、无重试
- 影响: 需 JS 校验的 TTS 源 404；注释"占位返回 None"与实际不符

### F6 [未修·中] QueryTTF 字体名解码忽略 charset（UTF-16BE 乱码）
- Kotlin: `QueryTTF.java:188-191,496-506` platformID==1 ? UTF-8 : UTF_16BE
- Rust: `QueryTTF.rs:263-273` read_strings 忽略 charset 恒 UTF-8 lossy
- 影响: Windows/Unicode 平台（platform 0/3，绝大多数 TTF）字体家族名乱码 → JS 按字体名匹配失败（replaceFont 的 code→glyph 映射不受影响）

### F7 [未修·中] saveLocalBookCover 超时 3s vs 30s（死链封面拖 30s）

### F8 [已修] getBookCover 0 字节空封面缓存（Rust 已加 !is_empty() 检查——Kotlin 有缺陷）

### F9 [已修] 下载链 Content-Disposition 6 处调用点全部已 URL 编码 ✓

### F10 [决策] cbz 静态服务 Content-Type octet-stream（与 vert.x 一致——决策保留）

### F11 [信息] Web 端 TTS 本来就支持（路由齐全）——故障在 F2 二进制损坏 + F5 HttpTTS 能力缺失

### F12 [已确认] getUserInfo fonts 列表一致；getSystemInfo 的 reader.system.fonts 两侧均注释（恒空）

---

## G. 实体序列化 round-trip（agent 7）

### G1 [未修·高] map_opt 把"数字样字符串"写成 JSON 数字 → Rust 自身 save→load 丢字段
- Kotlin: `GsonExtensions.kt:16-26` 字符串恒写字符串（"wordCount":"12345"）
- Rust: `json_conv.rs:341-366` map_opt/insert_opt："123"→123、"1.5"→1.5、"true"→true；读侧 gs() 只用 as_str() → 读回 None → **字段静默丢失**
- 受影响: Book.wordCount、BookSource.concurrentRate/bookSourceGroup、RssSource.concurrentRate、SearchBook.wordCount、ReplaceRule.group、HttpTTS.contentType/concurrentRate、RssArticle.pubDate 等全部 insert_opt 的 Option<String> 字段
- 影响: Rust 保存→重启→读取数据永久丢失；Kotlin 读 Rust 数字 toString 保真但 "1e3"→"1000.0"、"001"→"1" 变值
- 修复: map_opt 恒写字符串或读侧兼容数字

### G2 [未修·高] HttpTTS.concurrentRate 默认 "0" 写为数字 0 → serde derive 严格解析失败 → 批量操作整体被拒
- HttpTTS.rs:74 默认 Some("0") → json_conv.rs:304 写 0；HttpTTS.rs:14-16 derive Deserialize 严格（Option<String> 收数字即错）
- 影响: 每条新建 TTS 都会让 saveMulti/deleteMulti 返回"参数错误"、DB::find_by 恒 None

### G3 [未修·中高] ReplaceRule.group 数值字符串 → serde 硬失败（同 G2 机理）

### G4 [未修·中] Book.readConfig 的 Converters 路径键名 snake_case vs Kotlin camelCase（当前死代码——bookshelf 内由 json_conv 内联 camelCase 写出双向一致 ✓；仅 Converters 路径未激活）

### G5 [未修·中] User API 序列化键名：json_conv 写 camelCase vs Kotlin User 字段 snake_case（当前 controller 均经 format_user map——两侧一致 camelCase；仅直接装箱 User 的边缘路径不同）

### G6 [未修·低] 缺键默认值：Kotlin 构造默认（currentTimeMillis）vs Rust serde default（0）——Book.dur_chapter_time 仍回退 0（1970）；Bookmark/TxtTocRule/BookGroup 同

### G7 [未修·低] rule* 子对象 Rust 显式 null vs Kotlin Gson 省略 null（读侧兼容，仅字节差异）

### G8 [未修·低] Book 额外键 infoHtml/tocHtml/rootDir/userNameSpace（互读无丢失——Kotlin 忽略未知键）

### G9 [未修·低] Kotlin Jackson 额外 getter 键（bookshelf.json 多 displayCover/displayIntro 等——Rust 忽略未知键无丢失）

### G10 [未修·低] ReturnData data:null 显式写出 vs Kotlin 省略（前端兼容）

### G11 [未修·低] 响应缩进：Kotlin Gson pretty vs Rust 紧凑（API 响应格式差异，解析兼容）

### G12 [已确认] BookSource.enabledCookieJar 缺键 None vs false（使用处 unwrap_or(false) 语义等价）

### G13 [已确认] 数字边界：时间戳/位掩码/计数全部对齐（serde_json 整数走 i64 精确路径，不经 f64）

---

## 修复优先级汇总

### P0（数据丢失/主链路失效，必须修）
1. **B1 MULTILINE 丢弃** — TXT 目录解析失效
2. **B2 group_idx 重跑正则** — 目录/列表规则空列/崩溃
3. **B3 \w\d\s\b Unicode 语义** — 中文规则全面结果错误（建议 (?-u) 对齐 Java）
4. **B4 compile 吞错误→空模式+find 死循环** — 服务挂死
5. **F1 saveImages r#async no-op** — 正文图片零下载
6. **D1 getStringList XPath/Json 死 stub** — 规则链断裂（一行接线）
7. **G1/G2 map_opt 数字强转** — Rust save→load 丢字段 + HttpTTS 批量操作被拒
8. **C1 DB.rs 路径硬编码** — 非 cwd=/ 部署时数据分裂/清空

### P1（功能错误/崩溃）
9. **F2 TTS 二进制损坏** — MP3 播放失败
10. **C4 BookGroup onCheckEnd 失效** — 分组功能错乱
11. **D2/D3 JSONPath/XPath 引擎缺口** — 规则结果错误（静默返回全量/空）
12. **D5 变量链 put 缺 book 分支** — @put: 不持久化
13. **D6 RuleAnalyzer 索引混用** — 中文规则 panic
14. **B5-B9 正则替换/行终止符/替换串/日期/GBK 偏移**
15. **F5 HttpTTS getSpeakStream 能力缺失**
16. **A4 配置默认值**（userLimit/cacheChapterContent/shelfUpdateInteval/webdav 默认）
17. **D4 splitNotBlank 缺 trim**
18. **E2 cacheBookOnServer 假成功**
19. **F4/F11 save_image finally 泄漏**
20. **C2/C3 文件锁/原子性**

### P2（一致性打磨）
21. B10-B15、D7-D15、E3-E4、F6/F7、G3-G11、A5/A6/A9/A10、C5/C8/C9/C10/C13/C14

### 架构级（需决策）
- C6（block_on 阻塞 + noop waker）、C7（自旋 Mutex）、C11（跨线程共享）、D9（eval_js 对象绑定）、E1（单线程 tokio 全站冻结）、A8（SpringContextUtils）
- F3（PDF 文本渲染——决策保留）

### 已修/已确认（无需处理）
- A3（部署 WORKDIR=/ 已绕开——compose 需补 env）、C12、C15、E7、F8/F9/F12、G12/G13、F10（决策）
