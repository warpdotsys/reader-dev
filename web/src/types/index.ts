/** 后端统一返回结构（兼容 legacy ReturnData：isSuccess/errorMsg/data） */
export interface ReturnData<T = unknown> {
  isSuccess: boolean
  errorMsg: string
  data: T
}

/** 登录/注册返回（formatUser，camelCase） */
export interface UserInfo {
  username: string
  lastLoginAt: number
  accessToken: string
  /** 管理员（secure 模式可操作系统 default 配置与用户管理） */
  isAdmin?: boolean
  [key: string]: unknown
}

/** 书架书籍（books 表 ↔ /reader3/getBookshelf 输出，全字段 camelCase） */
export interface Book {
  bookUrl: string
  tocUrl: string
  origin: string
  originName: string
  name: string
  author: string
  kind?: string | null
  customTag?: string | null
  coverUrl?: string | null
  customCoverUrl?: string | null
  intro?: string | null
  customIntro?: string | null
  charset?: string | null
  type: number
  group: number
  /** 多分组 ID 列表（/reader3/getBookshelf 输出 groupIds；空/缺省 = 未分组） */
  groupIds?: number[]
  latestChapterTitle?: string | null
  latestChapterTime: number
  lastCheckTime?: number
  /** 阅读进度（服务端同步，/reader3/saveBookProgress 写入） */
  durChapterTitle?: string | null
  durChapterIndex?: number
  durChapterPos?: number
  durChapterTime?: number
  /** 总章数（后端 books.totalChapterNum；书卡进度角标用，缺省/为 0 时隐藏角标） */
  totalChapterNum?: number
  /** 追更开关（后端 books.can_update；F-35 定时更新任务按此刷新书架书） */
  canUpdate?: boolean
  [key: string]: unknown
}

/** 书签（/reader3/getBookmarks → Bookmark，camelCase；主键 bookUrl+title） */
export interface Bookmark {
  bookUrl: string
  title: string
  /** 书名（legacy Bookmark.bookName） */
  bookName?: string
  /** 作者（legacy Bookmark.bookAuthor） */
  bookAuthor?: string
  paragraphIndex: number
  chapterIndex: number
  /** 章节名（legacy Bookmark.chapterName） */
  chapterName?: string
  /** 书签段落文本（legacy Bookmark.bookText） */
  bookText?: string
  /** 书签备注（legacy Bookmark.content） */
  content?: string
  createdAt: number
  [key: string]: unknown
}

/** 书籍详情（/reader3/getBookInfo → ruleBookInfo，全字段 camelCase） */
export interface BookInfo {
  name: string
  author: string
  kind?: string | null
  intro?: string | null
  coverUrl?: string | null
  tocUrl: string
  /** 更新时间（legacy BookInfoRule.updateTime） */
  updateTime?: string | null
  wordCount?: string | null
  latestChapterTitle?: string | null
  bookUrl: string
  origin: string
  originName: string
  /** 书籍类型（legacy BookType：0 文本/1 音频/2 漫画/3 文件/4 视频——书源 bookSourceType 透传） */
  type?: number
  [key: string]: unknown
}

/** 章节（/reader3/getBookToc → ruleToc，camelCase；isVolume=卷标题分隔行） */
export interface BookChapter {
  title: string
  url: string
  /** 章节附加信息（legacy BookChapter.tag——目录规则 updateTime） */
  tag?: string | null
  isVolume: boolean
  index: number
  /** 本章字数（后端仅本地书返回；书源书省略 → 前端从已缓存正文估算） */
  chapterWordCount?: number
  [key: string]: unknown
}

/** 章节正文（/reader3/getBookContent → data；文本书 data.content 纯文本；
 *  非文本书按 book_type 返回：音频 {audioUrl, contentType} / 漫画 {images} /
 *  视频 {videoUrl} / 文件 {downloadUrl}） */
export interface BookContent {
  content?: string
  /** 音频书：音频流 URL（m3u8 → contentType=application/vnd.apple.mpegurl） */
  audioUrl?: string
  /** 音频书：媒体类型（按扩展名映射） */
  contentType?: string
  /** 漫画书：章节图片 URL 列表 */
  images?: string[]
  /** 视频书：视频流 URL */
  videoUrl?: string
  /** 文件书：下载链接 */
  downloadUrl?: string
  [key: string]: unknown
}

/** 搜索结果（/reader3/searchBookMulti → SearchBook，全字段 camelCase） */
export interface SearchBook {
  bookUrl: string
  origin: string
  originName: string
  type: number
  name: string
  author: string
  kind?: string | null
  coverUrl?: string | null
  intro?: string | null
  wordCount?: string | null
  latestChapterTitle?: string | null
  updateTime?: string | null
  tocUrl: string
  time?: number
  variable?: string | null
  originOrder?: number
  [key: string]: unknown
}

/** 探索分类（/reader3/getExploreUrls → string[]；视图层派生：url + 从 URL 尾部路径/参数提取的名称） */
export interface ExploreSourceInfo {
  bookSourceUrl: string
  bookSourceName: string
  categoryCount: number
}

export interface ExploreCategory {
  title: string
  url: string
  type?: string
}

/** 书架分组（/reader3/getBookGroups → BookGroup，camelCase；books.group 存分组 id，0=未分组）
 * 契约扩展：orderNum（=legacy order 排序）+ bookCount（组内书数，后端并行实现中——
 * 后端未返回时前端以本地书架统计兜底）。 */
export interface BookGroup {
  id: number
  name: string
  /** 分组封面（legacy BookGroup.cover；空 = 无封面） */
  cover?: string | null
  /** 是否显示该分组（隐藏分组不出现在书架分组栏；后端 book_groups.show） */
  show?: boolean
  /** 排序（后端当前输出 order；契约对齐名 orderNum，二者兼容读取） */
  order?: number
  orderNum?: number
  /** 组内书数（契约字段，后端返回时优先使用，否则本地统计） */
  bookCount?: number
  [key: string]: unknown
}

/** RSS 订阅源（/reader3/getRssSources → RssSource，legacy 兼容 camelCase） */
export interface RssSource {
  sourceUrl: string
  sourceName: string
  sourceGroup?: string | null
  sortUrl?: string | null
  sourceIcon?: string | null
  ruleArticles?: string | null
  ruleTitle?: string | null
  ruleContent?: string | null
  enableJs?: boolean
  enabled: boolean
  [key: string]: unknown
}

/** RSS 文章（/reader3/getRssArticles → data 数组；content 为正文 HTML，getRssArticle 单独拉取） */
export interface RssArticle {
  url: string
  title: string
  author?: string | null
  time: number
  content?: string | null
  cover?: string | null
  /** 已读标记（getRssArticles 返回 hasRead；点击文章后置 true） */
  hasRead?: boolean
  [key: string]: unknown
}

/** 文件管理（/reader3/file/list → FileItem，camelCase；isDirectory=目录） */
export interface FileItem {
  name: string
  size: number
  path: string
  lastModified: number | string
  isDirectory: boolean
  [key: string]: unknown
}

/** 替换规则（当前 localStorage: reader_replace_rules；后端就绪后 ↔ POST /reader3/saveReplaceRule 等，见 api/replaceRules.ts 契约注释） */
export interface ReplaceRule {
  id: string
  name: string
  find: string
  replace: string
  enabled: boolean
  order: number
  [key: string]: unknown
}

/** HttpTTS 听书源（当前 localStorage: reader_http_tts_list；后端就绪后 ↔ POST /reader3/saveHttpTTS 等，见 api/httpTts.ts 契约注释；type 0=在线合成 / 1=本地引擎预留） */
export interface HttpTts {
  id: string
  name: string
  url: string
  type: number
  contentType?: string
  concurrentRate?: string
  loginUrl?: string
  loginUi?: string
  header?: string
  jsLib?: string
  enabledCookieJar?: boolean
  loginCheckJs?: string
  lastUpdateTime?: number
  [key: string]: unknown
}

/** TXT 目录规则（/reader3/getTxtTocRules → TxtTocRule，对齐 legado TxtTocRule：id/name/rule/enable/serialNumber） */
export interface TxtTocRule {
  id: string
  name: string
  rule: string
  enable: boolean
  serialNumber: number
  [key: string]: unknown
}

/** 用户管理（GET /reader3/getUsers → ReaderUser；secure 模式需 secure+secureKey query，缺/错返回 NEED_SECURE_KEY） */
export interface ReaderUser {
  username: string
  enableWebdav: boolean
  enableLocalStore: boolean
  enableBookSource: boolean
  enableRssSource: boolean
  bookSourceLimit: number
  bookLimit: number
  isAdmin?: boolean
  lastLoginAt: number
  /** 注册时间（毫秒时间戳；legacy createdAt） */
  createdAt?: number
  [key: string]: unknown
}

/** 用户更新（POST /reader3/updateUser body：username + 各 enable/limit 字段，缺省字段不修改） */
export interface UserUpdatePayload {
  username: string
  enableWebdav?: boolean
  enableLocalStore?: boolean
  enableBookSource?: boolean
  enableRssSource?: boolean
  bookSourceLimit?: number
  bookLimit?: number
  isAdmin?: boolean
}

/** 系统信息（/reader3/getSystemInfo：版本/端口/用户数/书数/书源数） */
export interface SystemInfo {
  version: string
  port: number
  userCount: number
  bookCount: number
  bookSourceCount: number
  freeMemory?: string
  totalMemory?: string
  maxMemory?: string
  [key: string]: unknown
}

/** 服务监控（GET /reader3/getServerStats → data；内存/CPU/请求量/在线/书源成功率） */
export interface ServerStats {
  version: string
  port: number
  timestamp: number
  uptimeSeconds: number
  memory: {
    totalMb: number
    availableMb: number
    usedMb: number
    processMb: number
    /** 已用内存占比 0..=100 */
    percent: number
  }
  cpu: {
    /** 短采样使用率 0..=100 */
    percent: number
    /** 逻辑核心数 */
    cores: number
  }
  requests: {
    total: number
    today: number
    topEndpoints: { path: string; count: number }[]
  }
  online: {
    /** 活跃 token 会话数 */
    sessions: number
  }
  bookSource: {
    total: number
    ok: number
    failed: number
    /** 0..=1；从未检测时为 null */
    successRate: number | null
    checkedAt: number | null
    namespace: string
    note: string
  }
  [key: string]: unknown
}

/** 全书内容搜索命中（GET /reader3/searchBookContent → data；chapterIndex=章节索引 / title=章节标题 / snippet=匹配片段） */
export interface ContentSearchHit {
  chapterIndex: number
  title: string
  snippet: string
  [key: string]: unknown
}

/** 清理缓存类型（POST /reader3/clearCache body.type：toc=目录缓存 / chapters=章节缓存 / all=全部） */
export type CacheClearType = 'toc' | 'chapters' | 'all'

/** 清理缓存结果（POST /reader3/clearCache → data；deletedToc=删除目录缓存数 / deletedChapters=删除章节缓存数） */
export interface CacheClearResult {
  deletedToc: number
  deletedChapters: number
  [key: string]: unknown
}

/** 导入预览章（POST /reader3/importBookPreview → data.chapters[]，兼容字符串或 {title} 对象） */
export interface ImportPreviewChapter {
  title: string
  [key: string]: unknown
}

/** 导入预览（POST /reader3/importBookPreview → data；后端并行实现中——404/未实现时前端直接上传） */
export interface ImportPreview {
  name: string
  author: string
  format: string
  chapterCount: number
  /** 章节列表（预览前 5 章标题由前端截取；兼容后端返回 chapterList 的命名） */
  chapters?: ImportPreviewChapter[] | null
  chapterList?: ImportPreviewChapter[] | null
  [key: string]: unknown
}

/** 缓存统计（GET /reader3/getCacheInfo → data；tocCacheCount=目录缓存数 / tocCacheSize=目录缓存大小 / chapterCount=章节缓存数 / chapterSize=章节缓存大小 / totalSize=总大小(字节)） */
export interface CacheInfo {
  tocCacheCount: number
  tocCacheSize: number
  chapterCount: number
  chapterSize: number
  totalSize: number
  [key: string]: unknown
}

/** 书源订阅（后端 /reader3/getSourceSubs 为主，localStorage: reader_source_subs 降级，见 api/sourceSubs.ts；
 * 禁用后停止自动刷新，订阅记录与已导入书源保留） */
export interface SourceSub {
  url: string
  name: string
  /** 是否启用（默认 true；false 时定时任务跳过自动刷新） */
  enabled?: boolean
  [key: string]: unknown
}

/** 书源（/reader3/getBookSources → BookSource，legado 兼容 camelCase） */
export interface BookSource {
  bookSourceUrl: string
  bookSourceName: string
  bookSourceGroup?: string | null
  bookSourceType: number
  bookUrlPattern?: string | null
  customOrder: number
  enabled: boolean
  enabledExplore: boolean
  enabledCookieJar?: boolean | null
  concurrentRate?: string | null
  header?: string | null
  loginUrl?: string | null
  loginUi?: string | null
  loginCheckJs?: string | null
  loginJs?: string | null
  bookSourceComment?: string | null
  variableComment?: string | null
  lastUpdateTime: number
  respondTime: number
  weight: number
  exploreUrl?: string | null
  searchUrl?: string | null
  [key: string]: unknown
}

/** 书源登录态（/reader3/getBookSourceCookie → CookieRow，camelCase） */
export interface CookieRow {
  sourceUrl: string
  /** Cookie 原文（本人可见，UI 仅展示摘要） */
  cookie: string
  userAgent?: string
  loginHeader?: string
  updatedAt: number
  [key: string]: unknown
}
