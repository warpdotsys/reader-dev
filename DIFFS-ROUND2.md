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

---

# 第三轮深度差异报告（ROUND 3，追加）

> 7 个 agent 于 2026-08-17 完成。覆盖：JS 引擎（BoA vs Rhino）能力、导出链、数据迁移兼容、安全边界、资源泄漏/性能、HTTP 协议响应头、系统管理杂项。
> 状态标记：[未修] / [决策] / [架构] / [已确认]

## H. JS 引擎能力深链（agent 1）——Kotlin Rhino 1.7.13 fork vs Rust BoA 0.21.1（default features float16+xsum，无 annex-b/intl）

### H1 [未修·严重] String.prototype.substr 缺失（及 trimLeft/trimRight）——annex-b feature 未启用
- 模式: `str.substr(0, 10)`、`s.substr(-3)`——老书源大量使用
- Kotlin Rhino fork: 支持（已 backport）；Rust BoA: TypeError → 规则失败
- 验证: BoA 0.21.1 string/mod.rs:173 在 #[cfg(feature="annex-b")] 内
- 修复: Cargo.toml 开启 boa_engine "annex-b"

### H2 [未修·严重] Date.prototype.toLocaleString/toLocaleDateString/toLocaleTimeString 抛 "Function Unimplemented"
- 模式: `new Date().toLocaleDateString()`（"今天日期"常见）
- Kotlin: 返回本地化日期串；Rust BoA: 三个方法体均 Err（date/mod.rs:1611-1666）
- 修复: eval_js_script 注入 polyfill

### H3 [未修·严重] Promise/async 结果悬置
- 模式: `(async () => {...})()`、async function
- Kotlin Rhino fork: 解析期 SyntaxError（无 async/await）→ 显式失败
- Rust BoA: 能解析但 eval_js_script 从不调用 context.run_jobs() → await 永不恢复；Promise 经 to_json 变 {} → **静默错误结果**（比显式失败更难排查）

### H4 [未修·严重] book/chapter 对象方法全部缺失
- 模式: `book.getVariable('xxx')`、`book.putVariable('a','b')`、`book.save()`、`chapter.getVariable('pos')`
- Kotlin: Java Bean 反射绑定方法可调；Rust: 绑定纯 JSON 数据对象 → TypeError
- 字段访问（book.name 等）正常——仅方法缺失
- 修复: 注入 getVariable/putVariable/save 委托（类似 source 方法注入）

### H5 [未修·严重] 任何 JS 错误 → Rust panic! 崩溃（非可捕获异常）
- Kotlin: ScriptException → 单书源规则失败，应用继续
- Rust: eval_js_script 返回 None → AnalyzeUrl.rs:488/AnalyzeRule.rs:597 panic!("JS 执行失败") → 请求崩溃（有 handler 兜底 500 但把单源失败放大）
- 修复: 改返回 Err 或空结果

### H6 [未修·高] undefined 结果字符串化: Kotlin "undefined" vs Rust "null"
- BoA 0.21 to_json 对 undefined 返回 Ok(None) → Any::Null → "null"；URL/书源地址被拼成含 "null" 的串

### H7 [未修·高] 数组/对象结果字符串化: Kotlin "a,b"/"[object Object]" vs Rust ["a","b"]/{"a":1}（getString 路径；getStringList 的 Any::List 语义正确）

### H8 [未修·中高] 数值字符串化: AnalyzeUrl 路径整值 Double "3.0" vs Kotlin "3"（%.0f）；NaN/Infinity → "null" vs "NaN"

### H9 [未修·中] null 结果: Kotlin 调用方回退 vs Rust 产出 "null" 文本参与拼接

### H10 [未修·中] Date.parse/new Date(str) 严格 ISO——"2024-01-01 12:00:00"/"2024/01/01" → NaN（Kotlin Rhino 宽松）

### H11 [决策] 正则引擎差异（regress 0.10.5 vs Rhino regexp）——Rust 增强：lookbehind/命名捕获组/replaceAll/matchAll（Rhino fork 均无）；个别边界偏差（lookbehind 内含捕获组/\b 在 lookbehind 后/Unicode i+u 折叠）——整体增强保留

### H12 [未修·中] @cookie:{key} 旁路失效（String(cookie) → "[object Object]"——正路 cookie.getCookie() 可用）

### H13 [未修·中] book 绑定 false/namespace 串（ruleData 非 Book 时）——`book == null` 判空反转

### H14 [未修·低] annex-b 其他缺失（anchor/big/blink 等 HTML 包装方法）；console.log 双引擎均无（书源用 java.log）

### H15 [已确认] encodeURI/decodeURI 规范一致（中文/emoji 正确；0.21 修复 decodeURI OOB panic）

**BoA 0.21 验证通过清单**：String 全部标准方法、RegExp（含 lookbehind/命名组）、Array 全部、JSON、Date 基础、Math、URI、ES6+（含可选链/??/replaceAll——Rust 增强）、\uXXXX/emoji/.length（UTF-16 语义一致）
**10 个典型书源 JS 片段**: java.ajax().match()[1] ✅、IIFE ✅、JSON.parse ✅、fromCharCode ✅、encodeURIComponent 拼接 ✅、getTime ✅、exec 循环 ✅、substr ❌、toLocaleDateString ❌、book.getVariable ❌、async ⚠️悬置

---

## I. 导出链（agent 2）

### I1 [未修·高] EPUB 导出 META-INF/container.xml 写入 0 字节
- Rust: `EpubWriter.rs:123-136` write_container 用 OutputStreamWriter::new(result_stream.clone())——ZipOutputStream::clone（253-262 行）将 zip 字段置 None → 所有 write() 空操作（4857/4877 行 if let Some(w) 守卫失效）
- 影响: 导出 epub 的 container.xml 为 0 字节——Calibre/Apple Books/静读天下无法打开
- 验证: unzip -l 或 epubcheck

### I2 [未修·高] EPUB mimetype 条目被 Deflate 压缩（违反 OCF 规范）
- Kotlin: mimetype 以 STORED 不压缩 + CRC/Size 预置 + zip 第一项
- Rust: `EpubWriter.rs:144-153` 设置了 STORED/CRC/Size 但 put_next_entry（230-236）恒用 Deflated 忽略 entry.method/size/crc
- 影响: epubcheck 必报错；部分严格阅读器拒绝打开

### I3 [未修·中] TXT/EPUB 导出章节缓存非法 UTF-8 整章变空
- Kotlin: readText() UTF-8 解码非法字节替换为 U+FFFD（内容不丢）
- Rust: read_to_string(...).unwrap_or_default() → 任何非法字节整章返回空串
- 验证: 向缓存 {index}.txt 写入 0x81 字节后导出

### I4 [未修·中] exportCharset/exportNoChapterName 未绑定 env（Rust export_to_txt 根本不读 export_charset——固定 UTF-8；export_no_chapter_name 恒 false）
- Kotlin: 设 READER_APP_EXPORTCHARSET=GBK 可生效
- 修复: 绑定 env + export_to_txt 读取 charset

### I5 [未修·低] EPUB 章节标题 replacen 字面量替代 replaceFirst 正则（"\\s*\\n\\s*" 永远匹配不到→序号 span/br 失效；含 \n 标题输出未闭合 span）

### I6 [未修·低] EPUB OPF spine toc 属性为空（get_spine_mut 缺失——attach_toc_resource 降级；epubcheck 报错，多数阅读器可容忍）

### I7 [未修·低] 空备份 zip: Kotlin 产出空 zip 成功 vs Rust 返回 None → 前端"备份失败"

### I8 [决策] sendFile 读取失败 404 vs Kotlin 200 空（Rust 更合理）

### I9 [已确认] TXT 导出编码/换行（UTF-8 无 BOM + LF）、Content-Disposition、Content-Type、单文件导出、前端触发全部一致

### I10 [已确认] 备份 zip 中文条目名 UTF-8 flag（zip crate EFS bit 11 自动设置）；epub 封面 LazyResource 降级嵌入字节（图片不丢）

---

## J. 数据迁移兼容（agent 3）——Kotlin 存量数据 → Rust 读取

### J1 [未修·严重] Cookie jar 文件名算法不匹配 → 全部 cookie miss → 迁移后所有站点需重新登录
- Kotlin: ACache.kt:722 key.hashCode()（Java 31-hash）；Rust: stubs.rs:1116-1120 DefaultHasher(SipHash13)
- 内容格式兼容（无头文件 isDue=false 正常读）——仅文件名 miss
- 影响: 登录态全部丢失（服务端账号不受影响）

### J2 [未修·严重] users.json 缺字段默认值 → 旧 JAR 文件迁移后书源/加书被禁
- Kotlin: data class 构造默认（enable_book_source=true、enable_rss_source=true、book_limit=200、book_source_limit=100）
- Rust: to_data_class() 缺字段默认 false/0 → enable_book_source=false（书源功能被禁）、book_limit=0（无法加书——bookshelf.size()>=0 恒真）、book_source_limit=0（无法加源）
- 仅影响旧版 JAR 写出的缺字段文件；新格式 13 键全匹配
- 修复: to_data_class 缺字段给 Kotlin 默认值

### J3 [未修·严重] bookSource.json 旧格式字符串规则源经 map_to 路径静默丢规则
- Kotlin: Jackson mapTo 抛 DecodeException（源整体失败可见）
- Rust: BookSource.rs:398-403 手写 Deserialize serde_json 失败 → .ok() → rule_toc/rule_content 静默 None → 书源"还在"但解析全失效（更隐蔽）
- SourceAnalyzer 路径支持旧格式 ✓（仅 map_to 路径差异）

### J4 [未修·中] bookChaptersCache 文件名不匹配（hashCode vs md5）→ 非书架书目录缓存 miss（自愈重新抓取）

### J5 [未修·中] invalidBookSourceCache 目录（无 ns 子目录）+ 文件名（md5 vs hashCode）双重不匹配 → 失效标记 miss（自愈）

### J6 [未修·中] bookInfoCache Rust 从不读磁盘（纯内存 LocalCache）→ 书籍信息缓存 miss（自愈）

### J7 [未修·中] runtimeCache 文件名不匹配（QueryTTF 等缓存 miss——自愈）

### J8 [未修·低] bookshelf durChapterTime 缺省 0 vs Kotlin now（"最近阅读"排序异常——仅旧文件）

### J9 [已确认] enabledCookieJar None vs false（等价）；respondTime 缺省 0 vs 180000（map_to 路径排序显示差异）

### J10 [未修·低] rssSources variableComment 读侧丢弃（再保存时注释丢失）

### J11 [提示] cookie 内容格式反向迁移：Rust 写带日期头 → Kotlin isDue 立即判过期删除（仅回迁场景）

**已确认兼容**: bookshelf.json 全部键（含 Jackson getter 键被忽略不丢数据）、bookmark/bookGroup/replaceRule/httpTTS/txtTocRule（include_str 同一文件 100% 一致）、rssArticles（不持久化）、bookCoverCache（md5+ext 一致）、users.json 键与密码哈希算法、书架内章节缓存路径与 BookChapter 13 键

---

## K. 安全边界（agent 4）

### K1 [未修·严重-严重] getUserInfo 通过未校验 accessToken 泄露任意用户实时 token（完全接管）
- Kotlin: getUserInfo 只读 session().get("username")（仅成功 checkAuth 写入）
- Rust: check_auth 失败后回退解析未校验的 `?accessToken=用户名:任意值` → getUserInfoClass(victim) → format_user 返回 accessToken = victim 真实当前 token → **用该 token 完全以 victim 身份调用所有 API**
- 验证: secure 模式 GET /reader3/getUserInfo?accessToken=alice:xxx → data.userInfo.accessToken = alice:<真实token>
- 修复: getUserInfo 的 accessToken 回退路径必须先校验 token

### K2 [未修·严重] getUserNameSpace 的 accessToken 回退 → 未认证跨用户读数据
- Kotlin: 回退 "default" 命名空间（无跨用户）
- Rust: BaseController.rs:376-382 回退解析 accessToken 用户名——所有调用 checkAuth 但不检查返回值的接口（getBookSources/getBookSource/explore_book/search_book/get_book_info/文件管理 __HOME__/WebDAV resolve_webdav_path）→ 未认证可把命名空间指向任意用户读数据
- 验证: GET /reader3/getBookSources?accessToken=alice:garbage → 返回 alice 私有书源
- 修复: get_user_name_space 的 accessToken 回退需校验

### K3 [未修·严重-Windows] 静态文件服务反斜杠路径穿越
- Rust: serve_static L106 只按 / 切分、只过滤字面 ..——`GET /assets/..\..\data\users.json` → Windows 下解析到 storage\data\users.json（全部用户密码哈希+salt+token）200 直读
- Linux 无影响；Windows 开发/部署环境可直读
- 修复: 同时按 \ 切分

### K4 [未修·高] multipart 字段名未清洗 → 认证前任意路径写文件
- Rust: parse_http_body L168 dir.join(format!("{}_{}", name, safe_name))——filename 清洗了但字段名 name 未清洗；写入发生在认证之前（所有 multipart POST 触发）
- 利用: name="..\..\..\任意目录\x" → storage\file-uploads\..\..\..\任意目录\x_<sanitized> 任意创建文件
- 修复: name 也做字符清洗

### K5 [已确认-两版相同] secureKey 为空时 userNS 覆盖形同虚设（任何已登录用户可带 ?userNS=victim 访问任意用户数据）
### K6 [已确认-两版相同] /book-assets/*、/epub/* 未认证直接暴露整个 storage/data 树（allowRootFileSystemAccess）
### K7 [已确认-两版相同] secureKey 明文非恒定时间比较
### K8 [已确认-两版相同] token 生成（MD5(MD5(user+ts)+ts)）/7 天过期/日志含 accessToken
### K9 [未修·低] WebDAV Basic 前缀大小写敏感（Rust 拒绝 `basic xxx` 小写——Kotlin 忽略大小写）

**已确认**: check_auth 覆盖范围两版路由表一致、控制器层路径穿越（%2e%2e/双编码/绝对路径均拦截）、上传白名单与 10MB 上限、logout 失效已修、会话 cookie 隔离

---

## L. 资源泄漏与性能（agent 5）

### L1 [未修·最高] download_images 泄漏 → 图片重试永久挂死（服务挂死+内存泄漏）
- Kotlin: try/catch/finally（finally 必 remove）
- Rust: BookHelp.rs:224-267 remove 只在成功路径；网络错误 panic（OkHttpUtils.rs:209）被 catch_unwind 吞掉 → 条目永久残留 → 下次同 src `while contains { delay(100).await }`（无超时）无限等待 → **单线程服务器永久卡死全站无响应**
- 修复: finally 语义（catch_unwind 包住 + 必 remove）

### L2 [未修·高] session_store 无界增长（无 7 天超时清理）
- Kotlin: LocalSessionStore 自带 reaper 定期清理
- Rust: vertx.rs:369-394 OnceLock<Mutex<HashMap>> 无 TTL 无清理；无 cookie 客户端每请求新建条目 → 数天数十万级
- 修复: 加过期清理

### L3 [架构] 单线程 HTTP + 全同步阻塞 handler（慢书源 45s 冻结全站）——同 E1
### L4 [未修·高] 章节/书籍缓存无字节上限
- LocalCache 仅 10k 条计数上限（无字节）；章节列表单条 ~600KB → 最坏 6GB；Kotlin ACache 2MB/5MB 磁盘上限
- 修复: put 时按值大小淘汰或加字节预算

### L5 [未修·高] Matcher::find 空匹配死循环 + fancy-regex 无执行步数限制（ReDoS）
- stubs.rs:276-292 find 空匹配 pos 不前进（Kotlin Java 前进 1 字符）；fancy-regex 回溯 VM 无步数/耗时上限（(a+)+ 指数爆炸）
- 书源规则可空匹配/嵌套量词 → 单线程服务器挂死（catch_unwind 救不了挂死）
- 修复: find 空匹配推进 + 超时/步数限制

### L6 [未修·中高] HTTP 连接不复用（每请求新建 reqwest Client——TIME_WAIT 堆积、端口压力；Kotlin 共享 okHttpClient 连接池）
### L7 [未修·中] send_file 整读内存（GB 级 ×2 峰值——WebDAV 大备份/EPUB 导出）
### L8 [未修·中] 用户级 Mutex 无 RAII（fetch_start 区间内 panic → locked 永不复位 → 之后所有 lock 无限自旋 CPU 100%）
### L9 [架构] block_on noop-waker 忙轮询（当前热路径即时 Ready；真实 pending future 单核 100%）
### L10 [未修·中低] WebSocket 每连接一线程 + 无连接超时 + 无界发送通道
### L11 [未修·低] STORAGE_LOCKS thread_local（跨线程互斥失效——服务器/worker/定时线程不共享锁）
### L12 [未修·低] ACache mInstanceMap 键不一致（absoluteFile vs absolutePath——每请求泄漏小实例）

**已确认无问题**: thread::spawn 全部 join 回收、zip/文件句柄 Drop 关闭、DOWNCAST_CELL/OnceLock 有界、定时任务线程独立、TTSService 30s deadline

---

## M. HTTP 协议响应头（agent 6）——对照 vert.x 3.8.5 反编译字节码

### M1 [未修·中高] 静态资源 Range/206 缺失
- Kotlin: StaticHandler rangeSupport=true → Accept-Ranges: bytes、Range → 206+Content-Range、越界 416
- Rust: serve_static 整读 200 全量
- 影响: EPUB 内 audio/video seek、断点续传、部分 WebDAV 客户端分段 GET
- 注: 动态 sendFile 接口两端都不支持 Range（vert.x-core sendFile 无 Range 解析——已实锤）

### M2 [未修·中] 405 分支不可达（/* 静态规则恒匹配 → matched 恒非空 → 405 逻辑被架空 → 404）
- Kotlin: 路径匹配方法不匹配 → 405
- Rust: server.rs:334-344 的 405 分支因 /* (method=None) 恒匹配而不可达 → 落静态 → 404
- 之前修的 405 实际没生效！需要重做判定（方法不匹配应先于 /* 判定）
- 验证: curl -I /reader3/getBookSource → Kotlin 405 vs Rust 404

### M3 [未修·中] 会话 Set-Cookie Max-Age 差 1000 倍 + SameSite 差异
- Kotlin: vert.x 3.8.5 把 ms 当秒写 → Max-Age=604800000（≈19 年）无 SameSite/HttpOnly
- Rust: 真实秒 Max-Age=604800 + SameSite=Lax（跨站 iframe/子域请求不带 cookie）
- 影响: 第三方客户端按 Max-Age 判过期不同；SameSite=Lax 影响嵌入场景
- 决策: 对齐 Kotlin（ms 当秒写）还是保留正确语义？

### M4 [未修·中低] 登出不下发删除 cookie（Rust session() 先续期再 destroy——响应仍带有效 cookie；remove_cookie 未被调用）
- 修复: logout 路径调用 remove_cookie

### M5 [未修·低中] 静态 Cache-Control 无 public 前缀/Last-Modified/Vary（CDN/反代共享缓存与 304 协商差异）
### M6 [未修·低中] 静态 text/html 无 charset（web/index.html 有 meta charset 缓解——simple-web 自建页有风险）
### M7 [未修·低] 302 Location 相对 vs 绝对（/simple-web——浏览器均正常）
### M8 [未修·低] CORS 预检 body/Content-Type/Set-Cookie 删除头差异（浏览器不读 body——无影响）
### M9 [决策] Date 头：Kotlin API 响应无 Date vs Rust 恒有（中间缓存 Age 计算差异——Rust 更规范）
### M10 [未修·低] 静态 MIME 差异（.js/.ttf/.woff——浏览器均接受）+ 500 JSON exception 恒空 vs Kotlin throwable.toString()

**已确认一致**: gzip 均不压缩、Server 头均不输出、Content-Length 自动、JSON charset 均有、WebDAV 响应头、Cache-Control 怪值（86400/300）原样复刻、CORS echo、头名大小写（HTTP/1.1 不敏感）、Content-Disposition 编码、错误页无 Content-Type

---

## N. 系统管理/杂项 API（agent 7）

### N1 [未修·高] 备份/下载/恢复接口阻塞整个 HTTP（pollster::block_on 单线程）
- 接口: backupToWebdav、user/downloadBackupFile、file/restore——zip 打包+WebDAV 数十秒期间所有其他请求挂起
- Kotlin: 协程挂起不阻塞
- 验证: 触发保存备份后立即 curl /getBookGroups——Rust 直到备份完成才返回

### N2 [未修·中高] --ui 模式 windowConfig 死代码（ReaderUIApplication.rs 转录完整但 main.rs 不接线——windowConfig.json 永不读写；--ui 打开 URL 不带 nopwa=1 → SW 注册差异）
### N3 [未修·中] /health 404（RestVerticle::start 从未被调用——连带 SessionHandler/Cookie 延长/LoggerHandler 全局 handler 未生效；auth 靠前端 axios 追加 accessToken 兜底）
- 修复: 注册 /health 路由（简单）
### N4 [已确认-两端一致] Mongo 备份/恢复静默无效（backup 本地读写、restore 路径双后缀错误——前端无入口）
### N5 [已确认] 版本号前端硬编码 v6.0.0（两版一致）；getSystemInfo 无 version 字段
### N6 [已确认] getSystemInfo 死接口（两端一致，fonts 恒 null）
### N7 [已确认] updateForce/SW（Rust 部署可用——一致）
### N8-N12 [已确认一致] 恢复默认书源（无备份——两端一致）、用户备份/恢复（backupFileNames 10 文件一致）、字体管理（上传/删除/列表一致）、TTS 配置（纯前端 localStorage）、系统清理/定时任务（deleteBookCache 一致）

---

## 第三轮修复优先级汇总

### P0（安全/数据丢失/挂死，必须修）
1. **K1/K2** — getUserInfo/getUserNameSpace 的未校验 accessToken 回退（token 泄露+跨用户越权）
2. **L1** — download_images finally（图片重试永久挂死）
3. **L5** — Matcher::find 空匹配死循环 + 正则超时（服务挂死）
4. **K3** — Windows 反斜杠路径穿越（读 users.json）
5. **K4** — multipart 字段名清洗（认证前任意写文件）
6. **J2** — users.json 缺字段默认值（旧数据迁移后功能被禁）
7. **I1/I2** — EPUB 导出 container.xml 0 字节 + mimetype 压缩（导出 epub 打不开）
8. **H1-H5** — JS 引擎（annex-b/substr、toLocaleString polyfill、Promise 悬置、book 方法、panic→Err）

### P1（功能错误/泄漏）
9. **M2** — 405 判定重做（当前被 /* 架空）
10. **M4** — logout remove_cookie
11. **L2/L4** — session_store 清理 + 缓存字节上限
12. **H6-H13** — JS 结果字符串化差异
13. **J3** — bookSource 旧格式 map_to 路径
14. **I3/I4** — 导出缓存非法 UTF-8 + exportCharset 绑定
15. **L6/L8** — 连接复用 + Mutex RAII
16. **N1** — 备份阻塞（架构级——同 E1/C6）

### P2（打磨/决策）
17. H14/H15、I5-I10、J4-J11、K9、M1/M3/M5-M10、N2/N3、L7/L10-L12

### 已确认（无需处理）
- 迁移兼容大项（bookshelf/bookmark/bookGroup/replaceRule/httpTTS/txtTocRule/书封面缓存/密码哈希）
- 安全一致项（check_auth 覆盖/上传白名单/控制器层穿越拦截）
- 系统管理一致项（备份集/字体/TTS/SW/默认书源）
- 前端契约全部对齐

---

# 第四轮深度差异排查报告（ROUND 4）

> 7 个方向排查完成于 2026-08-17。本轮覆盖：
> - **O 章节**：变量系统深链（`{{}}`、`@get:`、`@put:`、`@js:` 解析、传递、作用域与持久化）
> - **P 章节**：TTS 完整实现（Edge TTS、HttpTTS、textToSpeechCn、WebSocket 与音频格式）
> - **Q 章节**：本地书格式深链（EPUB / UMD / CBZ / PDF / TXT 编码、目录、正文与边角）
> - **R 章节**：前端交互与渲染细节契约（Web 组件、阅读设置、SSE 流、漫画、字体）
> - **S 章节**：WebSocket 与长连接（出站 TTS 客户端、事件循环与保活机制）
> - **T 章节**：错误处理、重试策略与用户反馈（响应结构、状态码、异常传播、静默失败）
> - **U 章节**：性能热点与算法复杂度（HTTP 连接池、正则预编译、DOM 反复解析、JSON 查重、Tokio 阻塞）
>
> 状态标记：`[未修·严重/P0]` / `[未修·高/P1]` / `[未修·中/P2]` / `[架构]` / `[决策]` / `[已确认]`

---

## O. 变量系统深链

### O1 [未修·严重] `AnalyzeRule::new` 使用占位数据丢弃 `ruleData` 实体，导致 `book()` 恒 `None` 且 `@put:` 无法写入书籍实体
- **Kotlin**: `AnalyzeRule.kt:33-37` 构造函数接收 `var ruleData: RuleDataInterface`，`val book get() = ruleData as? BaseBook` 直接持有传入的 `Book` 或 `SearchBook` 实体引用。在解析过程中遇到 `@put:` 时，`put()` 直接调用 `book?.putVariable(key, value)` 将变量写入书籍并在内存和持久化层生效。
- **Rust**: `stubs.rs:8443-8470` 的 `AnalyzeRule::new` 构造器中硬编码 `rule_data: Box::new(AnalyzeRulePlaceholderData)`。`AnalyzeRule.rs:79-84` 的 `pub fn book(&self)` 通过 downcast 检查 `self.rule_data` 恒返回 `None`！当解析 `@put:` 时（`AnalyzeRule.rs:501-508`），变量仅写入 `AnalyzeRule` 局部的 `book_variables` HashMap 和 `AnalyzeRulePlaceholderData` 的空实现中，传入的 `Book` 实体从未被写入，后续落盘保存（`bookshelf.json`）时 `book.variable` 仍为空。
- **影响**: 所有依赖 `@put:` 提取并保存书籍变量（如动态 token、防盗链签名、加密参数）的书源，在解析完成后变量全部丢失；下次请求无法读取。
- **修复建议**: `AnalyzeRule` 改为真实保存传入的 `rule_data` 并在 `put` 时回写到实体。

### O2 [未修·严重] `Book` 与 `SearchBook` 的 `variable_map()` 恒返回静态空 Map，导致 `get_variable()` 恒 `None`
- **Kotlin**: `Book.kt:96-107` 与 `BookChapter.kt:29-41` 中，`variableMap` 由 `variable` JSON 反序列化延迟初始化，`getVariable(key)` 从 `variableMap` 中读取，`putVariable(key, value)` 写入 Map 并同步序列化回 `variable` 字段。
- **Rust**: `stubs.rs:9869-9873`（Book）与 `stubs.rs:9185-9189`（SearchBook）的 `RuleDataInterface` 实现中，`variable_map` 恒返回静态空 Map；`RuleDataInterface.rs:11-13` 默认 `get_variable` 实现调用 `variable_map()`，导致通过 `RuleDataInterface` 或 `BaseBook` 读取 `Book` 变量时恒返回 `None`。
- **影响**: 即使 `Book.variable` 字段有值（例如已持久化到 `bookshelf.json`），在规则引擎或 JS 中通过 `ruleData.getVariable(key)` 读取时永远读取不到。
- **修复建议**: 在 `impl RuleDataInterface for Book` 和 `SearchBook` 中重写 `get_variable`，直接解析 `self.variable` 获取。

### O3 [未修·高] `WebBook.rs` 各阶段（搜索/发现/详情/目录/正文）所有权转移以克隆副本传递，导致阶段间变量传递中断
- **Kotlin**: `WebBook.kt:48-78`（搜索）：`val variableBook = SearchBook()` 传入 `AnalyzeUrl` 和 `BookList.analyzeBookList`，`AnalyzeUrl` 解析 URL 时 `@put:` 写入的变量，在 `BookList` 解析列表时实时可见，并传递给每一个 `SearchBook`。
- **Rust**: `WebBook.rs:119, 148, 192, 215, 253, 331, 389` 中传给 `AnalyzeUrl` 的是 `Some(Box::new(variable_book.clone()))` 克隆副本。`init_url()` 内执行 `@js:` 或 URL `@put:` 产生的变量全被写入了克隆的 Box 中，而传给 `BookList` 的 `&variable_book` 仍然为空。
- **影响**: 搜索 URL、发现 URL 或详情页 URL 中通过 `@put:` 或 `@js: java.put()` 提取的变量，无法传递到列表解析器、详情解析器或正文解析器中。
- **修复建议**: 统一 `RuleData` 生命周期引用，或在 `AnalyzeUrl` 执行完成后将产生的变量同步回主对象。

### O4 [未修·严重] JS 引擎中缺失 `java.put` 且 `java.get` 被劫持为 HTTP GET（与变量系统冲突）
- **Kotlin**: `AnalyzeRule.kt:35` 实现 `JsExtensions`，`evalJS` 时注入 `bindings["java"] = this`。支持 `java.put(key, value)`（变量存入）、`java.get(key)`（变量读取）、`java.get(url, headers)`（HTTP 请求）。
- **Rust**: `src/runtime/js.rs:884-942` 中 `java.put` 完全未注册（调用必抛 `TypeError: java.put is not a function`）；`java.get` 被硬编码绑定为 `java_get_native`（HTTP GET），在 JS 中调用 `java.get("token")` 时会将 `"token"` 当作 URL 发送 HTTP 网络请求。
- **影响**: 大量在 JS 规则块中使用 `java.put()` 或 `java.get()` 的书源 100% 报错崩溃或发错请求。
- **修复建议**: 在 `js.rs` 注册 `java.put`；在 `java.get` 中根据参数特征区分变量读取与 HTTP 请求。

### O5 [未修·高] JS 绑定的 `book` / `chapter` / `source` 对象缺少 `putVariable` / `getVariable` 等实例方法
- **Kotlin**: `AnalyzeRule.kt:649-653` 绑定的 `source`, `book`, `chapter` 具备 `book.getVariable('key')`、`book.putVariable('key', 'val')`、`source.getKey()` 等实例方法。
- **Rust**: `js.rs:48-50` 将它们序列化为普通 JSON Object 传入 Boa 引擎，未注入 prototype 实例方法。
- **影响**: 进阶书源在详情/正文 JS 中读写书籍级或章节级变量时报错 `TypeError: book.getVariable is not a function`。
- **修复建议**: 在 JS 初始化脚本中为 `book`、`chapter`、`source` 注入 prototype 包装方法。

### O6 [未修·高] `RuleAnalyzer` 字节与字符索引混用导致中文字符串切片 panic 或错切
- **Kotlin**: 全程使用 UTF-16 字符（Char/CodePoint）计数与切片。
- **Rust**: `RuleAnalyzer.rs` 中 `consume_to` 使用字节索引，`consume_to_any` 使用字符计数 `chars().nth(pos)`，最终切片 `queue[start..pos]` 又直接按字节索引切片。当规则中包含中文字符时，触发 `byte index X is not a char boundary` panic。
- **修复建议**: `RuleAnalyzer` 内部统一基于字符边界或 `Vec<char>` 处理。

### O7 [未修·中] `AnalyzeRule` / `AnalyzeUrl` 变量作用域优先级与持久化生命周期不一致
- **Kotlin**: 3 级链：`chapter?.putVariable(...) ?: book?.putVariable(...) ?: ruleData.putVariable(...)`。
- **Rust**: `AnalyzeRule.rs:501-508` 的 `put` 跳过了 `Book` 实体；`get` 优先级倒置；多用户并发锁粒度缺少 userNS。
- **修复建议**: 对齐 Kotlin 3 级作用域链，确保所有写操作最终同步到实体对象的 `variable` 字段。

### O8 [未修·低] `eval_js` 中非 Book 时 `book` 绑定类型差异（string vs null）
- **Kotlin**: 当 `ruleData` 不是 `Book` 时，`bindings["book"] = null`。
- **Rust**: `AnalyzeUrl.rs:461` 绑定为命名空间字符串（如 `"admin"`）。
- **修复建议**: 改为绑定 `crate::stubs::Any::Null`。

---

## P. TTS 完整实现

### P1 [未修·严重] `BookController.rs` TTS 音频响应经 UTF-8 Lossy 损坏（MP3 100% 播放失败）
- **Kotlin**: `BookController.kt:3140, 3167` 使用二进制安全的 Vert.x Buffer 直接输出字节流。
- **Rust**: `BookController.rs:4172, 4221` 使用 `r.end(crate::stubs::io::vertx::Buffer::new(audio_bytes).to_string())`，将 MP3 二进制字节按 UTF-8 lossy 转码为 String，非 UTF-8 字节被破坏为 `\u{FFFD}`（`0xEF 0xBF 0xBD`）。
- **影响**: Web 前端或外部客户端通过 `/reader3/book/tts?type=edge&text=...` 请求音频流时，返回的 MP3 数据 100% 损坏，HTML5 `<audio>` 播放器解码报错。
- **修复建议**: 为 `ResponseHandle` 增加 `end_bytes(Vec<u8>)` 方法，直接发送原始二进制数据。

### P2 [未修·严重] `ByteString::last_index_of` 与 Edge TTS 帧解析逻辑缺陷导致垃圾字节混入音频流
- **Kotlin**: `TTSService.java:79-85` 正确寻找最后一个匹配项并判断索引非 -1。
- **Rust**: `stubs.rs:6146-6155` 中 `ByteString.last_index_of` 使用了 `.position()`（前向搜索）而非 `.rposition()`；`TTSService.rs:147-148` 在返回 `-1` 时直接做 `+12` 得到 `11`，使得 `audio_index != -1` 恒为 `true`。
- **影响**: Edge TTS 收到控制帧或元数据帧时，将非音频数据误拼入音频流，造成音频头部或中间混入噪声。
- **修复建议**: `ByteString.last_index_of` 改为 `.rposition()`，并在加偏移前先判别 `-1`。

### P3 [未修·高] `BookController.rs` `tts_by_api` 语速参数硬编码为 0（忽略前端 `rate` 配置）
- **Kotlin**: `BookController.kt:3155` 计算 `speechRate = (5 + (rate - 0.5) * 30).toInt()` 并传入。
- **Rust**: `BookController.rs:4193` 将 `speech_rate` 硬编码为 `0`。
- **影响**: 前端调节 HttpTTS 语速滑块完全无效。
- **修复建议**: 补充语速计算公式并传入 `get_speak_stream`。

### P4 [未修·高] `HttpTTS` `get_speak_stream` 缺失 `AnalyzeUrl` 规则解析、`loginCheckJs` 与重试机制
- **Kotlin**: `BookController.kt:3230-3289` 使用 `AnalyzeUrl` 完整支持 GET/POST、`{{speakText}}`、`@js:` 签名、`loginCheckJs` 校验、`Content-Type` 错误检测与 5 次重试。
- **Rust**: `BookController.rs:4300-4333` 仅做简单字符串替换，只支持 GET，无 JS 签名，无重试。
- **影响**: 复杂第三方 HttpTTS 无法使用。
- **修复建议**: 改用 `AnalyzeUrl` 驱动请求并补齐重试与类型检测。

### P5 [未修·高] `BookController.rs` TTS 异常处理器未关闭 HTTP 响应导致连接永久挂起
- **Kotlin**: `BookController.kt:3104` 异常时调用 `response.setStatusCode(404).end()`。
- **Rust**: `BookController.rs:4117` 异常闭包内创建了 response 副本但从未调用 `set_status_code` / `end`。
- **影响**: TTS 发生异常时前端连接挂死直到 30s 超时。
- **修复建议**: 补充 `r.set_status_code(404).end(String::new())`。

### P6 [架构·中] Edge TTS 每次请求重建 WebSocket 连接与 Tokio Runtime
- **Kotlin**: 单例复用 WebSocket 长连接，仅断开时重建。
- **Rust**: 每次请求新建单线程 Tokio Runtime 并建立全新 WebSocket 连接。
- **修复建议**: 实现全局 `TTSService` 单例复用长连接。

### P7 [已确认] `textToSpeechCn` 接口请求与 302 重定向行为两端一致

---

## Q. 本地书格式深链

### Q1 [未修·P0 极高] TextFile 非 UTF-8 编码（GBK / Big5 / UTF-16）章节偏移漂移与正文错位
- **Kotlin**: `TextFile.kt:143,209` 使用 `chapterContent.toByteArray(charset).size` 计算章节在原始字符集下的物理字节跨度。
- **Rust**: `TextFile.rs:177,245` 使用 `chapter_content.as_bytes().len()`（UTF-8 字节长度）。
- **影响**: 对 GBK / Big5 / UTF-16 编码小说，计算出的章节 start/end 字节偏移严重漂移，阅读时 `bis.skip` 跳至错误位置造成乱码、截断或空内容。
- **修复建议**: 按文件字符集重新编码字符串以获取准确字节长度。

### Q2 [未修·P0 极高] UmdReader::read 中 `std::mem::replace` 导致 `prev.unwrap()` 100% Panic 瘫痪
- **Kotlin**: 直接使用传入的 `InputStream` 构造 `StreamReader`。
- **Rust**: `UmdReader.rs:46` 用 `std::mem::replace` 替换 `None` 初值后调用 `prev.unwrap()`，100% 触发 panic。
- **影响**: UMD 格式书籍导入、预览、解析目录与正文完全瘫痪崩溃。
- **修复建议**: 删除 `std::mem::replace`，直接使用传入的 `input_stream`。

### Q3 [未修·P1 高] UmdUtils::unicode_bytes_to_string 遇到非法/代理区字节时 `.unwrap()` Panic
- **Kotlin**: `(char) c` 强转容错。
- **Rust**: `UmdUtils.rs:77` 调用 `char::from_u32(c as u32).unwrap()`。遇到 UTF-16 代理区码点时 panic。
- **修复建议**: 改为 `unwrap_or('\u{FFFD}')` 或 `std::char::decode_utf16`。

### Q4 [未修·P1 高] EncodingDetectHelp 文件打开与空文件 `.unwrap()` / 越界 Panic
- **Rust**: `EncodingDetectHelp.rs:146,153,172` 直接调用 `File::open().unwrap()`，未检查空文件直接读 `temp_byte[0]`，负数索引直接当下标。
- **修复建议**: 加 match 容错与长度/索引范围检查。

### Q5 [未修·P1 高] EpubFile / UmdFile 静态单例 `static mut` 无并发互斥保护
- **Kotlin**: 使用 `@Synchronized` 保证并发安全。
- **Rust**: `EpubFile.rs:81` 与 `UmdFile.rs:57` 使用裸 `pub static mut` 和 `unsafe` 读写，多请求并发时产生 Data Race 与实例覆盖。
- **修复建议**: 参照 `CbzFile.rs` 改用 `Mutex<Option<...>>`。

### Q6 [未修·P2 中] FileController 本地书导入 `root_dir` 引起 `storage/storage` 双重路径
- **修复建议**: 将 `FileController.rs:672` 改为 `get_work_dir("", vec![])`。

### Q7 [未修·P2 中] ZipFile / ResourcesLoader 解压单条目重复解析 Central Directory I/O 放大
- **修复建议**: 持有 `Arc<Mutex<ZipArchive<File>>>` 句柄避免单条目重复全量打开。

### Q8 [未修·P2 中] 本地书异常分类缺失与损坏文件静默入库
- **修复建议**: 区分 `TocEmptyException` 与 `CorruptedFile`。

### Q9 [已确认一致] EPUB HTML 标签清洗规则、CBZ 漫画元数据、PDF 双模式切分一致

---

## R. 前端交互与渲染细节契约

### R1 [未修·严重] 鉴权与权限拦截错误 `isSuccess` 状态倒置（NEED_LOGIN / NEED_SECURE_KEY）
- **Kotlin**: `CURD.kt:92` 返回 `{"isSuccess": false, "errorMsg": "请登录后使用", "data": "NEED_LOGIN"}`。
- **Rust**: `CURD.rs` 与 `UserController.rs` 内部调用 `set_data` 将 `is_success` 置为 `true`，返回 `{"isSuccess": true, "errorMsg": "请登录后使用", "data": "NEED_LOGIN"}`。
- **影响**: 前端 `axios.ts` 仅在 `!isSuccess` 时才弹登录/安全密码框；`isSuccess: true` 导致前端误判为成功，弹绿色 Toast 或渲染错误字符串，未登录拦截全面失效。
- **修复建议**: 在认证拦截与 `set_data_owned` 返回特定错误码时明确令 `is_success = false`。

### R2 [未修·严重] cacheBookSSE 伪流式响应与事件流全量缓冲阻塞
- **Kotlin**: `response.setChunked(true)` 实时发送每章进度事件流。
- **Rust**: `HttpResponse` 将 `write()` 缓存在内存中，全部章节抓取完毕后才一次性返回，且在此期间单线程事件循环被完全阻塞。
- **影响**: 前端进度条卡死在 0% 直至最后跳 100% 或 30s 超时；全站其他 HTTP 请求被冻结。
- **修复建议**: 引入真实 Axum SSE 流式响应并异步处理。

### R3 [未修·高] `saveBookConfig` 仅更新 pdfImageWidth 且缺少通用配置合并
- **影响**: 前端在阅读界面配置的段落重排、替换规则开启状态等无法持久化到 `Book.readConfig`。
- **修复建议**: 扩展 `save_book_config` 支持 `readConfig` 完整 JSON 对象合并。

### R4 [未修·高] 漫画/图片正文 `__API_ROOT__` 渲染与懒加载代理丢失
- **影响**: 跨域部署或反向代理下漫画和插图直连触发 403 裂图。
- **修复建议**: 补齐 `save_images` 并对齐图片 URL 代理重写逻辑。

### R5 [未修·中] 前端书源失效检测 (`checkBookSource`) 契约传参失配与本地状态未同步
- **修复建议**: 前端与后端统一在失败时将错误书源标识记录并返回标准错误。

### R6 [未修·中] 字体管理二进制传输与自定义字体 CSS 注入差异
- **修复建议**: 对字体文件名进行 URL 解码并补齐字体 MIME 类型。

### R7 [未修·中低] 书架/进度同步时间戳精度与 WebDAV 远端冲突解决机制
- **修复建议**: 确保 `save_book_progress` 始终注入当前系统毫秒时间戳。

### R8 [未修·中低] 批量操作 (`CURD.rs` / `BookManage.ts`) 反序列化失败熔断机制缺失

---

## S. WebSocket 与长连接

### S1 [未修·P0 极高] `ws_loop` 中 `async { rx.try_recv().ok() }` 导致 20ms 忙轮询与 CPU 浪费
- **Kotlin**: OkHttp 异步 WebSocket，空闲时处于系统 I/O 阻塞休眠状态，CPU 为 0。
- **Rust**: `stubs.rs:6234-6270` 在 `tokio::select!` 中使用 `async { rx.try_recv().ok() }` + `sleep(20ms)`。
- **影响**: 空闲时每秒被唤醒 50 次，持续产生无意义的上下文切换和 CPU 开销。
- **修复建议**: 通道改为 `tokio::sync::mpsc`，使用 `rx.recv().await` 实现真正的事件驱动挂起。

### S2 [未修·P1 高] OkHttpClient::new_web_socket 每连接泄露独立 OS 线程与 Tokio Runtime
- **影响**: 每次重试都会创建独立的 OS 线程和单线程 Runtime，增加线程栈内存开销。
- **修复建议**: 复用全局 Tokio Runtime 派生异步任务。

### S3 [未修·P1 高] `ping_interval` 为空壳桩，无心跳保活导致长连接静默断开
- **影响**: TTS 长连接空闲超过 30~60 秒后被服务端静默掐断，下次合成必报错。
- **修复建议**: 在 `ws_loop` 中加入 20s 周期性 Ping 保活心跳。

### S4 [未修·P2 中] TTSService `synthesising` 标志与超时保护完善
- **修复建议**: 加入 RAII 守护确保 `synthesising = false` 必被重置。

### S5 [未修·P2 中] TTS 二进制音频流多重内存克隆与缓冲开销
- **修复建议**: 将 `ByteString` 内部存储改为 `bytes::Bytes` 零拷贝切片。

### S6 [已确认一致] 确认全站无服务端 WebSocket 监听端点，仅出站 TTS 客户端长连接

---

## T. 错误处理、重试策略与用户反馈

### T1 [未修·严重] 500 异常响应缺少 `errorMsg` 字段导致前端静默吞错
- **Kotlin**: `VertRoute.kt:20` 返回包含 `message` 的错误结构。
- **Rust**: `server.rs:384` panic 兜底返回 `{"message":"服务器内部错误"}` 但**无 `errorMsg` 字段**。
- **影响**: 前端 `axios.ts:154` 依赖 `res.errorMsg` 弹窗，500 时 `errorMsg` 为 `undefined`，导致前端**完全不弹错误提示，用户点击毫无反应（静默失败）**。
- **修复建议**: 在 `server.rs` 的 500 响应中补齐 `"errorMsg": "服务器内部错误"` 及 `"isSuccess": false`。

### T2 [未修·严重] CURD 单条操作空请求体/非法 JSON 直接 panic 引发 500
- **Kotlin**: 捕获 `DecodeException` 返回 `ReturnData().setErrorMsg("参数错误")`。
- **Rust**: `CURD.rs:139, 226` 对 `context.body_as_json().unwrap()` 直接 unwrap。空请求体直接崩溃。
- **修复建议**: 改为 match 处理并在 None 时安全返回错误提示。

### T3 [未修·高] HTTP 跨模块调用异常栈与原始错误信息丢失
- **修复建议**: 使用 `thiserror` 保留错误分类与链路信息。

### T4 [未修·高] 正文抓取与换源网络超时缺少渐进重试与熔断降级
- **影响**: 网络抖动时正文加载失败率上升，单个死链书源阻塞 45 秒。
- **修复建议**: 在 `okhttp.rs` / `WebBook.rs` 增加 2~3 次轻量重试机制。

### T5 [未修·中] WebDAV 错误状态码映射不全 (401/404/409/412 vs 500)
- **修复建议**: 对齐 RFC 4918 WebDAV 标准 HTTP 状态码。

### T6 [未修·中] 文件上传与静态资源 404/413 响应结构不符合 ReturnData 契约
- **修复建议**: API 路由下的 404/405/413 错误统一输出 JSON 格式。

### T7 [未修·低] TTS 音频异常静默输出 200 乱码字节而非显式错误
- **修复建议**: TTS 失败时显式返回非 200 状态码或标准 ReturnData JSON。

---

## U. 性能热点与算法复杂度

### U1 [未修·严重] HTTP 请求每调用新建 OS 线程与 TLS 客户端（长连接与连接池彻底失效）
- **Kotlin**: 全局共享单例 `OkHttpClient`（5 空闲连接，5 分钟 Keep-Alive，共享 DNS 缓存）。
- **Rust**: `src/runtime/okhttp.rs:14-51` 每次请求都通过 `std::thread::spawn` 裸起 OS 线程并 `reqwest::blocking::Client::builder().build()` 新建客户端。
- **影响**: 彻底丧失 HTTP Keep-Alive，每次重新加载根证书并执行 TCP 握手 + TLS 协商；高并发搜索或批量抓取时瞬间创建数百个 OS 线程，延迟上升 5~10 倍并容易耗尽句柄。
- **修复建议**: 使用全局单例/延迟初始化的 `reqwest::blocking::Client`，移除每次请求裸起线程的设计。

### U2 [未修·严重] 正则表达式每次动态编译（伴生常量转关联函数引发 CPU 暴增）
- **Kotlin**: `PUT_RULE`、`GET_RULE` 等在伴生对象中作为 `val` 常量仅编译一次。
- **Rust**: `AnalyzeRule.rs:1004-1015` 将它们写为函数，每次调用都执行 `Pattern::compile_with`。
- **影响**: 遍历书籍列表或目录时，每项重复编译正则数千次，CPU 消耗暴增。
- **修复建议**: 使用 `std::sync::LazyLock` 将常用正则表达式预编译为静态单例。

### U3 [未修·严重] 规则引擎 DOM 树与 HTML 字符串反复 O(N*M) 往复解析与堆分配
- **Kotlin**: Jsoup 在内存中维护完整节点树，子选择器通过指针遍历，零重复序列化。
- **Rust**: `Element` 仅持有 `html: String`。多级规则链中，上一级生成的子 HTML 字符串被下一级重新 `parse_fragment` 数千次。
- **影响**: 2,000 章目录页触发 4,000+ 次完整 DOM 树构建与析构，严重拖慢解析速度。
- **修复建议**: 重构 `Element` 使其在规则链执行期间持有解析后的 DOM 引用。

### U4 [未修·高] DB 存储层线性扫描中的 O(N^2) 重复 JSON 反序列化
- **Rust**: `DB.rs:163-171, 203-211` 在 `save` 和 `save_multi` 查重循环中，每次迭代均重复调用 `serde_json::from_str(&json)`。
- **影响**: 批量保存 100 条书源时触发 100,000 次 JSON 解析，导致秒级卡顿。
- **修复建议**: 将 `&json` 的反序列化移至循环外。

### U5 [架构·高] 单线程 Tokio Runtime 阻塞式 IO 导致全站事件循环冻结
- **影响**: 任何耗时较长的同步操作会导致整个 HTTP 服务器主事件循环完全停摆。
- **修复建议**: 重型计算与阻塞 IO 逐步派发至阻塞线程池或改造为异步。

### U6 [未修·中] 大文本/章节解析中的大量 String Clone 与无预分配内存碎片
- **修复建议**: 传参引入 `&str` / `Cow<str>`，并在已知容量场景预分配内存。

### U7 [未修·中] ACache 缓存读写频繁 I/O 与缺少内存级 L1 缓存
- **修复建议**: 为 ACache 补充轻量级内存 LRU 缓存层。

---

## 第四轮修复优先级汇总（按严重度）

### P0（阻断/严重缺陷/数据丢失/服务崩溃）
1. **O1** — `AnalyzeRule::new` 丢弃 `ruleData`，导致 `book()` 恒空且 `@put:` 无法持久化写入书籍
2. **O2** — `Book`/`SearchBook` 的 `variable_map` 恒返回空 Map，导致 `get_variable` 恒空
3. **P1** — `Buffer::to_string()` UTF-8 Lossy 损坏二进制 MP3 音频流（TTS 播放 100% 失败）
4. **Q1** — `TextFile` 非 UTF-8 编码按 UTF-8 计算章节长度，导致正文字节偏移全盘漂移错位
5. **Q2** — `UmdReader::read` 中 `prev.unwrap()` 导致 UMD 解析 100% Panic 崩溃
6. **R1** — 鉴权拦截错误 `isSuccess` 状态倒置（`NEED_LOGIN` 返回 `true` 导致登录拦截失效）
7. **S1** — `ws_loop` 中 `async { rx.try_recv() }` 产生 20ms 忙轮询与无意义 CPU 占用
8. **T1** — 500 响应缺少 `errorMsg` 字段，导致前端所有异常静默吞错
9. **T2** — `CURD.rs` 空请求体 `.unwrap()` 直接 panic 升级为 500
10. **U1** — HTTP 请求每调用新建 OS 线程与 TLS 客户端（长连接池失效、线程创建开销巨大）
11. **U2** — 正则表达式伴生常量转关联函数导致每次重复编译（CPU 暴增）
12. **U3** — 规则引擎 DOM 树与 HTML 字符串反复 $O(N \times M)$ 往复解析与堆分配

### P1（高危/功能异常/资源泄露）
13. **O4** — JS 引擎缺失 `java.put` 且 `java.get` 被误作为 HTTP GET
14. **O3** — `WebBook.rs` 各阶段传递克隆副本导致变量链断裂
15. **O5** — JS 中 `book`/`chapter`/`source` 缺少 `getVariable`/`putVariable` 等实例方法
16. **O6** — `RuleAnalyzer` 字节与字符索引混用导致中文规则切片 panic
17. **P2** — `ByteString::last_index_of` 与 Edge TTS 帧解析逻辑缺陷（垃圾字节混入）
18. **P3** — `tts_by_api` 语速参数写死为 0（忽略前端设置）
19. **P4** — `HttpTTS` `get_speak_stream` 缺失 `AnalyzeUrl` 与校验重试机制
20. **P5** — TTS 异常处理器未关闭 HTTP 响应导致客户端挂死
21. **Q3** — `UmdUtils` 遇到非标/代理区双字节时 `from_u32().unwrap()` 崩溃
22. **Q4** — `EncodingDetectHelp` 文件打开与空文件 `temp_byte[0]` 未加防守 Panic
23. **Q5** — `EpubFile`/`UmdFile` 全局 `static mut` 裸访问导致多请求数据竞争与 UB
24. **R2** — `cacheBookSSE` 伪流式响应全量缓冲阻塞
25. **R3** — `saveBookConfig` 缺少通用排版配置合并
26. **R4** — 漫画/插图正文 `__API_ROOT__` 渲染与懒加载代理丢失
27. **S2** — `new_web_socket` 每次新建独立 OS 线程与 Tokio Runtime
28. **S3** — 缺乏周期性 Ping 心跳导致 TTS 长连接空闲静默断开
29. **T3** — HTTP 跨模块调用异常栈与原始错误信息丢失
30. **T4** — 正文抓取与换源网络超时缺少渐进重试与熔断降级
31. **U4** — DB 存储层线性扫描中的 $O(N^2)$ 重复 JSON 反序列化
32. **U5** — 单线程 Tokio Runtime 阻塞式 IO 导致全站事件循环冻结

### P2（打磨/边界优化/架构优化）
33. **O7** / **O8** — 变量作用域优先级与非 Book 上下文 `book` 绑定类型
34. **P6** — Edge TTS 每次请求重建 WebSocket
35. **Q6**~**Q8** — 本地书 `storage/storage` 路径、ZipFile I/O 放大、损坏文件静默入库
36. **R5**~**R8** — 失效书源检测传参、字体管理、时间戳精度、批量操作原子性
37. **S4** / **S5** — TTS `synthesising` 超时复位、音频 Vec 内存切片优化
38. **T5**~**T7** — WebDAV 状态码对齐、静态 404 JSON 契约、TTS 显式错误
39. **U6** / **U7** — 字符串预分配与 ACache 内存 L1 缓存

### 已确认一致项（无需处理）
- **P7** — `textToSpeechCn` 接口请求与 302 重定向行为两端一致
- **Q9** — EPUB 标签清洗、CBZ 漫画信息抽取、PDF 双模式切分一致
- **S6** — 确认全站无服务端 WebSocket 监听端点，仅出站 TTS 客户端长连接



---

## V. 第 5 轮深度差异排查报告（ROUND 5，2026-08-17）

### V1 [已修·P0 严重] 全文检索 `searchBookContent` UTF-8 字节索引与切片 Panic 崩溃
- **Kotlin**: `BookController.kt:2831-2869` 使用 UTF-16 字符维度 `indexOf` 逐字步进，`po1 = queryIndex - 20`, `po2 = queryIndex + query.length + 20` 截取前后 20 个字符。
- **Rust**: `BookController.rs:3753, 3779` + `stubs.rs:1104`：`index_of` 返回字节偏移量，`(index + 1)` 直接落入中文字符内部导致 `byte index is not a char boundary` panic；`substring_range` 字节切片直接越界 panic。
- **修复**: 重构为基于 Unicode `char` 维度的安全匹配与安全切片，前后截取精确为 20 个汉字/字符，彻底杜绝切片 Panic。

### V2 [已修·P0 严重] 书架部分刷新时列表排序严重错乱（未更新书前置，更新书全部沉底）
- **Kotlin**: `BookController.kt:1687-1721` 并发刷新但保持书架原始索引顺序一致。
- **Rust**: `BookController.rs:2436-2483` 将不更新的书直接先 push 进 `book_list`，刷新的书籍在 futures 完成后被追加在末尾，导致刷新的网络书被移至书架最后。
- **修复**: 采用固定大小槽位按原始书架索引就地更新，严格保持原始书架顺序（100% 稳定）。

### V3 [已修·P0 严重] `BookChapterList.rs` 中 `chapter` 被 `take().unwrap()` 导致解析规则执行时 `chapter` 恒为 `None`
- **Kotlin**: `BookChapterList.kt:183-188` `analyzeRule.chapter = bookChapter` 全程保留引用，支持 `@put:` 与 JS 访问 `chapter`。
- **Rust**: `BookChapterList.rs:218-228` `analyze_rule.chapter.take().unwrap()` 将其置空，导致字段规则执行时 `chapter` 恒为 `None`，`@put:` 错误存入全局变量。
- **修复**: 移除错误的 `take()`，在字段提取完成后统一回写 `book_chapter`。

### V4 [已修·P0 严重] `java.cacheFile` 返回值语义完全颠倒（返回路径 vs 文本内容）
- **Kotlin**: `JsExtensions.kt:148-159` 返回下载并缓存的**文本内容**。
- **Rust**: `src/runtime/js.rs:616-633` 返回了本地文件路径（`"storage/cache/js/xxx.bin"`），导致 JS 执行 `eval(java.cacheFile(url))` 时报 SyntaxError。
- **修复**: 改为返回缓存的文本内容（`String`）。

### V5 [已修·P1 高] `java.timeFormat` 签名错位导致 1 参调用恒返回空串
- **Kotlin**: `JsExtensions.kt:310-312` 1 参模式默认格式为 `"yyyy/MM/dd HH:mm:ss"`。
- **Rust**: `src/runtime/js.rs:421-437` 仅支持 3 参，1 参调用时格式为空串导致解析失败。
- **修复**: 支持 1 参（默认格式）、2 参及 3 参通用解析，默认时区 +8 区。

### V6 [已修·P1 高] `AnalyzeByRegex::get_element` unwrap panic
- **Kotlin**: `AnalyzeByRegex.kt:18-20`。
- **Rust**: `AnalyzeByRegex.rs:23` 对未命中的可选捕获组直接 `.unwrap()` 导致 panic。
- **修复**: 替换为 `unwrap_or_default()`（与 `get_elements` 保持一致）。

### V7 [已修·P1 高] Windows 下 `ZipUtils.rs` 打包 ZIP Entry 包含反斜杠 `\` 破坏跨平台恢复
- **Kotlin**: `ZipUtils.kt:146`。
- **Rust**: `ZipUtils.rs:149-151` Windows 下 `File::separator()` 返回 `\`，导致 WebDAV 备份 zip 包中的路径带有反斜杠，在 Linux/Android 上解压无法还原目录结构。
- **修复**: ZIP Entry 路径统一替换为正斜杠 `/`。

### V8 [已修·P2 中] WebDAV Basic 认证大小写敏感与 PROPFIND XML 实体转义
- **修复**: `WebdavController.rs` Basic 认证忽略大小写；PROPFIND 生成 XML 时对文件名进行 XML 实体转义（`&`, `<`, `>`, `"`, `'`）。


---

## W. 第 6 轮深度差异排查报告（ROUND 6，2026-08-17）

### W1 [已修·P0 严重] `BookHelp::save_images` 协程转录为空壳（正文图片缓存丢失）
- **Kotlin**: `BookHelp.kt:124-148` 遍历正文中的 `<img>` 标签，通过 `scope.async { saveImage(...) }` 并发下载图片并等待写入本地缓存。
- **Rust**: `stubs.rs` 中 `CoroutineScope::r#async` 丢弃闭包参数，正文图片下载任务完全不执行，离线阅读/导出 EPUB 图片附件失效。
- **修复**: 重构 `BookHelp.rs` 中的 `save_images`，提取全部图片 URL 并真实执行 `save_image` 异步下载落盘。

### W2 [已修·P0 严重] 临时文件写入重名碰撞与 Windows 原子重命名失败（静默丢数据）
- **Kotlin**: `VertExt.kt:166-175` 使用 `Files.createTempFile` 生成唯一临时文件，写完后先备份再 `ATOMIC_MOVE`。
- **Rust**: `stubs.rs:3090-3116` `create_temp_file` 采用毫秒命名（高并发下重名碰撞相互覆盖）；`move_path` 仅调用 `std::fs::rename`，在 Windows 上目标存在时报错并被静默丢弃，导致新内容丢失。
- **修复**: `create_temp_file` 引入纳秒时间戳与原子自增计数器；`move_path` 在 Windows 上若目标已存在先安全移除再 rename，若跨卷失败回退到 copy+remove，杜绝静默失败。

### W3 [已修·P1 高] `users.json` 管理端操作（增删改、重置密码）未纳入全局锁保护
- **Kotlin**: 依赖统一的锁机制或按接口事务更新。
- **Rust**: `UserController.rs` 的 `add_user`、`reset_password`、`delete_users`、`update_user` 均为无锁裸读写，若恰逢用户后台 Token 自动续期/登录（`save_user_session` 持有锁），两端并发覆写会导致新用户丢失或密码重置失效。
- **修复**: 在 `UserController.rs` 所有涉及 `users.json` 修改的入口补齐 `let _users_lock = users_json_lock_guard();`，确保全局事务互斥。


---

## X. 第 7 轮深度差异排查报告（ROUND 7，2026-08-17）

### X1 [已修·P0 严重] `HtmlFormatter.rs:74` Eager Evaluation 引发必现 Panic
- **Kotlin**: `HtmlFormatter.kt:47` `matcher.group(1)?.let { ... } ?: matcher.group(2) ?: matcher.group(3)!!`（短路求值）。
- **Rust**: `HtmlFormatter.rs:74` `...unwrap_or(matcher.group_idx(3).unwrap())`（急切求值）。当正文图片匹配第 1 分支或第 2 分支时，第 3 捕获组为 `None`，但 `unwrap_or(...)` 在传入前就执行了 `.unwrap()`，直接触发 panic 导致正文解析失败。
- **修复**: 改为 `.or_else(|| matcher.group_idx(2)).or_else(|| matcher.group_idx(3)).unwrap_or_default()`。

### X2 [已修·P0 严重] `BookContent.rs:155-156` 调试日志输出 UTF-8 字符边界切片 Panic
- **Kotlin**: `BookContent.kt:117-121` 使用 UTF-16 字符维度 `substring`。
- **Rust**: `BookContent.rs:152-160` 直接按字节切片 `content_str[..150]` 与 `content_str[len-150..]`，切在多字节汉字中间立即引发 `byte index is not a char boundary` Panic。
- **修复**: 使用字符迭代器 `content_str.chars().collect::<Vec<char>>()` 提取首尾 150 个字符。

### X3 [已修·P1 高] `BookContent.rs:111-135` 多页正文抓取 `res.body().unwrap()` Panic
- **Kotlin**: `BookContent.kt:86-107`。
- **Rust**: `res.body().unwrap()` 无判空保护，任何一页网络失败会导致整章抓取崩溃。
- **修复**: 补齐 `if let Some(body_) = res.body()` 判空保护，且防越界终止条件改为以 `redirect_url` 解析绝对路径比较。

### X4 [已修·P2 中] `BookController.rs:1714` POST SSE 默认搜索数对齐
- **Kotlin**: `BookController.kt:1134` POST 默认值为 30。
- **Rust**: `BookController.rs:1714` POST 默认值误写为 5。
- **修复**: 改为 30。

### X5 [已修·P3 低] `Utf8BomUtils.rs:36` `hasBom` 边界条件 `>= 3` 容错
- **修复**: 在 `removeUTF8BOM`、`removeUTF8BOM_bytes`、`hasBom` 中将 `len > 3` 改为 `len >= 3`。
