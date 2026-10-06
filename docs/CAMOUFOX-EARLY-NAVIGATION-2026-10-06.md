# 页面尚未 DOM 就绪时跳转的实际浏览器契约

## 状态与范围

新增可读 Kotlin 生成夹具，不修改产品 worker 或任何恢复逻辑。原 `c2679e7b` 的一次连续跳转 `Error` 仍未唯一归因；`1cf36e7b` 的 24 个独立 DOMContentLoaded 跳转通过，不能替代早期跳转，也不能把旧红灯追认为成功。

现有有限导航测试在 DOMContentLoaded 回调中跳转，因此新增 `generatedNavigationBeforeDomReadyReturnsFinalDocumentWithoutReplayingPost`：原 HTML 的解析被真实外部脚本阻塞，异步 arm 请求只在阻塞脚本已被请求后返回；浏览器随后在 `document.readyState=loading` 时导航至最终文档。最终请求到达才释放旧阻塞脚本。用请求握手及 loading 状态验证时序，不靠固定睡眠推断。

```text
初始生成 HTML（原 POST 一次）
  ├─ arm 请求 ── 等待阻塞脚本已请求 ── 返回
  └─ 阻塞脚本请求 ── 等待最终导航 ─────────┐
       arm 完成 → 页面仍 loading → 最终导航│
                     最终请求释放旧脚本 ───┘
                     返回最终文档
```

所有地址均是本次生成回环服务器，没有真实书源、账号、Cookie 或正文。仅测试时允许该生成回环访问；生产私网防护不变。生成 HttpOnly Cookie 必须保留，最终文档必须来自真正最终请求，原始初始请求／POST 必须恰好一次，原页面不能已 DOM 就绪。首次错误立即失败；异常附加的只有生成请求计数／时序布尔及原有固定 worker 类别。

两个握手期限各 5 秒，超时计数必须为 0；握手失败先标明夹具问题，不能直接归因为产品。取消旧脚本响应可能出现预期 IOException，夹具只关闭旧响应；它不会吞掉 renderer 的异常。测试结束仍关闭 renderer、回环服务器和线程池。不增加 goto、脚本或 POST 的重试，不放宽旧无穷跳转／超时／恢复／Cookie／私网契约。

## 门禁及本机检查

共享默认引擎门禁现在要求 16 个准确、无跳过／失败／错误的实际 testcase，包含新的早期跳转。已核对过的旧 15 项实际绿灯报告，针对新门禁会 exit 1；这表示新增覆盖缺失，不是把旧候选追认为产品失败。

本机报告校验夹具 12 项通过，涵盖旧 12／13／14／15 项报告的拒绝、缺失／重复／伪造计数／跳过与字节限额；全量 Python 172 项中 171 执行／1 个 Windows 符号链接环境跳过，零失败／错误。发布结构及制品完整性 Node 检查 43 项全部执行通过。它们是报告／发布守卫验证，不是真实 Camoufox 的新增场景验收。本机没有构建或启动新 JAR，也没有使用宿主 Chrome。

新的 16 项真实浏览器执行、产物构建及消费结果须来自本次新提交的 GitHub 托管 runner，当前尚未验证。测试失败须保留 XML 和固定阶段诊断，再依据实际阶段判断修复；不能因增加测试就宣称已修复或与旧失败同根因。

## 可重复入口与回滚

现有 `Browser image integration` 和 `Native release artifact rehearsal` 在对应源码提交触发。托管作业安装锁定的 Camoufox runtime 后运行：

```sh
PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 \
READER_CAMOUFOX_PYTHON=/path/to/pinned-camoufox/bin/python \
READER_CAMOUFOX_BROWSER_VERSION=152.0.4-beta.30 \
./gradlew -PreaderWebUi=vue3 test \
  --tests com.htmake.reader.utils.CamoufoxWebviewRendererTest \
  --max-workers=2 --no-daemon
python3 scripts/verify-camoufox-contracts.py \
  build/test-results/test/TEST-com.htmake.reader.utils.CamoufoxWebviewRendererTest.xml
```

路径由实际作业提供，不能填未安装的虚构 runtime 后把跳过当通过。共享 JAR 仍只在所有实际契约及包内 worker 身份通过后导出；两架构仍使用 GitHub 原生 runner，不部署、不推 registry、不用真实数据。

回滚只撤销新增测试夹具与门禁的第 16 个 testcase／配套报告测试，同时保留此轮失败证据；产品源码和数据无迁移。回滚不能冒充早期跳转已验收，也不得 reset／覆盖用户未提交工作或原始 JAR。
