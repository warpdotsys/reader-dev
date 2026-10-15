# Reader 同record切书与进度归属

更新日期：2026-10-11。维护基线35587，本轮产品源码已提交并推送为3e225；验收绑定自己的测试合并快照f180，不能借基线自己的四条托管绿灯接受新产品。工作区原有58份用户报告、原件与只读备份保留。

## 已从维护源码确认的缺陷

`/reader/:bookUrl` 是同一个Vue Router record。原App的RouterView没有书籍key，Reader只在mounted初始化且已有EpubDoc直接复用；A→B参数导航可复用A的正文/目录/EPUB。原bookUrl又直接读取全局route，A卸载时route可能已是B，进度保存有归属错误风险。相似分支名、旧生成UI成功或重新加载整页都不能排除这条路径。

原章节处理只将chapterIndex写入URL，没有query→chapter反向处理；同书历史后退/前进可改URL却仍显示旧章。临时书原onBeforeRouteLeave也不覆盖同record参数更新，直接切另一书可绕过入架提醒。这些是维护源码确认，不是重新从原3.2.14界面验证相同缺陷。

## 候选实现

- App仅按Reader的bookUrl作key，同书章节/query不重建，不按fullPath重建。
- Reader冻结创建时bookUrl及直读配置query；A卸载的本地/服务端进度仍归A。保留原AbortController、EPUB销毁和正文异步代次保护。
- 旧实例只能同步属于自己的路由；新增有效chapter query→现有goToChapter，忽略非整数、越界与卷条目，己方同步不会重复加载。
- leave/update共用临时书提醒，只在换书/离开触发，单书翻章不弹提醒。
- init、非文本加载及HLS动态加载在await之后检查实例/身份/会话/代次，避免卸载后继续发下一请求或创建迟到HLS实例。不更改正文安全、存储格式、HTTP接口、Worker15秒或下载60秒上限。

此修复与[准确f5/授权单书Worker压力诊断](EPUB-WORKER-DIAGNOSTIC-2026-10-11.md)分开；没有证明它是H/I/K超时或PID触限的根因。

## 本机已执行与实际 Chrome 验收

真实Vue createMemoryHistory/RouterView/renderer生成测试2项通过：A chapter0→chapter1→back→forward仍同实例，A→B卸载A/挂载B；另核对实际App/Reader源契约。它使用最小生成Reader，不能称实际6000行Reader、媒体、网络或EPUB端到端通过。

完整前端304项/零跳过、vue-tsc和Vite打包通过；本机首轮准备命令没找到npm，未计执行成功，后使用已提供Node直接调用现有本地测试/编译入口，未执行npm ci。本机现有plugin-vue6.0.9/vue3.5.43与锁文件6.0.8/3.5.40不一致，因此这部分仅为补充验证，不是锁定依赖构建证明；托管npm ci结果另验。完整Python717项/30.945秒/1项既有Windows-POSIX跳过，发布/截图等生成守卫139项全通过。

本机JDK11/Gradle6.1.1增量 `test bootJar` 成功，实际XML合计205项、失败/错误0、35项既有环境跳过。没有在本机clean整个build以避免删除授权输入、历史失败及用户测试数据；精确双clean可重复构建仍须本提交自己的GitHub托管CI。

首轮JAR 285,677,568B，SHA `4bf6e02a6b87ed8fbec1d8e72cac1d3543084c1bc3a7161d5983cc137a9cc0ad`。实际生成Chrome单case40.001秒、失败/错误/跳过0，但严格资源身份门禁拒绝整轮；不能只凭页面回归成功发版。[首轮拒绝回执](evidence/reader-route-first-ui-pass-resource-identity-rejected-2026-10-11.json)，SHA `ec0dc4398aea4de72c63dfaf5064934f5e122aa722a867bb5c1a89f1e97fe672`。

实际生成浏览器回归扩充原 `Vue3PreviewLocalReadingTest.generatedTxtAndEpubImportReadNavigateAndRefresh` 单一case，保持既有25旅程、截图与全部原EPUB安全/取消/超时/重试断言，不增加跳过。用真实生产Router直接A生成TXT→B生成EPUB，严格核对A卸载保存URL/index、B正文/目录、本地进度、B chapter query/back/forward和最终服务端进度，不能只看HTTP200。

编码修正后的第二轮，准确新JAR SHA `38522f23817738ea33c75a43df1aec91bfef06f759123d5cb0283e009badfca7`、285,677,426B，以只读副本启动独立生成目录并执行同一真实Chrome旅程，37.839秒、失败/错误/跳过0。同文档生产Router直接切A→B、B同实例query/back/forward、A卸载进度和A/B本地及服务端index1全部通过。两份生成PNG主代理实际查看：纯文本与原版排版中文可读，无空白；不含真实书籍正文。

第二轮重新逐项核验与准确旧托管f5：1570条目中仅111个Vue资源内容变化，全部1122个class（含1072个业务class及50个Boot启动class）、90依赖、worker及非Vue资源相同。worker SHA仍为 `d8b2674c5f28cf31d9003479c7e5035c9d4ebefc0286c0d8a61c4d9672a565f5`。不是相似名称判断版本一致。[本轮独立回执](evidence/reader-route-generated-chrome-38522-independent-2026-10-11.json)，SHA `a67da11c4b227d983102722352093feb52f0d11f8ff73976920440fed4df9d09`。

两轮均是本机JAR与项目Playwright驱动的headless Chrome，不是可见桌面Chrome、后端Camoufox或最新完整Linux镜像。各自服务端Windows Job为2CPU亲和性/2GiB committed/256**进程**；第二轮峰455,233,536B，Chrome在该预算外。不能把它扩称合计2GiB/256线程或用它抵消H/I/K压力失败。自有两轮服务均停止、18965关闭；没有导入真实凭据、读取其他私人书籍或修改生产。

## Windows 生成资源编码：确定差异与修正

首轮有九个非Vue资源字节差异，八个经CRLF→LF即等于旧托管资源。但 `web/js/reader.6e871769.js` 除行尾外，还把源中UTF-8 `f0ab8aae`（U+2B2AE）变成有效ASCII `3f3f`（两个问号）。严格UTF-8解码本身不能检测这种损失。源与Git blob/旧托管reader字节一致；损坏出现在`processResources`输出，再进入BootJar。内部具体编码器根因未证明，不推断为CPU性能或已修复线上全部乱码。

取消旧逐行filter，改为对明确白名单的生成JS/precache及两份YAML做≤8MiB完整严格UTF-8转换：非法字节报错、统一LF、保留补充字符、仅保留既定许可证域名字面替换与precache前缀。source、原件、二进制均不写。Windows的prod YAML源1043B含45个CRLF；生成998B与旧Linux托管完全相同。规范LF是跨系统构建选择，不改变YAML值或宣称原JAR全部配置相同。

Gradle6.1.1实际执行20条纯生成护栏：五种补充字符边界×三种行尾、precache、YAML、非法UTF-8、已有前缀与再次转换幂等。初次构建脚本编译曾因DSL遮蔽`java.nio`失败，显式import修正后通过；原失败不改写。这些生成护栏不能代替上述真实JAR逐entry核验或自己的托管双clean。

完整Python修正后再次717项/45.228秒、最终再跑44.662秒，均有1项既有平台跳过。最终明确Vue3开关的增量 `test bootJar` 1分36秒成功，实际JVM44 suites/207项、失败/错误0、35项原环境跳过；其中4项真实资源编码回归全部执行且零跳过。20条Gradle生成编码护栏在实际日志执行，前端304、发布/截图147项生成守卫另通过。它们不借35587绿灯、也不代替自己的托管双clean。

最终有限核验工具首轮误将全部1122 class当成业务class而断言失败，保留失败；实际结构是1072业务＋50 Boot类，后按真实分类核对，未改产品或绕过字节门禁。

[最终本机有限核验](evidence/reader-route-38522-final-local-checks-2026-10-11.json)，SHA `b7359b1fcd1fa6f6c4973d5084292fd7b0db5329eea423decc829362b8bfabe3`。明确Vue3开关及20条护栏后的最终JAR与第二轮实际浏览器输入38522完整SHA相同；没有用不含Vue3的准备产物或旧XML替代。构建日志确认20条执行；该JSON单独不证明日志，字段如实区分。

本机两份自有只读JAR测试副本共571,354,994B仍保留于ignored build子目录：精确清理命令被执行策略拒绝，未删除也不换途径绕过。自有服务停止、生成回执/XML/图片保留；原件/原备份/授权输入及58用户报告未改。不要把服务已停误写成这两份输入已清。

Native工作流原PR路径没有覆盖本轮UI/构建文件，会遗漏第四项验收。已将JAR、UI、浏览器旅程和Java源码/测试/资源路径加入既有GitHub托管Native排练；不改runner、权限、安全门禁或发布动作。新结果必须绑定本轮PR source与tested merge，不改用父提交绿灯或把dispatch的source-only冒充merge结果。

本机一次准备构建未带 `-PreaderWebUi=vue3`，按既定配置生成了不含Vue3的经典入口JAR；该产物不当作本轮UI验收输入，第二轮只读38522副本与原记录保留。随后用明确Vue3开关重建并另核验准确字节。干净重复构建命令（仅用于新的CI checkout，**不能在这个含授权输入的本机build目录clean**）：

```sh
npm ci --prefix web-vue3 && npm run build --prefix web-vue3
./gradlew -PreaderWebUi=vue3 clean test bootJar --max-workers=2 --no-build-cache --no-daemon
```

再以相同开关执行一次 `clean bootJar`，用既有 `scripts/verify-reader-jar-reproducibility.py` 比較两份完整JAR；命令或构建选项本身不证明可重复，须实际零差异回执。

## 尚未验证与回退

临时书提醒的实际页面、迟到媒体/HLS响应、同章不同epubAnchor的历史重定位仍须逐项补证；Linux完整镜像与自己的托管验收状态见后面的逐项记录，不用本机补充测试代替。chapter反向watch不等于所有query均已支持；同章anchor目前不在本轮成功承诺内。

不合并、发版或部署。回退定向撤回本次App/Reader源码、构建转换及相应新增回归，不改用户数据、原件或58脏报告；旧Vue2入口与原始JAR仍保留为对照。旧Windows filter已观察字节损失，回退也须重新过编码门禁，不能将该坏产物发版。不要reset/checkout覆盖整工作区。

## 3e225 自己的四项托管结果已独立接受

产品维护提交 `3e225f97efe1036e405490e0b4b253f1285a7ce4`、测试合并快照 `f18023ce9ab1b8a5f2d8b5997bae0fe602f9e9b7`；GitHub commit API已确认第二parent为3e225，SOURCE→MERGE ahead1/behind0/文件差异0。原Native漏触发已由本次真实PR自动运行证明，不以dispatch或父提交代替。

- [Java 38127563432](https://github.com/warpdotsys/reader-dev/actions/runs/38127563432)：Linux Python717/零跳过、JVM207/35原环境跳过；4项精确资源字节XML方法均实际执行、零跳过，20条Gradle护栏在两次clean日志各执行一次。两次无task-output-cache的完整JAR字节/条目/ZIP元数据/顺序均一致。[独立Java回执](evidence/reader-route-hosted-3e225f97-independent-java-acceptance-2026-10-11.json)，SHA `1f07316eb6c85ec73654a4be2f8e3b1b599c49b051b29c13996c1563834dd35c`。
- [Vue 38127563420](https://github.com/warpdotsys/reader-dev/actions/runs/38127563420)：前端304、实际生成UI25/零跳过和11份生成PNG结构守卫；扩大同record切书的单case实际执行，没有只看静态源码。[独立UI＋Java回执](evidence/reader-route-hosted-3e225f97-independent-ui-java-acceptance-v2-2026-10-11.json)，SHA `918180c3c01aac48db85cae372de7bb1981eda37dd4c92aa8e9c73bc98710f47`。
- [Full 38127563455](https://github.com/warpdotsys/reader-dev/actions/runs/38127563455)：本提交完整镜像required job15分22秒成功，7项按PR事件不适用的optional job跳过，不能写成全工作流零跳过。实际Cam24、Chromium23契约均零跳过；打包worker HTTPS10正例＋2证书负例/24目标请求/334头字段，实际Reader HTTPS8场景/12HTTP/14CONNECT/178头字段均独立核验。默认入口真实生成注册、登录、导入、阅读、翻章/刷新与退出1旅程9.051秒/零跳过，4份生成PNG主代理实际查看，中文可读、不空白；未使用真实账号/私人正文，不等于生产登录已验证。[独立Full回执](evidence/reader-route-hosted-3e225f97-independent-full-acceptance-2026-10-11.json)，SHA `b5cb199e5904feff4cc58edd39d5a0dd05c9db66bd3005d0fe120bf11094f41c`。JAR仍准确be87；最终服务预算峰844,333,056B/PID193/实际swap0，但普通Full允许1GiB swap，客户端在服务预算之外，不冒称合计2GiB/零swap上限压力验收。两项HTTPS各自零swap上限预算独立记录，不将峰值拼成同一轮。
- [Native 38127563397](https://github.com/warpdotsys/reader-dev/actions/runs/38127563397)：六个required实际job全部成功且独立接受，runner均GitHub托管Ubuntu24.04/24.04-arm，不再自托管。共享JAR准确be87；两原生架构各自导出、按完整字节转移到新job、重导入后用全新生成用户/存储实际运行；publisher无registry凭据，仅演练，32份小JSON重新核验。普通四轮预算各允许1GiB swap、观察swap均0，最大峰844,861,440B/PID202；不是H/I/K合并零swap压力轮次，也不是所有峰在同一轮。[独立Native回执](evidence/reader-route-hosted-3e225f97-independent-native-acceptance-2026-10-11.json)，SHA `f448be521c448ec045f55add9d7c83588ddd851e507bb98af0f246bf452a395c`。本机只下载小报告及准确JAR，未下载或重算大镜像archive，身份以下为托管实际转移报告，不能冒称本机完整镜像已验收。

| 架构 | 托管实际archive SHA-256 | 托管实际重导入image ID |
| --- | --- | --- |
| amd64 | `16d2abb59f5f0a6bbb1b555db0f92871813128f5fc16636e205119379c1d82ae` | `sha256:8d843e2ce91b36fbf4c57ac2b96c52ae6b23c0216c26f97f12fae9c12c16eafa` |
| arm64 | `4a45b551a59e3e5ef38c7b4a231a949822031592bc4be9768e3de1d409967524` | `sha256:470d980211b39bb6816e3bf820340a2c26b5ee423d67519b6f72331f33e77e96` |

注意Linux干净产物 SHA为 `be87f00490fc53a0462c48b06cc5500c6fea80ee07ee3f48b2fde035eb83eeab`、285,672,055B/1570条目，并非本机增量测试输入38522。第一份独立核验按38522期望而拒绝，原helper/失败保留；随后仅将新Linux helper绑定自己的真实双clean身份，旧守卫不放宽。已从本Native run的artifact11697809064下载准确JAR并将副本设只读，完整SHA与自己的Java双clean一致。相对本机38522及旧托管f5，内容差异各仅111个Vue资源；1122个class、90依赖、worker与全部非Vue资源不变。[准确下载和三方逐entry核对](evidence/reader-route-hosted-be87-jar-local-and-baseline-delta-2026-10-11.json)，SHA `93b285d638897dd79aaf4fe9093a325333ea24194f12ed4f1c81ff9929ba46bb`。未重算提供方ZIP archive digest，JAR内容已独立重算；不下载大镜像、不读取私人正文。跨OS字节一致性仍false，本机Chrome成功不能直接记在be87名下。

[本机依赖边界回执](evidence/reader-route-local-installed-dependency-boundary-2026-10-11.json)，SHA `3001d8457be399d550a381523d79915eb7ec182fb1716cb3189770e8cf2f2320`：只读核对四个包，plugin-vue与Vue两项版本不匹配，Vite7.3.6/TypeScript5.8.3版本匹配；现有依赖目录未改。源码行尾是另一个候选变量：本机已安装plugin-vue6.0.9的生产scope-ID计算含原始source，tracked Vue中的行尾不同可能传播至scope-ID/CSS/JS/chunk哈希。该API证据本轮只读自6.0.9，未在锁定6.0.8实物核对，也没有控制变量复建；不能认定行尾是唯一根因或声称跨OS可重复构建已完成。

已知边界明确区分：同章不同 `epubAnchor` 的history未被当前watch处理（维护源码确认，实际浏览器未复现）；临时书提醒与迟到媒体/HLS的新增门控仅编译/静态核对，尚未真正页面验收；大型真实EPUB合并预算失败及OPDS未实现均保留。旧JDK/Spring及固定旧版Actions的弃用警告需要独立维护，警告本身不等于该CI执行失败。

四份回执均对应产品3e225/快照f180，不能用它们接受后续产品提交。GitHub当前默认分支仍为legacy，PR56仍OPEN/draft、head3e225；未合并、registry发布或部署。官方uidmap `1:4.17.4-2ubuntu3` 已安装，newuidmap/newgidmap可用；两映射文件SHA仍 `d796e52bc335df4e55114fad949f19850e6b4008cf07bf8d53a4e88936be9cbd`，没有扩大映射、改代理/生产或复制浏览器会话。该前置依赖可用本身不证明最新完整镜像已在本机非root运行。最后仅流式哈希复核58报告、原件/只读备份和授权单书均未改，不查看报告正文。
