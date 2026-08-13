# 项目状态（v6.0.0 已发布）

> 更新于 2026-08-13。v6.0.0 已打 tag 并推送（release workflow 云端构建中：docker 多平台 + 6 平台二进制）。

## 已完成

- [x] 核心链路端到端 37 项测试全通过（tests/front_flow.ps1）
  - 登录 / 书源（保存/列表/simple/发现页/禁用/批量/远程导入/删除）/ 搜索（单书源/多书源并发）
  - 详情 / 目录 / 正文 / 书架（加入/读取/刷新/进度/分组/单书查询/移除）
  - 书签 / 替换规则 / RSS（含文章列表 XML 解析）/ 阅读配置
  - WebDAV（备份/上传/下载/删除）/ 封面下载 / 远程书源导入
- [x] 占位真实化：
  - GSON 反序列化（Book/SearchBook/BookChapter/BookSource/规则结构体）
  - JsonArray/JsonObject/Any 序列化（双重转义、Map/实体 downcast）
  - SimpleDateFormat/Calendar（chrono 日期格式化与解析）
  - XmlDocument（quick-xml DOM 树 → RSS/OPDS xml2map）
  - get_absolute_url（相对 URL 基于 base resolve + 调用方传 base）
  - okhttp 重定向后真实 URL、network_response
  - WebDAV 路由（段内通配）、MKCOL/PROPFIND 原始方法、send_file 文件读取
  - launch 闭包执行（封面下载/书籍缓存/TTS 触发）
  - blocking HTTP async 上下文 panic → 独立线程 async GET
  - 中文切片 panic、RefCell 双重借用 panic
- [x] UI：v5.2.4 设计风格主题（indigo 主色/8px 圆角/深浅色）重新构建并同步
- [x] 版本 6.0.0 同步（Cargo/前端/显示）
- [x] --ui 启动模式（应用版自动打开浏览器）
- [x] 发布配置：Dockerfile 多阶段、release workflow（docker amd64/arm64/386 +
      GitHub Release linux/windows × x64/arm64/x86）、build.sh、docker-compose、README

## 已知限制（不影响主链路）

- AES/RSA 加密（EncoderUtils）为占位（secure 模式 WebDAV Basic 认证依赖）
- TTS 边缘合成依赖外部服务
- EPUB/PDF 本地解析为占位（epublib 域模型未完整转录）
- XPath 规则部分降级为 CSS 选择器
- cache_chapter_content 默认关闭（与原版一致）
