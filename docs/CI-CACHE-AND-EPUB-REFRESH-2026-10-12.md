# 托管缓存候选与真实 EPUB 刷新失败

最新：789 的准确正式 JAR 已与自己的双次 clean 重建核对，四条托管及 Native 六作业/32 个 publisher 有限 JSON 独立复验；Full 导入 GHA cache manifest、两个缓存步骤、207 秒构建及 58.2 秒发送完成，但稳定提速和严格 60 秒总墙钟限未证。本机同书 S 四/八、首次原样 starting 超时；R 六/八刷新 expanding 失败保留，不以生成绿灯替代真实阅读。未发版或部署。[正式产物与失败复测](EPUB-FORMAL-ARTIFACT-ACCEPTANCE-2026-10-12.md)。下文保留捕获时状态。

后续403：Full/Java/Vue接受，Native固定浏览器下载HTTP504拒绝，原红灯不重跑覆盖。Full缓存导出实际完成152.8秒，声明timeout60s不是已证总墙钟限；warm命中/提速仍未证。新下载缓冲与固定URL有限重试回到源码，326前端及755Python本机测试执行，正式产物与八场景待验。[最新部分CI、字节检查与回退](EPUB-DOWNLOAD-BUFFER-CANDIDATE-2026-10-12.md)。下列保留捕获时状态。

## 已成功重建：a80自己的四托管与产物

源码 `a80ccf23829cf167e158a9d34325697f27a75ae2`／受测合并快照 `e146d6d280018369a8a7bf9d826c4ead0cea3fc9` 的四流程全部终态成功。Java222项/35项既有环境跳过，新8项缓存测试stderr0；Vue25条生成旅程零跳过，包含91失败的搜索、阅读、分组、书架布局和听书缓存五项。没有重跑覆盖91红灯，也没有断言五项失败原本必然全是同一根因。

正式共享JAR已下载重算：285,673,713B/1568条目，SHA `d00784f6f54041de0786236fd1aadb66f7d2ff9967d82ad89dddc702b83621a7`，两次clean完整字节相同。与60d逐条比较，新增4、移除7个转移到helper的内联类型、改变6条目，全部属于缓存/BookController及Kotlin模块元数据；Vue资源、依赖、MANIFEST及浏览器worker未变。不是原件所有行为完全等价的证明。

[自己的有限托管回执](evidence/cache-publication-a80-hosted-2026-10-12.json) SHA `8cf52d6b…4cfa`：Native双架构构建、重装载和publisher六作业完成，32份小JSON经原消费器复核；748 Linux Python零跳过、Cam24接受。完整镜像的原XML及四张生成截图也由原消费器再验，登录/生成TXT及两章EPUB三模式/翻章刷新/退出12.905秒；登录与读章两图实际查看，中文可读不空白。Reader HTTPS8场景/12请求/178头字段/两证书负例，Worker HTTPS10正例/两负例/24请求/334头字段接受。

完整镜像普通服务端子集峰849,977,344B/PID198，配置允许1GiB swap（实际0）；UI客户端在服务端预算之外。publisher资源子集峰864,804,864B/PID208。没有本机下载/重算两份大型原生镜像，不把这些子集当本机最新整镜像或真实单书合计预算接受。

## 纠正91完整镜像取消的原因

原完整作业38235052988启动10:14:22UTC，45分钟左右取消。镜像构建10:22:24→10:58:14（2150秒）；锁定MCR Playwright基础层自身耗2113.8秒。HTTPS验证43秒完成，Reader HTTPS38秒也实际输出接受回执，随后整作业限额用尽，界面旅程未运行，保留步骤因无产物失败。不是已证实的HTTPS死锁。

a80同样的未加缓存构建51秒，四流程成功。这是另一次真实观察，不能把自然网络速度变化算成缓存候选的改善；具体旧下载缓慢的网络根因仍未证。[有限时间证据](evidence/full-image-locked-base-pull-2026-10-12.json) SHA `ba7ce8fb…ba6a`，原取消不删除，原45/70分钟作业限额及所有业务超时不提高。

## 可维护的构建缓存候选，尚未实测提速

Full与Native用官方Buildx及Node24 build-push-action，均钉完整SHA，不采用浮动tag。官方动作输入已从对应提交核对；[Docker缓存说明](https://docs.docker.com/build/cache/backends/gha/)和[官方构建动作](https://github.com/docker/build-push-action)是配置依据。仍由三个锁文件固定基础镜像index、wheel哈希和浏览器版本，不更换依赖或引入实时apt。

使用本次准备好的路径上下文（不是忽略已复制JAR的Git上下文）、单原生架构、`load:true/push:false/pull:true`。加载后照常验架构、JAR/revision、TLS、Cookie、资源与UI，随后只能导出/发布实际测试的同一镜像；无registry凭据或额外推送。GHA v2缓存scope按架构及三个锁文件哈希区分；内容寻址仍按实际Docker输入使JAR和revision变更失效，不能用缓存标签代替实际身份验收。

缓存导入/导出参数声明timeout60s，max模式；403实测导出152.8秒，不能宣称总时长严格不超过60秒。导出错误是可见的可选优化失败，不改变构建/测试失败状态。冷缓存、限流、容量或导出失败可能仍需慢拉MCR，不保证45分钟内完成，不称开缓存就已修好取消。22项负例及120结构/两个YAML已验，首次错误文案匹配修正后实际guard仍拒绝错输入。自己的warm命中、耗时改善与新产物仍须核验。

## R真实单书：六场景完成，刷新解压仍超时

准确d007正式JAR及唯一授权EPUB，在未改401d浏览器库的缓存runtime中，UID10001/仅lo/2CPU/2GiB/high1.5GiB/256PID/零swap，继续原八场景。只加外部只读、最多16个Worker静态资源事件，限定本机Reader的确切JS文件；不捕获URL、正文、Cookie或原头，不包裹Worker、不注入HTML。三个生成观察器测试与代码预检已验。

text805、text806、刷新text806、HTML806、raw806、raw805完成，前四完整正文与Q同字节。两次原版iframe高883，脚本/替换字符0，是真正正文而非裸资源URL；raw805刷新仍失败，回到文本的末项未执行。第二次模块请求200、JS媒体类型正常，已收到expanding确认，再遇15秒解压超时；不能沿用Q“尚未收到ready”作为本轮结论，更不能称八场景已完成或唯一根因已证。

[有限单书事件记录](evidence/authorized-epub-d007-worker-assets-2026-10-12.json) SHA `f5d3086f…efe5`，绑定原回执 `18bcc83d…fd34`。Java峰48/池8+8、总PID236且事件0，内存1,620,348,928B/high66595，硬-OOM0、swap0，75.911秒；预算子门槛接受，阅读总门槛拒绝。R实例和采样已停、inventory0；只清两份本次校验输入副本323,483,509B（约308MiB），保留来源、原件、Q/R旧失败及58用户报告；没有提交正文、逐章哈希、截图或原日志。

源码检查也确认 `destroyEpubDoc` 是可复用blob表回收，现有回归要求回收后可再生成资源；不能为了内存数字直接清掉其files/manifest而破坏该契约。下一步分别验证终端卸载与刷新重新解压的资源持有、流式合并和解码峰值，保持完整字节、大小限制、60秒下载/15秒Worker及八场景。真实认证、原件HTTPS/WS/WSS三方、OPDS等既有缺口仍未关闭。

## 回退

缓存配置仅影响CI构建，在独立干净工作树定向回退两个workflow及结构guard/负例后重新构建；不reset用户工作区、不动生产数据、不清GitHub报告来掩盖失败。旧钉死镜像与JAR继续作对照，原始JAR及只读备份哈希保持不变。本轮不合并、发版、推registry或部署。
