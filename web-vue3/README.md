# Vue 3 界面迁移工作区（预览）

本目录以同仓库 Rust `master` 的 `web-ui` 在提交 `37eff863` 的源码为
**视觉和交互参考快照**，2026-09-25 导入到 Java/Kotlin `legacy` 的独立目录。
两条主线不会共享后端：开发代理默认只指向本机 Java/Kotlin 服务
`http://127.0.0.1:8080`，可通过 `READER_BACKEND_URL` 覆盖。

```bash
cd web-vue3
npm ci
npm run build
npm test
npm run audit:api
READER_BACKEND_URL=http://127.0.0.1:8080 npm run dev
```

本地构建与单元测试使用 Node.js 24；`npm test` 会运行全部 `src/**/*.test.ts`
（Node 原生测试运行器），而不是只挑选少数测试文件。`audit:api` 是前端路由名与
Java/Kotlin 路由注册表的静态比对，不能代替浏览器旅程或响应结构验证。

子目录构建保留根路径为默认值；仅在需要把界面挂载到 `/reader/` 时设置：

```bash
READER_UI_BASE=/reader/ npm run build
READER_UI_BASE=/reader/ READER_BACKEND_URL=http://127.0.0.1:8080 \
  npm run preview -- --host 127.0.0.1 --port 4173
```

此布局要求反向代理仍在站点根路径提供 `/reader3` API 和 `/assets` 动态资源，
并把 `/reader/*` 的刷新请求回退到此构建的 `index.html`。`READER_UI_BASE`
仅调整 Vue Router、静态资源及 PWA 清单路径，不会改写后端 API 前缀。
`Vue3PreviewSubdirectoryTest` 会在隔离后端与 Chromium 上检查构建产物的
登录、刷新、资源、PWA Service Worker 作用域和根路径 API。

`npm run build` 只验证 Vue/TypeScript 编译，不验证页面与 Java/Kotlin
后端兼容。现已新增显式 Gradle 打包选项 `-PreaderWebUi=vue3`，把构建产物
放入 JAR 的 `web-vue3/` 资源目录；`reader.app.web-ui=vue3` 会选择该静态资源并
为 HTML5 history 页面回退到 `index.html`。这条 JAR 集成仍待 GitHub 托管 runner
构建及浏览器旅程验证。未完成验证前，默认入口仍为原版 Vue 2；预览构建和既有隔离
旅程只能证明已覆盖的页面，不代表其余功能或生产部署可用。

本机的 opt-in 页面测试需要一个**隔离** Reader 工作目录、Vite
预览和已安装的 Chrome。先把 Vite 的 `READER_BACKEND_URL` 指向隔离
Reader，再在仓库根目录设置 `READER_VUE3_PREVIEW_URL` 和
`READER_BROWSER_EXECUTABLE`，并确认 `READER_VUE3_ISOLATED=1`，运行：

```bash
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 ./gradlew -p browser-poc test \
  --tests com.medwarp.reader.browserpoc.Vue3PreviewLoginTest --no-daemon
```

登录测试会注册随机账户，检查登录 token 和刷新后的空书架 API。
阅读测试还需要在 loopback 启动 `scripts/mock-book-source.py`，设置
`READER_BOOK_FIXTURE_URL`，然后把上述 `--tests` 改为
`com.medwarp.reader.browserpoc.Vue3PreviewReadingTest`；它会检查书籍详情、
目录、首章正文、下一章导航，以及从详情页分别下载 TXT 和 EPUB（检查后端
`isEpub` 参数和 EPUB ZIP 文件头）。导出 UI 仅提供当前 Java/Kotlin 后端支持的
TXT/EPUB，不显示未实现的 HTML 或 TXT 编码选项。`Vue3PreviewSearchTest` 还覆盖指定书源、
多源 SSE、批量降级及未入架直读。`Vue3PreviewSourceTest` 检查远程预览
不写库、确认后导入以及批量删除。`Vue3PreviewFileTest` 检查文件重命名、单文件／目录导入，
以及服务端移动目录与二进制文件。`Vue3PreviewSecureFileTest` 使用另一隔离工作目录、
启用 `reader.app.secureKey` 的 Reader 和独立 Vite 端口，检查错误密码后的重试、
正确密码下根目录新建与列表刷新；需额外设置 `READER_VUE3_SECURE_URL` 和合成的
`READER_TEST_MANAGER_KEY`。所有测试都禁止非 loopback 预览地址，且必须显式设置
`READER_VUE3_ISOLATED=1`。不要将 Vite 代理指向生产数据或公网服务。

GitHub 托管 runner 的 `vue3-preview.yml` 会自行构建 Reader、启动隔离服务，
执行登录、阅读、搜索、书源、分组、文件和备份七项浏览器测试、单独的安全文件测试以及子目录构建测试；
无需在用户或宿主 Docker 上预装 Chrome。

备份旅程遵循 legacy `/reader3/backupToWebdav` 的空字符串响应及用户 home 下
`webdav/legado/backupYYYY-MM-DD.zip` 路径；导出会经文件 API 下载此 ZIP。还原只接受
文件管理器中当前 home 已存在的 ZIP，并调用 `/reader3/file/restore`。原版恢复逻辑会替换
归档所包含的用户数据，因此界面明确警告；不提供后端并不存在的“覆盖/合并”开关，也不把任意
本地 ZIP 上传伪装成受支持的恢复 API。

后续按 `../docs/VUE3-UI-MIGRATION.md` 核查接口、登录、书源、阅读、
SSE/下载和 `/reader/` 子目录路径。Rust 前端中的“后端待实现”降级
与缺失路由必须逐项处理；不能靠静态假数据或复制 Rust 后端契约放行。
