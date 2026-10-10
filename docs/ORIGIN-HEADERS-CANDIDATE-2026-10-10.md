# 原生逐跳请求头候选修正

日期：2026-10-10。**ac0bb598 自己的四条托管 CI、新 JAR 的有限 HTTP 三方已接受；HTTPS 发现另一个 Host 问题，后续可读源码修正须另验。没有正式发布或部署。**

## 本轮产物与边界

不能把两次源码修正混成同一个绿色版本。下表仅记录各自实际执行的内容。

| 对象 | 已执行并独立核验 | 尚不能代表 |
| --- | --- | --- |
| ac0bb598／tested 1bf74f4f | 四条 GitHub 托管 CI、双架构构建/重导入、新 JAR 574ec14d、准确原件 HTTP 三方 | 后续 Host 修正、HTTPS 全部安全、真实登录或生产 |
| 后续 worker c3ef2486 | 取消初始 POST 的 Route.headers；表单缺省类型由原生监听器仅设置一次；497 Python、JS 24 项、10 HTTPS场景／24实际请求及2个证书拒绝控制 | 尚未重新构建的 JAR、自己的新 CI、全业务与正式发布 |

ac0bb598 的 [Java](https://github.com/warpdotsys/reader-dev/actions/runs/38038919034)、[Vue](https://github.com/warpdotsys/reader-dev/actions/runs/38038919055)、[Full](https://github.com/warpdotsys/reader-dev/actions/runs/38038919035)、[Native](https://github.com/warpdotsys/reader-dev/actions/runs/38038919081)全部终态成功，runner明确为GitHub托管ubuntu-24.04／ubuntu-24.04-arm，不是自托管。独立回执分别为[Java](evidence/origin-header-hosted-java-2026-10-10.json)、[UI与重建](evidence/origin-header-hosted-ui-java-2026-10-10.json)、[Full](evidence/origin-header-hosted-full-2026-10-10.json)、[Native](evidence/origin-header-hosted-native-2026-10-10.json)。Java的495 Python在Linux零跳过，194 JVM／35原环境跳过；强制浏览器阶段另行跑满24 Cam契约零跳过。新增父级清理XML的三例实际运行、零跳过／失败／错误，包括杀worker后清理和不跟随符号链接。Vue 25实际流程零跳过、296前端检查和20截图守卫；本轮实际查看了生成手机登录与听书设置截图，中文可读，不等于全UI或线上登录验收。Full的7个可选作业未执行，不能当作准确原件三方或长测证据。

两次干净构建字节相同：SHA `574ec14dc7b35daed93969e670fe6f5ed7a6ef933a32fbfc0e867a0f66bbff18`，285,670,961B／1,570条目；本轮从Native的共享JAR作业仅下载一次并重新计算字节SHA，不另下载整镜像。Native六作业和24个publisher小JSON独立接受，amd64／arm64及转移重导入都是同一个574产物；镜像最大实测897,863,680B／PID201，普通允许swap但实测0，不能冒充本机零swap测试。

新574 JAR的[m轮有限实际三方回执](evidence/origin-header-current-jar-three-way-2026-10-10.json) SHA `3a8aa5344f106e336212fd7f3d227cf000639c20fc0c90dcaa9434b45778d72b`／57,360B：原件b26和固定历史WebView不变，4组／12完整ReturnData均成功且逐字相同；25追加目标请求中原件与历史模式仍泄露跨站认证，当前JAR的Cam同源保留、跨站为空，POST303仍正确跟随并执行脚本。五项旧业务契约也实跑；严格三方安全仍为false，因为不能把旧引擎的危险行为当安全兼容目标。57.996秒／aggregate2CPU2GiB／high1.5GiB／256PID／零swap，峰值1,611,636,736B／PID245，触限/OOM0；参考进程停止PID0后才允许Cam，独立清理0。复用锁定runtime不是最新整个本机镜像验收。仅删除已停止m轮的两个JAR临时输入副本358,584,848B，原件、只读备份及三份已知CI下载均再验SHA并保留，可重新复制恢复。

## HTTPS 新发现与正式源码修正

前两轮临时策略路径没有被实际浏览器加载，连有效证书也因未知颁发者失败，均保留失败。只读检查锁定浏览器内的真实策略实现发现：已存在安装目录策略时，备用路径不是本轮有效的信任入口。依照[Mozilla Certificates策略](https://firefox-admin-docs.mozilla.org/reference/policies/certificates/)，仅在隔离OCI内用私有1MiB tmpfs覆盖安装目录的策略视图，保留原策略并加入临时生成CA。没有更改主机／共享缓存／生产证书库，没有关闭证书验证，也没有替换worker或浏览器二进制。有效证书可访问；错误域名被`SSL_ERROR_BAD_CERT_DOMAIN`拒绝；未安装的第二个正常CA签发证书被`SEC_ERROR_UNKNOWN_ISSUER`拒绝。

d轮10场景／24请求虽完成，独立逐字段检查发现表单POST跨站303的`Host`仍是原站，CONNECT却已到新站。e轮仅从Route.headers字典删除Host/Cookie/传输字段仍复现，故两轮均未写成功验收回执。HTTP Java代理会重新构造Host，原24项HTTP契约未覆盖这个TLS隧道里的原生字段问题；绿灯不能代替验证。[Playwright明确说明](https://playwright.dev/python/docs/api/class-route#route-continue)覆盖头会随重定向传播，本轮以目标实测为准，不凭文档的禁改字段说明假定Host安全。

后续可读worker完全取消POST的Route.headers。已有原生监听器仅对初始准确URL的首次POST设置缺省form类型；GET、其他URL/源、后续网站POST不消费或重用该缺省；显式类型包括空值不覆盖。方法／正文仍由原生浏览器导航，不抓取并伪装响应。无规则GET仍不创建扩展；无规则的合成form POST为保持旧行为也必须建立私有监听器并等待就绪。增加五项准确执行生产JS的守卫、无头POST策略守卫、不传Cookie/传输覆盖守卫和实际HTTP目标Host断言。

f轮观察器超时，保留`TimeoutExpired`，自有容器已经独立停止PID0并清理，不是TLS成功。g轮只做新c3ef worker的定向HTTPS POST303：两个实际目标/CONNECT的Host均准确，同源认证保留、跨源为空；23字节缺省form POST转为无正文无Content-Type的GET，生成DOM脚本正常。h轮已取得12份实际观察，但收尾检查后进程恰好自行退出，TERM命令返回非零，故没有最终成功/资源回执；另行独立确认该准确自有容器stopped/PID0后仅删除它的运行状态，保留原stdout，不事后伪造原报告。i轮收尾观察器只对该准确CID的stopped/PID0确认接受这个竞态，不忽略其他错误、不延长期限或增加预算。

i轮[完整有限HTTPS独立回执](evidence/origin-header-worker-https-2026-10-10.json) SHA `f32fb478f01480e03d7af4a787dbdb3247cc585d8f48c14187f6fb283c285983`／58,020B，原报告SHA `3233a6ba2dd9fe0666d8b48be0b547c12e302b9ea3ac0abadd75a27fe6ab6c25`。直接加载冻结c3ef可读worker，未启动JAR/真实账号。10正向场景／24目标请求／334解析字段：每个实际HTTP Host均对应CONNECT真实目标，目标侧连接源端口也确认属于代理；全部同源认证/规则头保留、跨源为空，User-Agent保留。四组显式JSON同/跨源307/308保留39个UTF-8正文原字节、类型和SHA；同源Secure/HttpOnly/hostOnly/Path Cookie正确重放，跨源不发送；页面和资源跳转实际执行并返回生成中文DOM。缺省form的303保23→0字节、POST→GET，后续GET不补Content-Type。错误域名和未信任第二CA的两负向控制分别返回准确错误码，均在发出任何HTTP字段前拒绝，代理只观测到TLS握手。

102.42秒、aggregate2CPU2GiB／high1.5GiB／256PID／零swap，峰值1,611,321,344B／PID201，max/OOM/PID触限0；主机独立身份／只读输入和私有策略tmpfs检查通过，原共享策略SHA未变，自有容器stopped/PID0并清理、清单0。没有更改系统／生产信任、关闭TLS校验或替换worker启动逻辑。最新Python共497项／80.254秒／失败0／1原Windows POSIX跳过；先前Node守卫仍要求旧19项时明确拒绝，更新为实际新增后的24项后重新完整通过，不把失败回执升级为成功。

这些是直接worker的有限HTTPS源级证据，不是新JAR的TLS黑盒、Java CONNECT出口完整验证、WebSocket、真实认证、最新整镜像本机或生产验收。新修正必须取得自己的JAR/托管CI；仍不合并、发版、推送registry或部署。

以下章节保留a8e612原候选捕获时的历史状态；后续托管/新JAR变化以上方按版本记录为准，不能将其改写成当时已通过。

## 改动与目的

原件和 a873 恢复 JAR 的[实际三方诊断](HEADER-SECURITY-CHARACTERIZATION-2026-10-10.md)显示跨源认证头外溢。本候选不复制该风险：不再把书源头设置在整个浏览器 context，也不把它们复制进 POST 导航覆盖头。

每次渲染建立可读的 Firefox 原生请求头监听器，在每个 HTTP/HTTPS 请求发出前按实际 URL 的 scheme/host/port 选择头；同源应用规则值，跨源剥离继承的规则值，但不错误删除网站脚本自己的不同认证值或浏览器原生 Accept。Cookie、User-Agent、Host、Content-Length 和 Proxy-Authorization 不进入该规则监听器。配置上限为64项／8192 UTF-8字段字节；非法字段、控制字符、大小写重复键和越界拒绝，不截断。

跨源307/308仍可按浏览器原生语义保留POST正文与Content-Type；认证和自定义规则头不随正文传播。303改GET的缺省表单Content-Type不重新加入。所有跳转、脚本、Cookie、页面地址及流式读取继续由浏览器处理，不用请求库抓取并伪装成浏览器响应。

[Camoufox官方扩展接口](https://camoufox.com/python/usage/)支持本地解包扩展；[Mozilla原生发送前事件](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/webRequest/onBeforeSendHeaders)提供阻塞请求头修改。本候选增加确定性就绪握手：随机`.invalid`地址的空HTML只在内存回填，无DNS或套接字；只接受本扩展、准确地址与随机nonce的回复。就绪前不导入Cookie、不发业务请求；仅重试私有bootstrap，最多16次且沿用递减期限，绝不重放业务导航。没有web-accessible资源，不把认证值放进URL、HTML、日志或Git。

另修正Firefox loopback代理绕过：`<-loopback>`不是Firefox的有效覆盖方法。设置`network.proxy.allow_hijacking_localhost=true`，让本机目标也先经过现有Java出口检查；**不是放开私网权限**，Java默认拒绝策略、完整DNS答案检查与IP固定保持不变。依据[Mozilla实现](https://searchfox.org/firefox-main/source/netwerk/base/nsProtocolProxyService.cpp)及生成实际代理观测，而不是仅凭配置存在推断效果。

父进程给每个worker分配准确、私有的临时目录（POSIX 0700），把TMPDIR/TEMP/TMP限制到其中。正常结束、异常和强制结束后，父进程仅清理这个准确的新目录，不跟随内部符号链接；避免worker被杀后留下请求头配置。此父级新增路径的JVM强制终止/符号链接测试已写入，尚待自己的托管编译实跑。

## 已实际验证

最新j轮直接加载冻结的可读worker，SHA `a8e612873a5a46c222ce848d8dfe83b103db3e5a19c638be01a69e99c6f94baf`，不启动JAR。固定只读runtime image `sha256:401d895c31a39634bf29654a7fbb29527272d287c8c1219b12794bbe1970bc44`，UID/GID10001、无附加组/能力、noNewPrivileges、独立命名空间、只有lo，主机独立检查通过后才准许执行。

10场景／24实际目标请求／310解析请求字段，全部经过严格限定两个loopback目标的测试代理；同源认证/自定义头保留，跨源对应头均空，生成User-Agent保留。4场景的同/跨源307/308均POST→POST，39 UTF-8字节和SHA一致，显式JSON Content-Type保留；同源重放HttpOnly Cookie，跨源不发送源站Cookie，返回结构化Cookie保留Path/hostOnly。303仍POST→GET／23→0字节。页面脚本跨源导航、同源和跨源脚本的307/303均实际执行，正文脚本返回生成中文书名。没有靠阻止脚本加载取得“无外溢”。

30.285秒；aggregate2CPU／2GiB／memory.high1.5GiB／256PID／零swap，峰值1,611,390,976B／PID214；max/OOM/PID触限0。独立确认自有容器stopped/PID0并删除，清单0。未读取正文、账号或浏览器会话，不改生产、映射或系统代理。

[有限独立回执，含完整24请求字段与生成worker结果](evidence/origin-header-worker-http-2026-10-10.json)，SHA `fc7985b98b05142a2482163486c6648116c013db4913ae6c95984b852b3e4ea2`／86,019B；原始j报告SHA `6b889cb7a105366d8a7da62e20bff34613bc04553dca167d81ba51c48067d73f`。回执独立重验实际字段、方法、正文SHA、Content-Type、Cookie和资源/身份，不仅相信探针的passed布尔。

本机Python495项／26.856秒／失败0／1项既有Windows POSIX跳过，包括精确执行监听器JS的19项Node语义守卫；这些不是浏览器/JVM证据。正式Cam契约增至24项，四组新增不可跳过：原生头作用域、POST跳转/Cookie、脚本/资源跳转、loopback不得绕过Java出口。报告守卫拒绝旧20项或新增项跳过；新增两项父级JVM检查临时凭据目录正常/输出超限终止清理和符号链接边界。

## 保留失败，不把原语或旧绿色报告升级为验收

a轮错误浏览器selector失败；d轮测试代理缺CONNECT导致单跳请求库失败，不能冒称生产代理缺陷。e轮原生监听器启动竞态使前两场景没有同源认证，且所有请求绕过loopback代理；f轮扩展自身页面握手超时；均保留失败、清理自有进程，不覆盖结果。逐请求continue仍随303泄漏，单次fetch/fulfill丢失同源跳转认证，两种方案弃用。g轮补内存bootstrap和Firefox偏好后4场景／9请求通过，仅是原语。h轮为旧版本worker直接验证；i轮10场景虽完成，字段复核发现跨源307/308的Content-Type被误删，不能接受。仅j轮取得上述完整有限HTTP字段条件。

此前诊断提交1ac19cb4／tested3a06的四条GitHub托管CI均已独立接受，但它们构建的是**改前a873 JAR**，不是本候选。其Native六作业、双架构镜像/重导入及24 publisher小JSON已取得[独立原生回执](evidence/header-security-hosted-native-2026-10-10.json)，SHA `d49f4dd634ea0b07dd8012a4ac4e2797074abb42526471542cbc616d3640a3d8`；没有推送registry或部署。早期[三条阶段回执](evidence/header-security-hosted-partial-2026-10-10.json)保留当时Native未完成的真实状态，不事后覆盖。

## 尚未验证／已知风险与回退

本候选自己的JVM编译、24项实际Cam契约、双干净JAR、新JAR/准确原件三方、完整镜像和双架构都须重新运行；不能借1ac旧绿灯。HTTPS/TLS/CONNECT、真实登录、WebSocket头作用域、浏览器崩溃时文件清理、并发/长测、OPDS和生产尚未验收。当前仅HTTP有限子集通过，源代码新增并不等于认证安全整体完成，PR56仍draft，不合并、发版或部署。

该修改有意改变危险的跨源规则头行为；依赖跨站复用凭据的书源需明确修订，不能全局重新加头作为兼容回退。反射/JVM构造器和父级临时目录变更也必须由自己的编译和测试接受。无头规则仍不创建扩展/bootstrap；前端、业务存储格式及原件不变。无字节码补丁，原件和58个用户未提交报告SHA保持不变。

代码回退限于本候选worker、renderer和新增契约，保留独立诊断及原件；回退会重新引入认证外溢和loopback代理绕过，不作为生产安全方案。运行时临时目录只能在子进程终止后由创建它的父进程清理；不得对一般/tmp、用户目录或其他Reader进行递归删除。
