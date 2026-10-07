# 生成 WebView UTF-8 六用例三方复验

## 范围与输入身份

本轮扩展既有无外网生成三方工具，不修改业务源码、存储格式或前端。旧 [2026-10-04 五用例](WEBVIEW-THREE-WAY-2026-10-04.md)继续按原规则验证，不能代表当前 worker，也不能冒充新六用例验收。

- 原件：`reader-pro-3.2.14.jar` SHA-256 `b26fb4769d689d98ff26408ce79a275d719f360906c84acf52ff404e98030c8c`，沿用服务器授权临时目录内的只读副本，未重传或覆盖原件。
- 新恢复输入：托管 native run `37580306817`／源码 `09163efe`／受测 `3aec4f9e0102626c9f47b74b6130a872ae541b27` 的 `reader-release-jar-3aec4f9e0102626c9f47b74b6130a872ae541b27`（artifact `11465045291`）。实际解包 JAR 285,662,082 B，SHA-256 `221d41efbab68fa61e2117a4db888c1d8996d671c99f4a034c233c55002d1a43`；包内 worker SHA-256 `01863dcd45eab836bc0e76e2f0512d97656112a43c8f7dd1ffa3e3e8845087be`。上传后再次核对 JAR 哈希；不是本机 F9 构建、旧 74 恢复件或随后 594 的独立 JAR。
- 运行时：服务器既存只读 `sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf`，amd64／UID 10001，**当前业务 JAR 与 worker＋旧锁定运行时**，不是当前最终完整镜像运行证明。旧运行时自带 JAR 不执行。
- 历史引擎：`hectorqin/remote-webview@sha256:b61d8e86f69a743baa06aadb58cdba6d1e11c5baa45cec05a908d541ce9a684a`；仍不能证明原作者生产服务当年的真实版本。

输入、新 probe 和 run 放在原授权根目录内的新 `webview-threeway-utf8-20261007-a` 子目录。旧输入／旧报告保留；不联网，不读正文，不复制 Cookie／真实账号，不改生产挂载、代理、映射范围或用户 18931 Reader。新输入和脚本设为只读，容器 bind 只读。

## 新增断言与复跑方式

工具 `--exercise-encoding` 默认关闭。显式六用例模式必须同时启用隔离原件、实际历史引擎、Camoufox、原脚本和 POST 五用例；不允许 synthetic `/render.html` 冒充浏览器执行。原有四 GET＋一个 POST 和 Cookie 回放／删除约束保留，第六次为独立 `/search-utf8` POST：

- 明确规则 `charset: UTF-8`／JSON Content-Type，正文含中文、`𠮷`、`😀` 及 `+ & %`；保存书源后逐字符读回 URL 规则。
- 目标服务直接对收到的原字节计数／SHA-256，再严格 UTF-8 解码；60 B，SHA-256 `8d0383070028f4f457194a194e8e9133df2487261e341c1da121171d59eeeef1`。不从 Reader 摘要重编码后冒充原字节，不用替换字符吞掉错误。
- HTML 原书名故意为 `WebView编码原始书`；必须由 webJs 改为 `WebView编码书𠮷😀 + & %`。三方全部相同乱码也会判失败。
- 完整 ReturnData／书目字段按严格 JSON 类型比较；缺响应、额外目标请求、旧五次、错误 byte count／digest、布尔伪装整数、丢失 Cookie 删除或原 POST 不接受。
- 失败可保留生成阶段报告（完成的搜索／最后观测搜索响应／实际目标请求），不填补缺失数据。原件或历史配对未通过时不许可 Camoufox 交接。

只在已授权 Linux Docker/systemd 主机、精确已有运行时下执行，输出必须是全新 `/var/tmp` 子目录；示例中的输入和目录需先落实为授权目标，不能把原件直接在 Windows 上运行：

```bash
sudo python3 -B scripts/run-three-way-webview-in-docker.py \
  --original /var/tmp/authorized-test/original.jar \
  --restored /var/tmp/authorized-test/hosted-restored.jar \
  --runtime-image sha256:190e971278aa1530536b3c635d8bdeb344f0101e445bd2c61750f4417a62ffaf \
  --output /var/tmp/authorized-test/new-utf8-run \
  --exercise-encoding
```

实际两容器及其所有 Java／浏览器／probe 子进程合计仍为 2 CPU／2 GiB／零 swap／slice PID 512（单容器 PID 256），没有放行私网出口或发布端口。外侧 host PID 1 独立验证私有 loopback netns，UID 10001 probe 再核对 inode；历史两侧完成、原预算守卫通过、旧引擎实际停止和 8050 拒绝连接后才运行 Camoufox。保留累计峰值和触限事件，不重置、扣缓存或升预算。内部 360 秒 deadline，外侧自有服务 480 秒上限；只按本次随机 ownership label 清理自有容器。

## 本轮实际状态与限制

新工具纯测试全量 249 项：248 实际通过、1 Windows POSIX symlink 环境跳过，0 失败／错误；定向 33 项中 32 通过、1 同一环境跳过，包含新增 11 项。发布结构及 69 项 pipeline／身份／manifest 纯测试通过。纯 fixture／mock 验证不等于真实浏览器执行。

服务器已成功启动 `reader-threeway-utf8-20261007-a.service`，invocation `15dfdadf8378416895272a7681308b9e`。间歇 SSH banner 超时后，一次只读查询已确认终态 failed／Result exit-code／ExecMainStatus 1／MainPID 0；run 内保留 `three-way.original-failed.json`（211 B）、`probe.log`（2,994 B）、`provenance.json`（9,949 B），无完整 `three-way.json`。**该轮在原件阶段失败，尚未读到具体原因、原资源报告或自有容器清理确认，不能算新的六用例兼容验收**。后续复制报告再次在 SSH 握手超时，没有重启测试、改代理或升预算。下一次只读取此 run 的生成报告，不重复启动第二份。

即使该轮生成对照通过，仍不代表非 UTF-8 全部编码、真实网站认证／章节、当前完整镜像、长期容量或生产验收。此前本机起点解析／旧 worker 资源红灯仍保留。`594cec9c` collector 自己的 [hosted 分类验收](BROWSER-MEMORY-CATEGORIES-2026-10-07.md)单独记录，不拿它代替此业务差分。

回滚工具时不传新增 flag，可继续严格使用旧五用例；在干净检出撤销本测试工具增量即可，不影响产品 JAR／数据库。旧原件、输入、报告和失败结果保留，禁止 reset 用户工作区或忽略触限取绿灯。
