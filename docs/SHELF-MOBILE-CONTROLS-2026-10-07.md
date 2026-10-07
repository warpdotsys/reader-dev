# 手机书架分组控件重叠：实际复现和源码修复

## 已验证的故障

核对 `de7c0c7e` 的托管截图时发现疑似导航遮挡。新增页首观测要求 `scrollY=0`，先实际排除了初始导航遮挡：此前悬浮截图会自动滚动，标题经过半透明 sticky 顶栏，不足以判定初始布局错乱。

但旧生成 JAR 的 360px 页首截图确实存在另一问题：分组标签画到右侧管理按钮下；“我的书架”和管理文字也被挤成多行。只检查 document 横向溢出的旧测试不会发现它。新增矩形相交及 `elementFromPoint` 命中断言，旧 JAR `1986ce33...` 实际 1 项失败／0 跳过，19.618 秒：`width=360 density=0 group controls overlap filters`。第 13 个观测记录重叠 true、标签可见命中 false；失败 XML、几何、截图和生成 JAR 均在独立 `build/shelf-page-top-20261007-a/` 保留，没有覆盖原始 3.2.14 JAR。

## 源码修复

- 分组标签在自己的容器内横向滚动，不能越过 flex 区域盖住操作按钮。
- 480px 以下，标题独占网格第一行；统计、管理、导入和刷新完整保留在下一行。分组筛选与管理操作分行。
- 不隐藏功能或改变业务接口／存储／账号。测试扩到 6 宽度 × 3 密度，保存页首截图和有限几何 JSON；保留原悬浮预览检查。360px 实际点击此前被盖住的最后一项筛选，再点击“全部”恢复生成书架。
- GitHub hosted Vue 3 工作流保存新截图与几何，不把 CSS 文本断言冒充渲染结果。

## 已成功重建与本机实际验收

241 项前端测试无失败／跳过，类型检查和生产构建通过；发布结构／原生身份 55 项本轮回归通过。离线 `bootJar` 实际 58 秒，Java/Kotlin 编译 up-to-date、资源和 JAR 重建，不冒充全后端又重编译／测试。新 JAR SHA-256 `eaa34cc99308aed6bfad159f2d3e1e19c08a55a629ad77cbffd4224327228f32`，书架 CSS 与本次 dist 字节一致，原界面保留，Camoufox worker 未改。

现有 Chrome 对新 JAR／隔离数据真实执行 16.076 秒，1 项、0 失败／错误／跳过。1135／1024／768／720／360／320px × 三密度共 18 组几何全部通过：页首无导航遮挡，标题／导入可命中，分组标签不与管理按钮相交且能命中。原横向溢出、搜索宽度、全部导航和悬浮简介检查仍通过。两张新页首截图已目视，桌面布局保留、手机标题和管理控件正常。

只用随机账号、15 本生成元数据和回环夹具，不读真实正文或复用网站会话。Reader JVM 限 512 MiB 堆及 2 个可用处理器，Gradle 堆 256 MiB；这不是 Linux 全镜像累计资源预算。两个自有进程句柄均停止，18946／18947 监听独立复查消失，用户 18931 未操作。[失败和通过的散列、指标与范围](evidence/shelf-mobile-controls-2026-10-07.json)。

可重复构建：在 JDK 11 和锁定 Node 依赖的干净检出上，先 `npm ci --prefix web-vue3`、`npm run build --prefix web-vue3`，再 `./gradlew -PreaderWebUi=vue3 bootJar --max-workers=2 --no-daemon`；Windows 用 `gradlew.bat`。浏览器回归用全新回环 Reader／夹具和显式隔离变量，执行 `./gradlew -p browser-poc test --tests com.medwarp.reader.browserpoc.Vue3PreviewShelfLayoutTest --no-daemon --max-workers=2`；必须检查 XML 无跳过及 18 组几何，不能指向生产或 18931。

## 尚未验证、已知问题与回退

此修复自己的托管构建／镜像待验收；`de7c0c7e` 的默认入口证据不验收本次 CSS。865px 高度和鼠标点击不是实体手机、触摸／软键盘或全部短视口验收；长自定义分组实际滚动未专项测试。真实起点解析／认证三方、原 JAR 新样本差分及生产发布未由本次修复完成。

无数据迁移、无字节码补丁。需要回退时，在干净维护检出撤回这组响应式 CSS 并重建，保留失败回归和证据；不要在用户脏工作区 reset／checkout。保留 Vue 2 的候选镜像可显式选择 `READER_APP_WEBUI=vue2`；本轮不修改生产环境，不把回退写成已执行生产切换。
