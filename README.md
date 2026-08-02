# reader-dev

阅读3服务器版，不需要手机。

本项目是 [hectorqin/reader](https://github.com/hectorqin/reader) 与 [changshengyu/reader](https://github.com/changshengyu/reader) 的持续维护分支，并集成了 reader-pro 的部分增强能力。

## 致谢

本仓库基于以下原始作者的贡献：

- **hectorqin** — [hectorqin/reader](https://github.com/hectorqin/reader)，阅读3服务器版原始作者
- **changshengyu** — [changshengyu/reader](https://github.com/changshengyu/reader)，社区维护者，合并大量社区 PR（#648 #653 #667 #668 #701 等）并持续增强

感谢两位作者的开源贡献。

## 功能

书源管理、书架管理、搜索（含并发多源搜索/SSE）、书海、看书（翻页/滚动/滑动/自动阅读）、换源（含书源搜索）、WebDAV 同步、文字替换过滤、听书（本地/Edge TTS/HttpTTS）、视频书、漫画、音频、本地书导入（TXT/EPUB/UMD/PDF/CBZ）、书籍分组、RSS 订阅、定时更新书架、本地书仓、Kindle 阅读、简繁转换、多用户管理、许可证（License）支持。

## Docker 部署

```bash
docker pull ghcr.io/warpdotsys/reader-dev:v4.0.5
```

```bash
docker run -d \
  --name reader \
  --restart always \
  -p 4396:8080 \
  -v /home/reader/logs:/logs \
  -v /home/reader/storage:/storage \
  -v reader-data:/data \
  -e SPRING_PROFILES_ACTIVE=prod \
  -e READER_APP_SECURE=true \
  -e READER_APP_SECUREKEY=adminpwd \
  -e READER_APP_INVITECODE=registercode \
  -e READER_APP_USERLIMIT=50 \
  -e READER_APP_DEFAULTUSERENABLEWEBDAV=true \
  -e READER_APP_DEFAULTUSERENABLEBOOKSOURCE=true \
  -e READER_APP_DEFAULTUSERENABLELOCALSTORE=true \
  ghcr.io/warpdotsys/reader-dev:v4.0.5
```

或使用 docker-compose：

```yaml
services:
  reader:
    image: ghcr.io/warpdotsys/reader-dev:v4.0.5
    ports:
      - "4396:8080"
    volumes:
      - ./logs:/logs
      - ./storage:/storage
      - reader-data:/data
    environment:
      SPRING_PROFILES_ACTIVE: prod
      READER_APP_SECURE: "true"
      READER_APP_SECUREKEY: adminpwd
      READER_APP_INVITECODE: registercode
    restart: always

volumes:
  reader-data:
```

### 主要环境变量

| 变量 | 说明 | 默认 |
|---|---|---|
| `SPRING_PROFILES_ACTIVE` | 运行 profile | `prod` |
| `READER_APP_SECURE` | 是否开启安全模式 | `true` |
| `READER_APP_SECUREKEY` | 管理模式密码 | - |
| `READER_APP_INVITECODE` | 注册邀请码 | - |
| `READER_APP_USERLIMIT` | 最大用户数 | `50` |
| `READER_APP_USERBOOKLIMIT` | 每用户书籍上限 | `20000` |
| `READER_APP_DEFAULTUSERBOOKSOURCELIMIT` | 每用户书源上限 | `80000` |
| `READER_APP_DEFAULTUSERENABLEBOOKSOURCE` | 默认启用书源 | `true` |
| `READER_APP_DEFAULTUSERENABLERSSSOURCE` | 默认启用 RSS 源 | `true` |
| `READER_APP_DEFAULTUSERENABLEWEBDAV` | 默认启用 WebDAV | `true` |
| `READER_APP_DEFAULTUSERENABLELOCALSTORE` | 默认启用本地书仓 | `true` |
| `READER_APP_CACHECHAPTERCONTENT` | 缓存章节内容 | `true` |
| `READER_APP_REMOTEWEBVIEWAPI` | 远程 WebView 渲染 API | - |
| `JAVA_OPTS` | JVM 参数 | `-Xms256m -Xmx512m` |

## 自行构建

```bash
docker build -f Dockerfile.source -t reader:latest .
```

或通过 GitHub Actions（打 tag 自动构建并发布到 ghcr.io）：

```bash
git tag vX.Y.Z && git push origin vX.Y.Z
```

发布流程会自动构建镜像并打 `vX.Y.Z`、`latest`、`master` 标签，同时发布 GitHub Release（jar）。

## 开发

```bash
# 前端（web/）
cd web && npm ci && npm run serve

# 后端
./gradlew -b cli.gradle run
```

## 版本

- v4.0.5 — 全代码库对齐 reader-pro 审计修复、CI 优化、前端功能补齐

## License

本项目基于原始项目的开源协议衍生，具体请参考各上游仓库的 License。
