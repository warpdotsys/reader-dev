# 剩余工作 TODO

> 状态更新于 2026-08-12。已完成：真实控制器接线、ReturnData JSON 序列化、async handler 驱动、DB JSON 持久化、前端重新构建（v4.0.7-08121902）、19 项 API 冒烟测试全通过。

## 核心功能（书源可用性，高优先级）

- [ ] **JS 规则引擎真实化**：`AnalyzeRule.eval_js` 接线 boa_engine（依赖已装未用）。书源规则的核心，未完成则搜索/详情/目录/正文全部空。
- [ ] **HTML 解析真实化**：scraper 接入 AnalyzeByJSoup（CSS 选择器）。
- [ ] **XPath 解析真实化**：`AnalyzeByXPath.get_string`（scraper/xmltree 实现）。
- [ ] **JSONPath 解析真实化**：`AnalyzeByJSonPath.get_string`（serde_json 实现 `$.a.b[0]` 路径）。
- [ ] **搜索链路验证**：searchBook / searchBookMulti 用真实书源抓取一本书验证全链路（搜索→详情→目录→正文→净化）。

## 已知 stub / 行为缺口（中优先级）

- [ ] `getUserList`：目前返回"不支持的操作"（未实现）。
- [ ] `getUserConfig`：首次无备份时返回"没有备份文件"，应返回默认配置。
- [ ] WebDAV 备份/同步真实化（get_user_webdav_home / save_to_webdav / sync_from_webdav 上传下载）。
- [ ] `JSONTable`/`SQLTable` 接入 `DB::table`（目前两分支都走 `DB::new`，独立文件未接线）。
- [ ] TTS 边缘处理（tts_by_edge / tts_by_api）验证与修复。
- [ ] 书源导入/导出（导入 local 文件、订阅 remoteBookSourceSub）。
- [ ] 定时任务（shelf_update_job / 书架更新）真实调度。

## 工程收尾（中低优先级）

- [ ] 测试覆盖扩展：写操作（保存书源、导入、搜索真实抓取）+ 持久化重启验证。
- [ ] 前端交互实测：浏览器走一遍 书源管理 → 搜索 → 阅读 → 进度保存。
- [ ] `cargo build --release` 验证。
- [ ] README：构建、运行、测试说明。
- [ ] simple-web（手机版 UI）路由验证。
- [ ] 清理 `web_build*`/`tests/map_to_test.rs` 等临时文件归属。
