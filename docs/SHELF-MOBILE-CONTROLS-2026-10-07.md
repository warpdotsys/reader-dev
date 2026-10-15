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

## 本修改自己的托管验收与长分组增量

源码 `e5cc2ba8`、PR 受测合并快照 `ea3d709f...` 的 [Java/Kotlin](https://github.com/warpdotsys/reader-dev/actions/runs/37574666793)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37574666764)、[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37574666765)、[双架构原生构建／重载／发布导入](https://github.com/warpdotsys/reader-dev/actions/runs/37574666745)已全部完成成功，原生六作业均实际执行。18 组页首几何逐项复查通过，360px 托管截图已目视。19 份 Vue 核心／子路径 XML 无失败／错误／跳过；两份默认 Camoufox 各 18 项、文档 helper 各 3 项及 Chromium 23 项通过原守卫。Java 161 项中 132 实际通过、29 门控跳过，不算全部引擎实际执行。

24 份 publisher JSON 精确文件集合、严格 JSON、四组身份、两架构共享 JAR、5 份默认 UI／异步 API／资源报告已独立验证。最高短时峰值 847,716,352 B、PID 205，触限／OOM／实际 swap 均为 0，但配置允许 1 GiB swap。原生共享 JAR 为 `bbd52660...`，普通完整镜像独立构建 JAR 为 `9752bdb2...`，不宣称跨 workflow 字节一致。没有下载本机多 GiB 镜像，归档散列由 CI 记录。

随后补上此前缺失的长自定义分组回归，只改测试和制品保存，不改已验收 CSS。新随机账号通过界面创建四个 20 字分组，重载后共 9 个标签；1135／360／320px 实际滚动并点击最后一项、打开管理、再切回全部恢复 15 本生成书籍，三组无横向文档溢出／操作遮挡。页首 `scrollY=0` 的最终回归 24.084 秒、1 项、0 失败／错误／跳过，320px 图已目视；原 18 组及悬浮检查仍通过。Java／夹具自有句柄停止，测试端口消失。新长分组断言自身的 hosted 执行仍待后续提交验收，不倒填为 `e5cc2ba8` 已执行此新增断言。[准确制品、守卫、散列与后续本机范围](evidence/shelf-mobile-hosted-e5cc2ba8-2026-10-07.json)。历史失败／通过报告原样保留。

## 尚未验证、已知问题与回退

上节待验的长分组增量现已由自己的源码 `c3307ded`／受测合并快照 `69a222de...` 完成：[Java](https://github.com/warpdotsys/reader-dev/actions/runs/37576894576)、[Vue 3](https://github.com/warpdotsys/reader-dev/actions/runs/37576894560)、[完整镜像](https://github.com/warpdotsys/reader-dev/actions/runs/37576894595)、[原生六作业](https://github.com/warpdotsys/reader-dev/actions/runs/37576894578)均成功；五个互斥专项跳过不算通过。下载的核心 18 份 XML 全部实际执行，三组长分组 JSON 的滚动／遮挡／页首及 9 标签逐项核对通过，320px 生成截图已目视。该 UI 审计没有独立消费 c330 的全镜像／原生小身份和预算报告，不宣称所有制品已独立验收，也不能验收随后新增的第 19 个数值线程契约。[本增量自己的散列、指标及范围](evidence/shelf-custom-groups-hosted-c3307ded-2026-10-07.json)。

此修复自己的托管构建／镜像已按上节验收，不借用 `de7c0c7e` 的绿灯。865px 高度和鼠标点击不是实体手机、触摸／软键盘或全部短视口验收；长分组只证明四个生成的 20 字分组／三个视口，不证明所有组数和所有名称。真实起点解析仍失败，认证三方、原 JAR 新样本差分及生产发布未由本次修复完成。

无数据迁移、无字节码补丁。需要回退时，在干净维护检出撤回这组响应式 CSS 并重建，保留失败回归和证据；不要在用户脏工作区 reset／checkout。保留 Vue 2 的候选镜像可显式选择 `READER_APP_WEBUI=vue2`；本轮不修改生产环境，不把回退写成已执行生产切换。
