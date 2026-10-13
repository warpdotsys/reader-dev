# 前端依赖安全补丁候选

## 已从官方公告核验

本轮生产依赖审计原有五个 high 包条目，不等于五种全部可达的攻击。只更新下列修复族，不引入全量重写或后台存储迁移：

| 依赖 | 原锁定 | 新锁定 | 修复依据 |
| --- | --- | --- | --- |
| Axios | 1.19.0 | 1.20.0 | [请求选项原型污染修复](https://github.com/advisories/GHSA-vh66-26gq-q6x8)及同版本相关公告 |
| Vue 及同版 @vue 族 | 3.5.40 | 3.5.42 | [SSR 属性名缺失回车过滤](https://github.com/advisories/GHSA-g2v6-rqmx-r4w6) |
| nanoid | 3.3.16 | 3.3.18 | [零尺寸生成器无限循环](https://github.com/advisories/GHSA-2v37-7h3g-55p8) |
| source-map-js | 1.2.1 | 1.2.2 | [索引映射偏移放大](https://github.com/advisories/GHSA-68fv-2mgg-jv7q) |

Vue/Axios 直接依赖使用固定补丁版本；两个传递依赖在既有父范围内更新锁，不添加强制 override。锁生成工具顺带刷新的 Babel/parser、sourcemap-codec 已还原，Linux libc 元数据保留。静态对照确认只有目标族记录改变，未增加或删除其他包。

## 已验证与尚未验证

已对新锁执行只读 `npm audit --package-lock-only --omit=dev --json`：生产漏洞条目为零。语法检查及仅目标族改动检查通过；原 JAR、获准单书和 58 份用户报告哈希不变。该审计只反映当时已登记的依赖公告，不证明全系统没有漏洞，也不代替真实请求、登录、Cookie、阅读或完整镜像验收。

新增十项回归检查锁/实际安装版本，并执行真实库：零尺寸 nanoid 不调用随机源；源码映射拒绝超大、非法和嵌套累计偏移；Vue SSR 不把回车属性名变成事件/自动聚焦；Axios fetch 在仅回环的生成 HTTP 请求里保留显式方法与生成鉴权头，不采纳污染头。危险样本使用独立子进程、128 MiB JS 堆和五秒期限，超时必须失败，不能把没有崩溃当成修复。测试不导入用户密码、Cookie、真实账号、正文或浏览器会话。

独立最小样本最终十/十通过，测试与正式 helper 源码完整字节一致，不是 stub 或伪实现。首轮三个通过/两个失败，纠正污染描述符不可写和索引映射列边界样本后，仍有真实 Axios 反例：1.20.0 的已解析 Request 包含正确头，但第二参数缺少自有 headers/method，Node 24.19 的 fetch 最终采用原型继承的头。没有删除原断言或宣称仅升级版本就修好。

因此新增可读 `readerFetch`，只接入 Reader 中央 Axios 实例的 env.fetch：保留已解析 Request 的方法、实际头、body、Cookie 策略、取消信号及其他传输值为显式自有字段；不改变默认 XHR 选择或原 15 秒期限。Axios 显式 manual 重定向仍优先。回归另覆盖真实 JSON/FormData、取消和禁止跟随重定向，以及输入/冻结选项不变和错误原样传播。Node FormData 默认 MIME 与守卫无关，两边对照同为 urlencoded；测试采用显式 multipart 声明核对 native 边界，没有把错误 MIME 当产品守卫修复。真实浏览器业务仍须自己的新托管流程复验。

本机宿主物理可用内存不足，不启动 WSL、Firefox 或完整 Gradle 构建；当前旧 node_modules 也不能冒称已安装新依赖。只建立独立最小依赖样本核对新回归，整前端、类型检查、双 clean、实际业务和单镜像均交给本提交自己的 GitHub 托管 runner。不得沿用 b2ed 的绿灯证明这一补丁。

[独立真实库样本、未接受反例及源码摘要](evidence/frontend-security-deps-local-2026-10-13.json)。本轮只证明这些回归样本和 Reader 中央 Axios 的防御范围，不宣称所有原生 fetch 或其他环境均可抵抗任意原型污染。源码守卫不执行用户脚本，不清理用户浏览器会话。

## 重建与回滚

项目 `web-vue3` 中运行 `npm ci`、`npm test`、`npm run build`、`npm run audit:security`；再用 JDK 11 执行项目的 `gradlew.bat -PreaderWebUi=vue3 bootJar --max-workers=2 --no-daemon`。审计需要 npm 官方服务连通，失败不自动放过或改源。类型/接口/原版排版和双架构产物必须以新源码及实际报告复验。

回滚只恢复本轮 package/lock 的依赖记录和新增安全回归/审计脚本，然后重新冷安装和构建；不得恢复或覆盖用户 reports、storage、原始 JAR或 Go/Rust/旧项目。b2ed 的源码和准确 8f2e JAR 作为此前对照保留。两轮 529b 大书八/八仅属于旧依赖组合，不能移借为新依赖已通过；准确产物本机、剩余三方/实站/长时仍为缺口，未合并、发版、部署，Goal 未完成。
