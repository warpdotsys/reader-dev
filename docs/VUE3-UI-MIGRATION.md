# Vue 3 UI 迁移核查

更新日期：2026-09-25。本文记录源码核查结果与接入边界，不代表新界面已经发布。

## 目标与来源

- 视觉与交互参考：同一工作区 `work/reader-dev` 的 Rust `master/web-ui`，使用 Vue 3、Element Plus、Pinia 和 Vite。其 `src/styles/main.css` 提供浅/深色变量、紧凑卡片和阅读页主题；页面组件包括登录、书架、搜索、阅读、书源管理等。
- 功能与数据基线：本工程 Java/Kotlin 服务、原始 `reader-pro-3.2.14.jar` 以及受控差分报告。Rust 的 API 实现和前端降级逻辑不作为兼容性证明。
- 当前对外前端仍是 Vue 2。已从 Rust `37eff863` 导入独立的 `web-vue3/` 预览工作区；它不覆盖 `web/`、JAR 原版静态资源或生产入口。快照来源和构建方法见 `web-vue3/README.md`。

## 已从当前源码核查

Rust 前端 `web-ui/src/api/request.ts` 将请求发往 `/reader3`，以 query 参数附带 `accessToken`，并按 `ReturnData.isSuccess/errorMsg/data` 处理结果；`web-ui/src/router/index.ts` 使用 HTML5 history 路由。Java/Kotlin 的 `YueduApi.kt` 显式注册了 `/reader3/login`、`getBookshelf`、`getBookInfo`、`searchBook`、`getBookContent`、`getBookGroups` 和部分 `/reader3/file/*` 路由。这只能证明路径名初步重合，不能证明参数、默认值、响应体或鉴权语义一致。

可重复的静态扫描命令为 `cd web-vue3 && npm run audit:api`。2026-09-25 最新快照中，API 文件含 **110 个静态路由引用**，其中 **67 个**与当前 `YueduApi.kt` 注册路径同名，**43 个**不同名或未注册，另有 **7 处动态调用**尚未纳入。这只统计路径名，不校验 HTTP 方法、参数、响应或页面调用条件；部分不同名路由可能有替代接口。核心链路中的 `getBookToc` 已适配当前 `getChapterList`，正文按目录 `index` 取值并经隔离页面测试；其他同名路由仍须逐项核对语义。`/file/rename` 已补为沿用文件 home 与权限边界的真实文件／目录重命名；新增 `/file/move` 在同一 home 内移动目录与二进制文件，不经前端读写；`scanLocalBookDir` 已接到本地文件导入书架。`getServerStats`、`getReadingStats`、`getBookCacheChapters`、`cacheBookRangeOnServer` 等仍无同名路由。不能因为前端存在或能构建就标为可用，也不能在没有核对语义时机械改名。

Vue 3 默认仍构建为根路径。通过 `READER_UI_BASE=/reader/` 可生成子目录构建，Vue Router、HTML 入口、字体、Logo、PWA 清单及 Service Worker 作用域随之指向 `/reader/`；两种挂载位置的 Service Worker 缓存分离，激活时只清理本作用域旧缓存。`/reader3` API 和 `/assets` 后端资源仍须由反向代理在站点根路径提供。本机与 [GitHub 托管 runner](https://github.com/warpdotsys/reader-dev/actions/runs/36110736651) 已用生产构建的 Vite preview、隔离 Reader 和 Chromium 验证 `/reader/login` 直达、登录、跳转、搜索页刷新、静态资源、Service Worker 与 API 代理。此验证不等于已在真实反向代理上部署。

## 实施顺序与验收

1. 在独立 Vue 3 工程中提取视觉变量、导航和布局；保留 Vue 2 的现有生产入口。先做可预览页面，不宣称业务已打通。
2. 建立逐接口契约表，至少覆盖登录和 token、用户命名空间、书架、搜索与书源、目录/正文、文件、SSE 和下载；对缺失接口明确是补后端、适配前端还是暂时隐藏入口。不能用静态假数据伪装成功。
3. 按登录 → 书架 → 搜索/书源 → 阅读 → 文件/设置的顺序接入，每页同时验证空态、错误态、权限、刷新和浏览器返回。对非 JSON 响应单独测试。
4. 在站点根路径及 `/reader/` 子目录分别做浏览器端到端测试；记录截图、网络请求和实际响应。移动端、深色模式、键盘操作与字体加载也要验收。
5. 仅在主要链路通过、已知缺陷写入发布说明且有一键回退到 Vue 2 的办法后，考虑切换默认入口。

## 当前结论

**已从源码验证**：Rust 前端的设计系统和请求约定存在；Java/Kotlin 注册了部分同名路由。`getBookInfo/getChapterList/getBookContent` 查询需用 `bookSourceUrl` 查书源；正文还要求 `index` 且不能同时给非空 `chapterUrl`。批量搜索返回 `{lastIndex,list}`，不是数组或页码；旧后端的精确搜索使用关键词 `=` 前缀。预览界面已按这些契约适配；“路由同名”仍不等于可用。原 `saveFromRemoteSource` 是写入接口，不能当只读预览使用；新增的 `previewRemoteBookSources` 在用户确认前不写书源。**已成功重建**：独立 `web-vue3/` 的 Vue/TypeScript 生产构建、隔离 Reader 服务及 Chromium 页面测试均在本机和 GitHub 托管 runner 通过（[Vue 3 runner 记录](https://github.com/warpdotsys/reader-dev/actions/runs/36122941645)）。测试覆盖注册、令牌、刷新后空书架，确定性书源的搜索、多源 SSE、指定单源、SSE 失败后批量降级、未入架详情/目录/正文、入架书正文和下一章导航，远程预览只读、确认导入和批量删除，以及文件重命名和单文件／递归目录入架。本次托管运行还通过书架分组创建、重命名、内置筛选只读、多组持久化、刷新后掩码解码和删除清理，以及受保护文件写入与 `/reader/` 构建冒烟。**尚未验证**：线上真实书源、大量书源下的游标续搜、完整页面/接口兼容、缺失接口的替代方案、真实反向代理部署和生产可用性。**尚未实施**：完整逐页功能迁移和默认入口切换。

预览回滚：当前生产入口仍在 `web/`，Vue 3 只在独立 `web-vue3/` 和 CI 中运行。停止 Vite 预览即可回到现有 Vue 2 界面，不需要改动生产数据；将来若切换正式入口，必须先准备独立的静态资源版本和明确的入口回切步骤，再按第 5 条验收。

文件页增量验证：`Vue3PreviewFileTest` 已在本机和 [GitHub 托管 runner](https://github.com/warpdotsys/reader-dev/actions/runs/36113258471) 中验证重命名、正文不变、拒绝 `../` 路径和同名覆盖，以及单文件和递归目录导入；同一托管流程还验证了目录与二进制文件移动后的内容、拒绝移动到自身子目录／越界路径／同名目标。`Vue3PreviewSecureFileTest` 也在该 [托管流程](https://github.com/warpdotsys/reader-dev/actions/runs/36113258471) 中通过：独立工作目录和非生产管理密码验证错误密码提示、正确密码重试、根目录新建、列表刷新，以及请求 URL 不含密钥；测试另行确认旧版 `secureKey` query 仍可用。`checkManagerAuth` 现优先读取 `X-Reader-Secure-Key` 请求头，同时保留 `secureKey` query 作为 Vue 2 和脚本的兼容入口；Vue 3 只发请求头。旧客户端若继续用 query，生产仍须使用 HTTPS 并避免记录完整 URL。`/file/move` 是恢复工程的新增接口，不是原 JAR 接口，尚无 JAR 黑盒对照。文件页可导入格式提示已按当前 Java/Kotlin 实际支持的 txt、epub、umd、cbz、pdf 收紧，未宣称支持其他格式。文件页其余入口未全部验收，不能把整个文件页视为完成。

导出兼容增量：Vue3 的详情页和书架弹窗现只显示 Java/Kotlin 后端可实现的 TXT、EPUB；统一将格式映射到 `GET /reader3/exportBook` 的 `isEpub=0|1`，不再展示后端会忽略的 HTML、GBK 编码选项或尚未实现的 warning。隔离 Reader + Chromium 本机测试已覆盖两种 UI 下载路径，并断言 EPUB 响应为 ZIP；当前工作流提交后的托管 runner 验证尚待完成。该测试使用合成书源，不代表线上真实书源或所有书籍导出均已验证。

书架分组兼容增量（从当前 Java/Kotlin 路由与控制器源码验证；不等于原始 JAR 黑盒验证）：Vue 3 预览原先直接把 `id/name/orderNum` 发给 legacy `BookGroup(groupId/groupName/order)`，并调用不存在的 `/addBookGroup`、`/removeBookGroup`、`/setBookGroups`。现由适配层将读取/保存/删除/排序映射到 `getBookGroups`、`saveBookGroup`、`deleteBookGroup`、`saveBookGroupOrder` 的真实字段；书籍多分组映射为 `Book.group` 的位掩码，并使用 `addBookGroupMulti/removeBookGroupMulti` 的 `{ groupId, bookList: [{ bookUrl }] }` 结构。由于旧版移组接口以 XOR 清位，调用方先读当前书架，只把确实具有该位的书传给移除接口；清空分组按现有位逐项移除。旧版删除分组只删分组记录、不清理书籍掩码，因此 Vue 3 先移除相关书籍位，再删记录。内置负数分组保持只读，并按旧 Vue 2 的源码语义过滤全部、本地、音频、未分组和更新错误书籍。纯映射/位掩码 Node 测试 **5 项通过**，Vue 3 `vue-tsc` + 生产 Vite 构建本机通过；新增隔离 Reader + Chromium E2E 已在 [GitHub 托管 runner](https://github.com/warpdotsys/reader-dev/actions/runs/36122941645) 验证创建、重命名、多组持久化、刷新解码和删除清理。测试期间还发现并修复成功保存后受 `menuBusy` 保护而无法关闭分组弹窗的问题。尚未由该测试证明排序的浏览器拖放、多用户并发写入或原 JAR 行为。
