# 生成详情三方差分：默认时间字段契约

更新：2026-10-07。当前是附加验收工具与实际历史两侧证据，不是起点故障已修复或当前最终镜像三方通过。

## 已从原 JAR 验证

原件 SHA-256 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`。仅只读提取 `BOOT-INF/classes/io/legado/app/data/entities/Book.class`，类散列 `5aa525ae490d03a64013209a3354af95a357aff83258059c6a1a9545b85ba1ea`，没有修改 JAR 或字节码。

JDK 11 `javap -c -p -s` 的完整默认构造器显示三次 `System.currentTimeMillis()`：

| 字段 | 默认构造器调用偏移 | 暂存参数槽 | 主构造器赋值偏移 |
| --- | ---: | ---: | ---: |
| `latestChapterTime` | 192 | 18 | 139 |
| `lastCheckTime` | 205 | 20 | 145 |
| `durChapterTime` | 273 | 27 | 181 |

恢复源码 `Book.kt` 保持同样的系统时钟默认值。因两次实际 API 调用发生在不同时间，不能要求这些新建对象时间戳逐值相等，更不能直接把它们删除后声称完整原始 JSON 相同。

## 新的附加工具

`scripts/reader_metadata_differential.py` 复用现有默认八布尔诊断规则和有界 8 秒只读脚本，固定散列分别为 `603a86357f45c3b850ec3c8cce2fe98266053aa1276fdf7178fbd5a4fe61166a`、`045f640185166e2d9415e09b1e2d574ba45cf6119345ba180f4fe6e5a9bf4b04`。书源只改为隔离 loopback 地址及生成测试名称，不增加登录、正文、购买或外网规则。

页面初始没有书名／作者／封面选择器；300 ms 后由生成页面脚本插入虚构测试 DOM。名字沿用“黎明之剑”，但没有使用真实 EPUB 或网络正文。新增选项：

- `--exercise-metadata`：先执行原五例搜索／脚本／POST／Cookie 门禁，再保存并读回元数据书源，各侧实际调用一次 `/reader3/getBookInfo`，保留 HTTP 状态、完整 `ReturnData`、所有默认字段、目标 GET／Cookie 与规则往返回执。默认模式仍要求完整 JSON 逐值相等。
- `--metadata-clock-contract`：必须显式搭配前者；在实际 HTTP 调用前后记录同一宿主系统时钟窗口。三个原字节码确认的时间值必须是整数并落在各自不超过 35 秒的窗口内。其余完整 JSON 的字段、类型、值、嵌套内容和额外字段仍严格相等。原始观察不改写，限定结果明确记录 `fullRawReturnDataEqual`；不得把此模式叫作字节完全等价。

两个选项不能与 UTF-8 六例诊断组合。旧五例和旧 UTF-8 严格红灯都不放宽。缺真实观测、规则丢失、Cookie 外泄、缺时间值、布尔／浮点冒充时间、过期／未来时间、未知其他时间字段差异均拒绝。

历史两侧先实际通过，宿主停止自己的旧 renderer、再次确认相同仅 loopback 网络命名空间，UID 10001 探针确认 8050 已拒绝连接后，才允许 Camoufox 阶段。预算与清理规则不变。

## 已执行的真实差分

授权根 `/var/tmp/reader-generated-fragments-20261003.RMZay2`；使用现有只读 JAR 副本，不重新上传 JAR／正文，不改生产。恢复副本散列 `221d41efbab68fa61e2117a4db888c1d8996d671c99f4a034c233c55002d1a43`；承载环境镜像 `sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf`，不是本增量新构建镜像。历史 renderer 固定 `hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`，并不证明它就是原 JAR 生产时的确切 renderer。

### a：逐值比较失败，实际观察完整保留

`reader-threeway-metadata-20261007-a.service` 已终态失败。原件和恢复远程侧各五搜索／五目标请求严格通过，各一次实际详情 API／一次目标 GET，HTTP 200、`isSuccess=true`、`errorMsg=""`，22 个详情字段、默认值及页面诊断均保留。完整 JSON 唯一差异是上述三个时间字段：原件均 `1791378013869`，恢复远程均 `1791378034876`。其余完整观察逐值相同。

默认严格门禁正确拒绝 handoff，Camoufox **未执行**。共同峰值 1,165,811,712 B，2 CPU／2 GiB／零 swap／512 PID 总预算；内存 max／OOM／PID max 为 0，`sock_throttled=1` 原样保留，不称所有计数为零。两自有容器已移除，精确归属标签查询剩余 0，两个 JAR 不变。

### b：UID 10001 入场权限失败，不是 Reader 失败

开启显式时钟契约的 b 在 Python 打开脚本前失败：root 创建的 `0700` 挂载目录不可由 UID 10001 读取。探针 exit 2，原件／恢复 Reader／Camoufox 均没有执行，不能冒充业务差分。峰值 112,738,304 B；两自有容器已清理，精确标签剩余 0，原报告保留。

### c：已收终态，原件侧超时，三方未完成

只在新 c 目录把公开生成脚本挂载根设为 `0755`、六源文件设为 `0444`；六个源文件总计 107,836 B，字节与 b 一致并经 `sha256sum -c` 验证。未提高权限或预算，UID 仍为 10001。

`reader-threeway-metadata-20261007-c.service` 的 Invocation ID 为 `9a778621e52246daa1058eaa386374a7`。外层服务 CPU 200%／2 GiB／零 swap／8 分钟时限／30 秒停止时限；内部所有 Java／浏览器后代仍共同受原预算约束。运行中 SSH banner 查询与首页 HEAD 超时，没有据此重启测试、改变代理或生产配置。

连接恢复后实际取得终态：服务 failed／MainPID 0／主进程 exit 1；主控报 `Three-way probe exceeded its bounded runtime`。原件失败回执只记录 `TimeoutError`、完成搜索 0、目标请求 0、详情调用 0、最后响应及元数据 null，没有记录精确 API 阶段；不能唯一归因为 readiness、newContext 或元数据脚本。恢复远程／Camoufox 尚未执行，时钟契约没有得到真实三方验收。空 `probe.log` 也保持空，不补造具体异常。

共同峰值 529,154,048 B，内存 max／OOM／PID max 为 0；两个自有容器已移除，独立精确标签查询剩余 0，两个 JAR 不变。a／b／c 三个服务均独立核对 MainPID 0，不能借 a 的成功观察填补 c。

测试清理后的宿主观测：总内存 4,100,820,992 B，可用 473,833,472 B，2,147,479,552 B swap 已全部使用。该数据是宿主所有业务合计，不是测试组“使用 swap”，也不足以唯一解释运行中超时。没有停止、修改或读取其他服务的内容；不在此容量状态下追加重负载测试。实际三方仍待资源充裕、身份已验证的隔离环境。

[有限证据与逐文件散列](evidence/generated-metadata-three-way-2026-10-07.json)区分这三轮。原始生成报告保留隔离目录，不上传真实正文、凭据或用户报告。

## 验证与可重复命令

本机 Windows Python 全套 323 项：322 通过、1 项 POSIX 专属跳过；新增 24 项在 Windows 和 WSL 均全执行通过。发布流程结构 49 项通过。它们验证生成 fixture／CLI／报告门禁，不代表实际浏览器三方通过。

```sh
python3 -B -m unittest discover -s src/test/python -p '*_test.py'
python3 -B -m unittest discover -s src/test/python -p 'metadata_three_way_test.py'
node --test scripts/check-release-pipeline.test.mjs
```

真实三方命令只允许在已经核验输入、仅 loopback、无外网的专用 Linux 隔离环境执行，输出目录必须新建且不存在；不能在用户现有 Reader／Windows 上裸启动原件：

```sh
python3 -B scripts/run-three-way-webview-in-docker.py \
  --original /validated/read-only/original.jar \
  --restored /validated/read-only/restored.jar \
  --runtime-image sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf \
  --output /validated/new-generated-only-output \
  --exercise-metadata --metadata-clock-contract
```

本次同步提交前一轮可选结构诊断的有限失败结果，避免只为结果文档重启完整镜像构建。新增测试随现有 GitHub hosted runner 的 `unittest discover` 执行，不引入自托管 runner。

## 已知问题、未验证与回退

起点匿名实站仍失败，真实认证三方未完成；不绕过验证码、不重放可能有副作用的脚本。旧 UTF-8 44 B／正确 60 B 差异、历史 newContext／readiness 故障仍保留。c 已结束并清理，但没有真实详情结果；当前最终镜像长期容量和生产用户关键流尚未获得本次接受证据。不标记完整目标完成、不合并 draft PR、不发布稳定版本或部署生产。

回退只是不传新增两个选项，即保留原五例／UTF-8 流程；或撤销新增 helper、CLI 分支及其测试。没有业务／存储迁移、字节码热补丁或前端替换，58 用户报告散列已核对不变，不 reset、覆盖或删除现有工程。
