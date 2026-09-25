# Vue 3 界面迁移工作区（预览）

本目录以同仓库 Rust `master` 的 `web-ui` 在提交 `37eff863` 的源码为
**视觉和交互参考快照**，2026-09-25 导入到 Java/Kotlin `legacy` 的独立目录。
两条主线不会共享后端：开发代理默认只指向本机 Java/Kotlin 服务
`http://127.0.0.1:8080`，可通过 `READER_BACKEND_URL` 覆盖。

```bash
cd web-vue3
npm ci
npm run build
READER_BACKEND_URL=http://127.0.0.1:8080 npm run dev
```

`npm run build` 只验证 Vue/TypeScript 编译，不验证页面与 Java/Kotlin
后端兼容。本目录尚未打入 Reader JAR、镜像或生产入口，原版 Vue 2
仍是正式界面。预览构建只证明可编译；隔离浏览器测试只证明其覆盖的页面旅程，
不能推断其余功能可用。

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
目录、首章正文和下一章导航。`Vue3PreviewSearchTest` 还覆盖指定书源、
多源 SSE、批量降级及未入架直读。`Vue3PreviewSourceTest` 检查远程预览
不写库、确认后导入以及批量删除。四项测试都禁止非 loopback 预览地址，且必须显式设置
`READER_VUE3_ISOLATED=1`。不要将 Vite 代理指向生产数据或公网服务。

GitHub 托管 runner 的 `vue3-preview.yml` 会自行构建 Reader、启动隔离服务，
并执行上述三项浏览器测试；无需在用户或宿主 Docker 上预装 Chrome。

后续按 `../docs/VUE3-UI-MIGRATION.md` 核查接口、登录、书源、阅读、
SSE/下载和 `/reader/` 子目录路径。Rust 前端中的“后端待实现”降级
与缺失路由必须逐项处理；不能靠静态假数据或复制 Rust 后端契约放行。
