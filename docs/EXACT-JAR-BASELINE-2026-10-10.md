# 准确原件的本机隔离基础对照

## 身份与范围

2026-10-10 实际在本机 WSL 顺序运行用户原件 b26 JAR 与已接受的 c594 CI e59 JAR，未使用公共错版 7222 归档、上传原件、复用真实账号/Cookie或读取私人正文。两份独立副本都在同一总预算内拷贝并完整流式 SHA 校验，只读挂载；原件原路径保持不变。

- 原件：`b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`，72,913,887 B。
- 当前构建：`e59a01bf190de757f73216557ff353411019507d25bad76f0c808b48366adb84`，285,665,012 B，来自 Native37946438729 的共享 JAR 制品11624549044。
- Java/runtime来自已验8d／3fe镜像只读rootfs；Java实际为Temurin11.0.32.1+1。两侧直接运行各自JAR，不以新版入口写入的release identity代替原件证据。这不是最新完整Native镜像的本机运行。

独有systemd预算、rootless OCI user/mount/pid/net/ipc/uts隔离，Reader UID/GID10001、附加组空、能力全零、禁止提权、仅loopback；宿主核对实际PID/cgroup/net与四处只读挂载后才授权Java启动。总限2 CPU／2GiB／256PID／零swap，memory.high=1.5GiB；账户及两侧storage分别新建，未导入真实数据。历史WebView、Camoufox、可见界面与真实书源均未在本轮执行。

## 实际失败及纠正

| 轮次 | 原始结论 | 父级峰值 B | PID峰值 | 秒 | 实际主程序退出 |
| --- | --- | ---: | ---: | ---: | ---: |
| a | JSON读取失败；未采到具体请求阶段 | 787,222,528 | 38 | 7.761 | 2 |
| b | 注册和登录成功，错误大小写书架路径404/HTML | 797,745,152 | 38 | 6.020 | 2 |
| c | 正确书架路径，四项选定比较通过 | 803,790,848 | 39 | 10.174 | 0 |
| d | 实际使用仓库探针，补token/默认字段，九项选定比较通过 | 800,038,912 | 39 | 10.228 | 0 |

a只有JSONDecodeError，不能唯一归因登录、请求头或产品故障。b增加有限响应元数据，确认原件注册／登录HTTP200、ReturnData成功，失败来自探针`/reader3/getBookShelf`而非正确的`/reader3/getBookshelf`；响应HTTP404、53B、HTML且没有重定向。对齐既有JSON请求头并补观察没有修改产品；c只改探针大小写，不新增兼容别名、删除用例或放宽成功条件。原a/b文件保持失败，不事后补造响应。

全部轮次父级max/OOM/OOM-kill/PID触限/high/swap为0，原件散列未变，精确清理后自有容器剩余0。c/d另外独立观察stopped/pid0和PID1实际主程序0，不用启动驱动0替代执行结果。宿主随后实际查询四个精确单元，全部inactive/MainPID0，不凭日志推断。[原始报告完整SHA和有限机器记录](evidence/exact-jar-baseline-2026-10-10.json)。

## d：具体通过项与不能扩大之处

两侧都实际启动、首页HTTP200；原首页5,625B／`c0eee9e643947b8064f0c956e00de68e322dc7a609f8761a5de75b9e2b540a8f`。新旧首页不作为同字节验收：原件Vue2与候选Vue3本来不同，本轮也没有实际渲染。

九项比较：systemInfo的ReturnData信封；注册与登录的静态默认字段；accessToken类型和长度；无Cookie客户端token退出前及退出后的书架完整JSON；空书架完整JSON；默认分组完整JSON；退出后原会话完整JSON。登录只明确排除三个随机/时钟字段`accessToken/createdAt/lastLoginAt`的字面相等，仍检查类型，其他默认／未知／null字段留在比较中；不是整个登录响应同字节或时间契约全部通过。

实际两侧token均为60字符字符串；WebDAV／本地／书源／RSS默认开启，bookSourceLimit100、bookLimit200。空书架、默认分组及退出后信封的实际响应字节各自两侧一致。Linux UTF-8下五个默认分组中文与字段一致，分组响应346B／`36b30d3bd57512aae03a7e8d577939487a467a0cdbb95e108d4b09dede246aee`；这不抹掉此前Windows编码差分或证明全界面没有乱码。

**已从两份JAR验证的安全语义**：POST logout后原会话返回`isSuccess=false/errorMsg=请登录后使用/data=NEED_LOGIN`，但无Cookie客户端持有的既有accessToken仍返回成功空书架。退出会话不等于撤销全部设备令牌；两侧一致，不擅自改变旧协议或宣称已实现令牌撤销。

## 可维护探针与生成回归

仓库源码为`scripts/probe-baseline-api-in-guarded-runtime.py`，本机d实际执行的工作区文件字节SHA为`d03f6ae2cbc6b0b46d0f96ec280063be97ce099d213e0a2135740e824b560adb`。它先拒绝错误单元／候选SHA／关闭assert，再核对非root、能力、网络、cgroup、原件SHA、宿主授权与原硬预算。固定独立新storage，非JSON只记录状态/类型/大小/SHA/布尔，不输出正文或token查询；生成凭据不发布。

只可在外部已验证的上述隔离runtime执行，不能在宿主裸跑原JAR：

```sh
python3 /verification-scripts/probe-baseline-api-in-guarded-runtime.py --unit reader-baseline-api-20261010-d.service --expected-restored-sha256 e59a01bf190de757f73216557ff353411019507d25bad76f0c808b48366adb84
```

主调用者仍必须先提供独有预算、只读两个JAR、PID1回收、loopback、宿主门禁和精确清理；这条内部命令不会替调用者创建隔离。固定d存储／报告不可重复覆盖，复验必须使用新单元和新目录。13项生成守卫不是实际JAR证明；本机初版427通过后再加两个真实CLI拒绝守卫，最终Windows429项／26.261秒／1项原平台跳过，WSL429项全执行／19.374秒；发版结构／制品／manifest的85项Node守卫全部执行通过，两个修改的workflow也实际解析通过。自己的托管CI仍需另验，不借文档版2c或父版32绿灯。

## 下一阶段与回退

固定历史WebView的index b61仍可从官方registry读取，AMD64子manifest为88c250…53573、config5d85fb…4e6bb、四压缩层合计379,489,069B，配置声明Node/index.js、8050与版本3.2.0。只进行了小型元数据读取，没有下载／执行大层、安装Docker、改宿主代理或部署；未独立复算这些远端描述符指向的原始blob SHA。它是可追溯参考，不证明曾是用户生产实例。

下一阶段仍要准确原件＋固定历史WebView、当前JAR＋同参考、当前JAR＋Camoufox的生成及真实站点对照。当前基础成功不能替代脚本、POST/Cookie、异步、SSE、下载、用户存储格式全范围、认证书源或生产界面验收，也不宣称全部目标完成。

回退仅停用这支增量探针，不回退产品、改认证语义、删除原始报告或用户数据。58份受保护报告和原始b26文件本轮完整散列再验未变；未合并、正式发版或部署。
