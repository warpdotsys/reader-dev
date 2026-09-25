# Vue 3 UI 迁移核查

更新日期：2026-09-25。本文记录源码核查结果与接入边界，不代表新界面已经发布。

## 目标与来源

- 视觉与交互参考：同一工作区 `work/reader-dev` 的 Rust `master/web-ui`，使用 Vue 3、Element Plus、Pinia 和 Vite。其 `src/styles/main.css` 提供浅/深色变量、紧凑卡片和阅读页主题；页面组件包括登录、书架、搜索、阅读、书源管理等。
- 功能与数据基线：本工程 Java/Kotlin 服务、原始 `reader-pro-3.2.14.jar` 以及受控差分报告。Rust 的 API 实现和前端降级逻辑不作为兼容性证明。
- 当前对外前端仍是 Vue 2。已从 Rust `37eff863` 导入独立的 `web-vue3/` 预览工作区；它不覆盖 `web/`、JAR 原版静态资源或生产入口。快照来源和构建方法见 `web-vue3/README.md`。

## 已从当前源码核查

Rust 前端 `web-ui/src/api/request.ts` 将请求发往 `/reader3`，以 query 参数附带 `accessToken`，并按 `ReturnData.isSuccess/errorMsg/data` 处理结果；`web-ui/src/router/index.ts` 使用 HTML5 history 路由。Java/Kotlin 的 `YueduApi.kt` 显式注册了 `/reader3/login`、`getBookshelf`、`getBookInfo`、`searchBook`、`getBookContent`、`getBookGroups` 和部分 `/reader3/file/*` 路由。这只能证明路径名初步重合，不能证明参数、默认值、响应体或鉴权语义一致。

可重复的静态扫描命令为 `cd web-vue3 && npm run audit:api`。2026-09-25 快照中，API 文件含 **108 个静态路由引用**，其中 **63 个**与当前 `YueduApi.kt` 注册路径同名，**45 个**不同名或未注册，另有 **7 处动态调用**尚未纳入。这只统计路径名，不校验 HTTP 方法、参数、响应或页面调用条件；部分不同名路由可能有替代接口。核心链路中的 `getBookToc` 已适配当前 `getChapterList`，正文按目录 `index` 取值并经隔离页面测试；其他同名路由仍须逐项核对语义。`getServerStats`、`getReadingStats`、`getBookCacheChapters`、`cacheBookRangeOnServer`、`scanLocalBookDir` 与 `/file/rename` 尚未找到同名路由。不能因为前端存在或能构建就标为可用，也不能在没有核对语义时机械改名。

Rust UI 的字体资源和 Vite 开发代理使用根路径（例如 `/fonts`、`/reader3`）；history 路由及资源根路径还需覆盖 `/reader/` 子目录部署，不能只在站点根目录测试。

## 实施顺序与验收

1. 在独立 Vue 3 工程中提取视觉变量、导航和布局；保留 Vue 2 的现有生产入口。先做可预览页面，不宣称业务已打通。
2. 建立逐接口契约表，至少覆盖登录和 token、用户命名空间、书架、搜索与书源、目录/正文、文件、SSE 和下载；对缺失接口明确是补后端、适配前端还是暂时隐藏入口。不能用静态假数据伪装成功。
3. 按登录 → 书架 → 搜索/书源 → 阅读 → 文件/设置的顺序接入，每页同时验证空态、错误态、权限、刷新和浏览器返回。对非 JSON 响应单独测试。
4. 在站点根路径及 `/reader/` 子目录分别做浏览器端到端测试；记录截图、网络请求和实际响应。移动端、深色模式、键盘操作与字体加载也要验收。
5. 仅在主要链路通过、已知缺陷写入发布说明且有一键回退到 Vue 2 的办法后，考虑切换默认入口。

## 当前结论

**已从源码验证**：Rust 前端的设计系统和请求约定存在；Java/Kotlin 注册了部分同名路由。`getBookContent` 旧后端要求 `index` 且不能同时给非空 `chapterUrl`，预览界面已按此适配；“路由同名”不等于可用。**已成功重建**：独立 `web-vue3/` 的 Vue/TypeScript 生产构建在本机及 GitHub 托管 runner 通过。隔离数据目录下的浏览器测试完成注册、令牌写入、刷新后空书架；另用本机确定性书源走通书籍详情、两章目录、首章中文正文及点击下一章后第二章正文渲染（两项测试均 0 跳过、0 失败）。**尚未验证**：线上真实书源、完整页面/接口兼容、缺失接口的替代方案、子目录部署和生产可用性。**尚未实施**：完整逐页功能迁移和默认入口切换。
