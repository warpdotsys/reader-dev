//! 本地书双轨同步仓（GAP 170/173——重点任务）
//!
//! 设计（用户确认）：文件与 DB 双保留——
//! - 正常读写全走 DB（快）：书架/目录/正文均读 books + book_chapters 表；
//! - 文件变更自动同步进 DB：书仓目录（storage/data/{ns}/books/ + 可选环境变量
//!   `READER_LOCAL_BOOK_DIR`）由 notify 后台监听（300ms 去抖批量），事件后对账；
//! - 仅 DB 书（local:// 且无文件关联）自动生成 epub 落书仓目录（全量元数据，GAP 173）；
//! - 仅文件书（书仓目录内无 local_file 关联的文件）自动导入 DB。
//!
//! 对账任务（启动时 + 文件事件后，幂等，每命名空间互斥锁）：
//! ① 新文件（无 local_file 记录）→ 自动导入（parse_file_bytes → book_chapters + 元数据，
//!    book_url = local://{新 uuid}，local_file 记路径）
//! ② 文件修改（mtime/大小变化）→ 重扫（重新解析，replace_chapters 替换章节 + 元数据 patch）
//! ③ 文件删除 → 书籍保留（is_in_shelf/阅读进度/书签/章节不丢），local_file 保留 +
//!    local_file_deleted=1（文件重现时自动重链重扫——避免重复导入产生副本；
//!    若直接移除书籍或清空 local_file，重现的文件会被当成新书导入 → 副本）
//! ④ DB 书（local:// 且无 local_file）→ 自动生成 epub 落书仓目录（文件名 {书名}.epub，
//!    冲突加后缀 (1)(2)…；生成后立即记录 mtime/大小 → 后续对账幂等跳过，无事件循环）

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::Mutex;

use crate::service::export_book::{build_epub_full, EpubMeta, ExportChapter};
use crate::service::local_book::{self, ImportedBook};
use crate::storage::Storage;

/// 事件去抖窗口（300ms 批量——连续写入合并为一次对账）
const DEBOUNCE_MS: Duration = Duration::from_millis(300);
/// 书仓目录名（storage/data/{ns}/books/）
const BOOKS_DIR: &str = "books";
/// 书仓支持的文件格式（与 SUPPORTED_EXTENSIONS 一致但排除 zip——zip 语义歧义
/// （EPUB 容器 / 裸 OPF 结构），书仓对账只认确定格式）
const SYNC_EXTENSIONS: &[&str] = &[
    "epub", "txt", "mobi", "azw3", "pdf", "fb2", "docx", "cbz", "umd",
];

// ---------------------------------------------------------------------------
// 启动入口
// ---------------------------------------------------------------------------

/// 启动本地书双轨同步（lib.rs serve() 时调用一次）：
/// 1. notify 文件监听（storage/data 递归 + env READER_LOCAL_BOOK_DIR 可选）
/// 2. 初始对账（启动时）
/// 3. 事件去抖循环（300ms 批量 → 受影响命名空间对账）
pub fn spawn_local_sync(storage: Storage) {
    tokio::spawn(async move {
        let data_dir = storage.config.storage_dir().join("data");
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            tracing::warn!("本地书同步：data 目录创建失败: {e}");
        }
        // ① 文件监听（先于初始对账——避免初始对账期间漏事件）
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        spawn_watcher(&storage, event_tx);
        // ② 初始对账（启动时；幂等）
        if let Err(e) = run_reconcile_all(&storage).await {
            tracing::error!("本地书初始对账失败: {e:#}");
        }
        // ③ 事件去抖循环：每个命名空间独立计时，300ms 无新事件 → 对账一次
        let mut pending: HashMap<String, Instant> = HashMap::new();
        loop {
            let wait = pending
                .values()
                .map(|t| *t + DEBOUNCE_MS - Instant::now())
                .min()
                .unwrap_or(DEBOUNCE_MS);
            tokio::select! {
                _ = tokio::time::sleep(wait.max(Duration::ZERO)) => {
                    let nss: Vec<String> = pending.keys().cloned().collect();
                    pending.clear();
                    for ns in nss {
                        let storage = storage.clone();
                        tokio::spawn(async move {
                            let lock = namespace_sync_lock(&ns);
                            let _guard = lock.lock().await;
                            if let Err(e) = reconcile_namespace(&storage, &ns).await {
                                tracing::warn!("本地书对账失败 [{ns}]: {e:#}");
                            }
                        });
                    }
                }
                Some(ns) = event_rx.recv() => {
                    pending.insert(ns, Instant::now());
                }
                else => break,
            }
        }
    });
}

/// notify 文件监听：storage/data 递归 + env READER_LOCAL_BOOK_DIR（可选，映射
/// default 命名空间——secure 多用户模式下该环境目录统一归 default，需注意）。
/// 事件路径 → 命名空间 → mpsc 上报（去抖循环消费）。
/// notify watcher 全局保活：RecommendedWatcher Drop 即停止投递事件——
/// 部署测试发现 spawn_watcher 的局部 watcher 在函数返回时被 Drop（双轨仓监听运行时失效）。
/// 存 static 保活（进程生命周期），事件泵线程独立于 watcher。
static KEEPALIVE_WATCHERS: std::sync::Mutex<Vec<notify::RecommendedWatcher>> =
    std::sync::Mutex::new(Vec::new());

fn spawn_watcher(storage: &Storage, event_tx: tokio::sync::mpsc::UnboundedSender<String>) {
    use notify::Watcher as _;
    let data_dir = storage.config.storage_dir().join("data");
    let env_dir = std::env::var("READER_LOCAL_BOOK_DIR")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);
    let mut roots: Vec<PathBuf> = vec![data_dir.clone()];
    if let Some(d) = &env_dir {
        if !roots.iter().any(|r| r == d) {
            roots.push(d.clone());
        }
    }
    for root in roots {
        let tx = event_tx.clone();
        let (std_tx, std_rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher =
            match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                let _ = std_tx.send(res);
            }) {
                Ok(w) => w,
                Err(e) => {
                    tracing::warn!("本地书监听创建失败（{}）: {e}", root.display());
                    continue;
                }
            };
        if let Err(e) = watcher.watch(&root, notify::RecursiveMode::Recursive) {
            tracing::warn!("本地书监听目录失败（{}）: {e}", root.display());
            continue;
        }
        // 保活：watcher 移入 static（函数返回不 Drop——否则 notify 停止投递，监听失效）
        if let Ok(mut keep) = KEEPALIVE_WATCHERS.lock() {
            keep.push(watcher);
        } else {
            continue;
        }
        let root2 = root.clone();
        let data2 = data_dir.clone();
        // 事件泵线程：std 通道（notify 回调）→ ns 映射 → tokio 通道
        std::thread::spawn(move || {
            for res in std_rx {
                let Ok(ev) = res else { continue };
                // 只关心 创建/修改/删除/重命名；纯访问（Access）忽略
                if matches!(ev.kind, notify::EventKind::Access(_)) {
                    continue;
                }
                let mut nss = HashSet::new();
                for p in ev.paths {
                    if let Some(ns) = namespace_of_path(&p, &data2, &root2) {
                        nss.insert(ns);
                    }
                }
                for ns in nss {
                    let _ = tx.send(ns);
                }
            }
        });
        tracing::info!("本地书监听已启动: {}", root.display());
    }
}

/// 事件路径 → 命名空间：
/// - `storage/data/{ns}/books/...`（路径含 books 组件）→ ns
/// - env READER_LOCAL_BOOK_DIR 根下（不在 data 子树内）→ "default"
/// - 其他（opds_files/webdav/assets 等）→ None（不触发对账）
fn namespace_of_path(p: &Path, data_dir: &Path, root: &Path) -> Option<String> {
    if p.starts_with(root) && !p.starts_with(data_dir) {
        return Some("default".to_string());
    }
    let rel = p.strip_prefix(data_dir).ok()?;
    let mut comps = rel.components();
    let ns = comps.next()?.as_os_str().to_string_lossy().into_owned();
    let rest: Vec<String> = comps
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if rest.iter().any(|c| c == BOOKS_DIR) {
        Some(ns)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// 对账任务
// ---------------------------------------------------------------------------

/// 每命名空间对账互斥锁（避免对账与手动 refreshLocalBook 并发冲突——重扫/重链
/// 读改写序列必须串行；对账本身幂等，锁只是防止交错）
pub fn namespace_sync_lock(ns: &str) -> Arc<Mutex<()>> {
    static LOCKS: once_cell::sync::Lazy<std::sync::Mutex<HashMap<String, Arc<Mutex<()>>>>> =
        once_cell::sync::Lazy::new(|| std::sync::Mutex::new(HashMap::new()));
    LOCKS
        .lock()
        .unwrap()
        .entry(ns.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

/// 对账全部命名空间（启动时）。返回有变更的命名空间数。
pub async fn run_reconcile_all(storage: &Storage) -> Result<usize> {
    let nss = storage.schedule_namespaces().await;
    let mut changed = 0usize;
    for ns in nss {
        let lock = namespace_sync_lock(&ns);
        let _guard = lock.lock().await;
        if reconcile_namespace(storage, &ns).await? {
            changed += 1;
        }
    }
    Ok(changed)
}

/// 单命名空间对账（幂等；文件事件后 / 启动时调用）。返回是否有任何变更。
/// 书仓目录 = storage/data/{ns}/books/ + env READER_LOCAL_BOOK_DIR（可选，仅 default）。
pub async fn reconcile_namespace(storage: &Storage, ns: &str) -> Result<bool> {
    let env_dirs: Vec<PathBuf> = std::env::var("READER_LOCAL_BOOK_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .into_iter()
        .collect();
    reconcile_namespace_dirs(storage, ns, &env_dirs).await
}

/// 对账核心（env_dirs 为额外书仓目录；测试直接传入临时目录，避免进程级 env 竞态）
async fn reconcile_namespace_dirs(
    storage: &Storage,
    ns: &str,
    env_dirs: &[PathBuf],
) -> Result<bool> {
    let books_dir = storage
        .config
        .storage_dir()
        .join("data")
        .join(ns)
        .join(BOOKS_DIR);
    std::fs::create_dir_all(&books_dir)?;
    // 环境变量书仓目录（可选；仅 default 命名空间对账——secure 多用户下 env 目录归属 default）
    let env_books: Vec<PathBuf> = if ns == "default" {
        env_dirs.to_vec()
    } else {
        Vec::new()
    };

    let mut changed = false;

    // ---------- ① 磁盘文件清单（books_dir 递归 + env 目录递归） ----------
    let mut files: Vec<PathBuf> = Vec::new();
    collect_book_files(&books_dir, &mut files).await;
    for d in &env_books {
        collect_book_files(d, &mut files).await;
    }

    // ---------- ② DB 关联清单（含已删除标记的——文件重现时重链） ----------
    let linked = storage.list_linked_local_books(ns).await?;
    let mut linked_by_path: HashMap<String, crate::model::Book> = HashMap::new();
    for b in &linked {
        if let Some(p) = &b.local_file {
            linked_by_path.insert(normalize_path(p), b.clone());
        }
    }

    // ---------- ③ 逐文件对账（新文件导入 / 修改重扫 / 重现重链） ----------
    // TXT 用户自定义目录规则（启用+排序；无则空 → parse 回退内置默认规则）
    let user_rules: Vec<String> = storage
        .get_txt_toc_rules(ns)
        .await
        .ok()
        .map(|rules| {
            rules
                .into_iter()
                .filter(|r| r.enable)
                .map(|r| r.rule)
                .collect()
        })
        .unwrap_or_default();
    for path in &files {
        let key = normalize_path(&path.to_string_lossy());
        crate::service::fs_rate::tick().await;
        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue, // 事件竞态：文件已被删除
        };
        let mtime = file_mtime_ms(&meta);
        let size = meta.len() as i64;
        match linked_by_path.get(&key).cloned() {
            Some(book) => {
                // 已关联：文件修改（mtime/大小变化）或此前标记删除（文件重现）→ 重扫
                if book.local_file_deleted
                    || book.local_file_mtime != mtime
                    || book.local_file_size != size
                {
                    match reparse_and_update(storage, ns, &book, path, mtime, size).await {
                        Ok(()) => changed = true,
                        Err(e) => {
                            tracing::warn!("本地书重扫失败 [{}] {}: {e:#}", ns, path.display())
                        }
                    }
                }
            }
            None => {
                // 新文件（无 local_file 记录）→ 自动导入
                match import_file(storage, ns, path, &user_rules, mtime, size).await {
                    Ok(()) => changed = true,
                    Err(e) => {
                        tracing::warn!("本地书导入失败 [{}] {}: {e:#}", ns, path.display())
                    }
                }
            }
        }
    }

    // ---------- ④ 已关联但文件消失 → 标记删除（保留书籍/进度/章节） ----------
    let disk_keys: HashSet<String> = files
        .iter()
        .map(|p| normalize_path(&p.to_string_lossy()))
        .collect();
    for book in &linked {
        let Some(p) = &book.local_file else { continue };
        let key = normalize_path(p);
        if !disk_keys.contains(&key) && !book.local_file_deleted {
            // 方案说明：不直接移除书籍、不清空 local_file——is_in_shelf/阅读进度/
            // 书签/章节全部保留（DB 仍可读）；置 local_file_deleted=1 后文件重现
            // 时自动重链重扫（③），不会把重现文件当新书重复导入。
            storage
                .link_local_file(
                    ns,
                    &book.book_url,
                    Some(p),
                    book.local_file_mtime,
                    book.local_file_size,
                    true,
                )
                .await?;
            tracing::info!(
                "本地书文件已删除，书籍保留 [{}] {}（local_file_deleted=1）",
                ns,
                book.name
            );
            changed = true;
        }
    }

    // ---------- ⑤ local:// 且无文件关联的 DB 书 → 自动生成 epub 落书仓 ----------
    let db_only = storage.list_local_db_books_without_file(ns).await?;
    for book in db_only {
        crate::service::fs_rate::tick().await;
        match generate_epub_for_book(storage, ns, &book, &books_dir).await {
            Ok(()) => changed = true,
            Err(e) => tracing::warn!("本地书 epub 生成失败 [{}] {}: {e:#}", ns, book.name),
        }
    }

    Ok(changed)
}

// ---------------------------------------------------------------------------
// ① 新文件导入 / ② 修改重扫
// ---------------------------------------------------------------------------

/// 新文件自动导入：解析 → save_local_book（local:// 新 uuid + 章节 + 元数据）→
/// 封面落盘（与上传导入一致）→ 文件关联（双轨）
async fn import_file(
    storage: &Storage,
    ns: &str,
    path: &Path,
    user_rules: &[String],
    mtime: i64,
    size: i64,
) -> Result<()> {
    let ext = local_book::file_ext(&path.to_string_lossy());
    if !SYNC_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(()); // 非书仓格式（zip 等）跳过
    }
    crate::service::fs_rate::tick().await;
    let bytes = std::fs::read(path)?;
    let imported = local_book::parse_file_bytes(
        &bytes,
        &ext,
        user_rules,
        local_book::DEFAULT_EPUB_TOC_MODE,
        false,
    )
    .map_err(|e| anyhow::anyhow!("解析失败: {e:#}"))?;
    if imported.chapters.is_empty() {
        anyhow::bail!("未解析到章节内容");
    }
    let book_url = format!("local://{}", uuid::Uuid::new_v4());
    let name = if imported.meta.title.is_empty() {
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "本地书籍".to_string())
    } else {
        imported.meta.title.clone()
    };
    let info = crate::model::book_chapter::BookInfo {
        name,
        author: imported.meta.author.clone(),
        kind: imported.meta.subjects.first().cloned(),
        intro: imported.meta.description.clone(),
        language: imported.meta.language.clone(),
        publisher: imported.meta.publisher.clone(),
        published_at: imported.meta.published_at.clone(),
        toc_url: Some(format!("{book_url}/toc")),
        book_url: book_url.clone(),
        origin: "local".to_string(),
        origin_name: "本地书".to_string(),
        ..Default::default()
    };
    storage.save_local_book(ns, &info, &imported).await?;
    // 封面落盘（与上传导入一致）
    if let Some(cover) = &imported.cover {
        let cover_dir = storage
            .config
            .storage_dir()
            .join("assets")
            .join(ns)
            .join("covers");
        let _ = std::fs::create_dir_all(&cover_dir);
        let file_id = format!("{}.jpg", uuid::Uuid::new_v4());
        if std::fs::write(cover_dir.join(&file_id), cover).is_ok() {
            let _ = storage
                .update_book_cover(ns, &book_url, &format!("/assets/{ns}/covers/{file_id}"))
                .await;
        }
    }
    // 文件关联（双轨）
    storage
        .link_local_file(
            ns,
            &book_url,
            Some(&normalize_path(&path.to_string_lossy())),
            mtime,
            size,
            false,
        )
        .await?;
    tracing::info!(
        "本地书自动导入 [{}] {}（{} 章）← {}",
        ns,
        info.name,
        imported.chapters.len(),
        path.display()
    );
    Ok(())
}

/// 文件修改/重现 → 重新解析，替换章节与元数据（保留阅读进度/书签/分组/
/// custom_intro/custom_tag 等用户编辑字段；仅刷新文件可表达的元数据）
async fn reparse_and_update(
    storage: &Storage,
    ns: &str,
    book: &crate::model::Book,
    path: &Path,
    mtime: i64,
    size: i64,
) -> Result<()> {
    let ext = local_book::file_ext(&path.to_string_lossy());
    crate::service::fs_rate::tick().await;
    let bytes = std::fs::read(path)?;
    let imported: ImportedBook =
        local_book::parse_file_bytes(&bytes, &ext, &[], &book.toc_url, book.split_long_chapter)
            .map_err(|e| anyhow::anyhow!("解析失败: {e:#}"))?;
    if imported.chapters.is_empty() {
        anyhow::bail!("未解析到章节内容");
    }
    // 替换章节（先删后插单事务——章数减少时无旧章残留）
    let pairs: Vec<(String, String)> = imported
        .chapters
        .iter()
        .map(|c| (c.title.clone(), c.content.clone()))
        .collect();
    storage.replace_chapters(ns, &book.book_url, &pairs).await?;
    // 元数据 patch（仅文件可表达的字段；用户编辑字段 custom_intro/custom_tag 不动）
    let mut patch = serde_json::Map::new();
    patch.insert(
        "totalChapterNum".to_string(),
        serde_json::json!(pairs.len() as i64),
    );
    if book.name.is_empty() && !imported.meta.title.is_empty() {
        patch.insert("name".to_string(), serde_json::json!(imported.meta.title));
    }
    if !imported.meta.author.is_empty() {
        patch.insert(
            "author".to_string(),
            serde_json::json!(imported.meta.author),
        );
    }
    if let Some(d) = &imported.meta.description {
        if !d.is_empty() {
            patch.insert("intro".to_string(), serde_json::json!(d));
        }
    }
    if let Some(l) = &imported.meta.language {
        patch.insert("language".to_string(), serde_json::json!(l));
    }
    if let Some(p) = &imported.meta.publisher {
        patch.insert("publisher".to_string(), serde_json::json!(p));
    }
    if let Some(d) = &imported.meta.published_at {
        patch.insert("publishedAt".to_string(), serde_json::json!(d));
    }
    if let Some(k) = imported.meta.subjects.first() {
        patch.insert("kind".to_string(), serde_json::json!(k));
    }
    let _ = storage.patch_book(ns, &book.book_url, &patch).await;
    // 封面更新（新封面存在时替换）
    if let Some(cover) = &imported.cover {
        let cover_dir = storage
            .config
            .storage_dir()
            .join("assets")
            .join(ns)
            .join("covers");
        let _ = std::fs::create_dir_all(&cover_dir);
        let file_id = format!("{}.jpg", uuid::Uuid::new_v4());
        if std::fs::write(cover_dir.join(&file_id), cover).is_ok() {
            let _ = storage
                .update_book_cover(
                    ns,
                    &book.book_url,
                    &format!("/assets/{ns}/covers/{file_id}"),
                )
                .await;
        }
    }
    // 更新关联（mtime/大小 + 清除删除标记）
    storage
        .link_local_file(
            ns,
            &book.book_url,
            Some(&normalize_path(&path.to_string_lossy())),
            mtime,
            size,
            false,
        )
        .await?;
    tracing::info!(
        "本地书重扫 [{}] {}（{} 章）← {}",
        ns,
        book.name,
        pairs.len(),
        path.display()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// ④ DB 书 → epub 落书仓（GAP 173：全量元数据）
// ---------------------------------------------------------------------------

/// DB 书（local:// 无文件关联）自动生成 epub 落书仓目录。
/// 文件名 {书名}.epub，冲突加后缀 (1)(2)…；生成后立即记录 mtime/大小，
/// 后续对账幂等跳过（不会把生成文件当新书导入，也无事件循环）。
async fn generate_epub_for_book(
    storage: &Storage,
    ns: &str,
    book: &crate::model::Book,
    books_dir: &Path,
) -> Result<()> {
    let rows = storage.list_chapters_full(&book.book_url).await?;
    if rows.is_empty() {
        return Ok(()); // 空书不生成
    }
    let chapters: Vec<ExportChapter> = rows
        .iter()
        .map(|(_, title, content)| ExportChapter {
            title: title.clone(),
            content: content.clone(),
        })
        .collect();
    // GAP 173 全量元数据：description = custom_intro 优先（其次 intro）；
    // subject = custom_tag 优先（其次 kind）；语言/日期/出版社直接映射；封面取本地 cover_url
    let meta = EpubMeta {
        description: book.custom_intro.clone().or_else(|| book.intro.clone()),
        language: book.language.clone(),
        published_at: book.published_at.clone(),
        publisher: book.publisher.clone(),
        subject: book.custom_tag.clone().or_else(|| book.kind.clone()),
        cover: read_local_cover(storage, ns, book.cover_url.as_deref()),
        // GAP 176：双轨同步自动生成的 epub 不内嵌字体（保持体积最小；导出 API 可指定）
        font: crate::service::export_book::EmbedFont::None,
    };
    let bytes = build_epub_full(&book.name, &book.author, &meta, &chapters);
    // 文件名：{书名}.epub（冲突加后缀）
    let mut fname = sanitize_filename(&book.name);
    if fname.is_empty() {
        fname = "本地书".to_string();
    }
    let mut target = books_dir.join(format!("{fname}.epub"));
    let mut i = 1usize;
    while target.exists() {
        target = books_dir.join(format!("{fname} ({i}).epub"));
        i += 1;
    }
    crate::service::fs_rate::tick().await;
    std::fs::write(&target, &bytes)?;
    crate::service::fs_rate::tick().await;
    let meta = std::fs::metadata(&target)?;
    storage
        .link_local_file(
            ns,
            &book.book_url,
            Some(&normalize_path(&target.to_string_lossy())),
            file_mtime_ms(&meta),
            meta.len() as i64,
            false,
        )
        .await?;
    tracing::info!(
        "本地书 epub 已生成 [{}] {} → {}",
        ns,
        book.name,
        target.display()
    );
    Ok(())
}

/// 读取本地封面字节（cover_url 为 /assets/{ns}/covers/{file} 形态；远程 URL 跳过）
fn read_local_cover(storage: &Storage, ns: &str, cover_url: Option<&str>) -> Option<Vec<u8>> {
    let url = cover_url?;
    let rest = url.strip_prefix("/assets/")?;
    if !rest.starts_with(&format!("{ns}/covers/")) {
        return None;
    }
    let path = storage.config.storage_dir().join("assets").join(rest);
    std::fs::read(path).ok()
}

/// 文件名净化（去路径分隔符/非法字符；保留中文与扩展名语义）
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if matches!(
                c,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\n' | '\r' | '\t'
            ) {
                ' '
            } else {
                c
            }
        })
        .collect();
    cleaned.trim().trim_matches('.').trim().to_string()
}

// ---------------------------------------------------------------------------
// 工具
// ---------------------------------------------------------------------------

/// 递归收集书仓格式文件（epub/txt/mobi/azw3/pdf/fb2/docx；zip 不入仓）。
/// 每次 readdir/stat 前经过 fs_rate 全局限速（网盘挂载目录风控保护）。
async fn collect_book_files(dir: &Path, out: &mut Vec<PathBuf>) {
    // 显式栈替代 async 递归（递归 async fn 需 Box::pin，迭代更直接）
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        crate::service::fs_rate::tick().await;
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
        for p in paths.into_iter().rev() {
            crate::service::fs_rate::tick().await;
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                let ext = local_book::file_ext(&p.to_string_lossy());
                if SYNC_EXTENSIONS.contains(&ext.as_str()) {
                    out.push(p);
                }
            }
        }
    }
}

/// 路径规范化（分隔符统一为 '/'——DB 存储与比对键一致，跨平台可移植）
fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// 文件修改时间（ms epoch；失败回退 0——与大小共同判定变更）
fn file_mtime_ms(m: &std::fs::Metadata) -> i64 {
    m.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Book;

    /// 独立临时目录存储（避免污染真实 storage/reader.db）
    async fn test_storage(tag: &str) -> Storage {
        let dir = std::env::temp_dir().join(format!(
            "reader-localsync-test-{}-{tag}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let mut config = crate::AppConfig::from_env();
        config.work_dir = dir.to_string_lossy().into_owned();
        crate::storage::init(&config).await.unwrap()
    }

    async fn cleanup(storage: Storage, tag: &str) {
        storage.pool.close().await;
        let dir = std::env::temp_dir().join(format!(
            "reader-localsync-test-{}-{tag}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 隔离 env READER_LOCAL_BOOK_DIR：多数测试走 reconcile_namespace_dirs（显式目录），
    /// 不读进程 env——唯一读 env 的测试单独 set_var（无其他测试再写该变量，无竞态）
    fn no_env_dirs() -> Vec<PathBuf> {
        Vec::new()
    }

    fn books_dir(storage: &Storage, ns: &str) -> PathBuf {
        storage
            .config
            .storage_dir()
            .join("data")
            .join(ns)
            .join(BOOKS_DIR)
    }

    fn write_txt(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    const SAMPLE1: &str = "第一章 起点\n内容一。\n第二章 成长\n内容二。";
    const SAMPLE2: &str =
        "第一章 起点\n内容一（修订）。\n第二章 成长\n内容二（修订）。\n第三章 终章\n内容三。";

    /// ① 新文件 → 自动导入 DB（local:// + local_file 关联 + 章节/元数据）
    #[tokio::test]
    async fn reconcile_imports_new_file() {
        let storage = test_storage("import").await;
        write_txt(&books_dir(&storage, "default").join("测试书.txt"), SAMPLE1);
        let changed = reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        assert!(changed);
        // 书架新增 1 本 local:// 书
        let books = storage.list_books("default").await.unwrap();
        assert_eq!(books.len(), 1);
        let b = &books[0];
        assert!(b.book_url.starts_with("local://"));
        assert_eq!(b.name, "第一章 起点", "TXT 首行应为书名");
        assert_eq!(b.origin, "local");
        // 文件关联
        assert!(b
            .local_file
            .as_deref()
            .unwrap()
            .ends_with("books/测试书.txt"));
        assert!(!b.local_file_deleted);
        assert!(b.local_file_mtime > 0);
        // 章节入库
        assert_eq!(
            storage
                .count_chapters("default", &b.book_url)
                .await
                .unwrap(),
            2
        );
        let toc = storage.list_chapters(&b.book_url).await.unwrap();
        assert_eq!(toc[0].1, "第一章 起点");
        assert_eq!(toc[1].1, "第二章 成长");
        // 幂等：再次对账无变更
        let changed2 = reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        assert!(!changed2);
        assert_eq!(storage.list_books("default").await.unwrap().len(), 1);
        cleanup(storage, "import").await;
    }

    /// ② 文件修改（mtime/大小变化）→ 重扫：章节替换（章数变化无残留）、
    /// 同一 book_url 不重复导入
    #[tokio::test]
    async fn reconcile_rescans_modified_file() {
        let storage = test_storage("rescan").await;
        let path = books_dir(&storage, "default").join("测试书.txt");
        write_txt(&path, SAMPLE1);
        reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        let book_url = storage.list_books("default").await.unwrap()[0]
            .book_url
            .clone();
        // 修改文件（内容 + 大小变化；sleep 保证 mtime 推进——Windows NTFS 高精度，
        // 但跨平台稳妥起见内容长度也变化，mtime/大小任一不同即触发）
        std::thread::sleep(Duration::from_millis(20));
        write_txt(&path, SAMPLE2);
        let changed = reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        assert!(changed);
        let books = storage.list_books("default").await.unwrap();
        assert_eq!(books.len(), 1, "重扫不应重复导入");
        assert_eq!(books[0].book_url, book_url);
        // 章节被替换（3 章，无旧章残留）
        assert_eq!(
            storage.count_chapters("default", &book_url).await.unwrap(),
            3
        );
        let toc = storage.list_chapters(&book_url).await.unwrap();
        assert_eq!(toc[2].1, "第三章 终章");
        let content = storage
            .get_chapter_content("default", &book_url, 1)
            .await
            .unwrap()
            .unwrap();
        assert!(content.contains("内容二（修订）"));
        cleanup(storage, "rescan").await;
    }

    /// ③ 文件删除 → 书籍保留（is_in_shelf/章节不丢）+ deleted 标记；
    /// 文件重现 → 自动重链重扫（不重复导入）
    #[tokio::test]
    async fn reconcile_marks_deleted_and_relinks() {
        let storage = test_storage("delete").await;
        let path = books_dir(&storage, "default").join("测试书.txt");
        write_txt(&path, SAMPLE1);
        reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        let book_url = storage.list_books("default").await.unwrap()[0]
            .book_url
            .clone();
        // 删除文件
        std::fs::remove_file(&path).unwrap();
        let changed = reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        assert!(changed);
        let b = storage
            .find_book("default", &book_url)
            .await
            .unwrap()
            .unwrap();
        assert!(b.local_file_deleted, "应标记删除");
        assert!(b.is_in_shelf, "书籍保留在书架");
        assert_eq!(
            storage.count_chapters("default", &book_url).await.unwrap(),
            2,
            "章节保留可读"
        );
        // 文件重现（内容变化）→ 重链 + 重扫
        std::thread::sleep(Duration::from_millis(20));
        write_txt(&path, SAMPLE2);
        let changed = reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        assert!(changed);
        let books = storage.list_books("default").await.unwrap();
        assert_eq!(books.len(), 1, "重现不重复导入");
        assert_eq!(books[0].book_url, book_url);
        assert!(!books[0].local_file_deleted);
        assert_eq!(
            storage.count_chapters("default", &book_url).await.unwrap(),
            3
        );
        cleanup(storage, "delete").await;
    }

    /// ④ DB 书（local:// 无文件关联）→ 自动生成 epub 落书仓；
    /// GAP 173：全量元数据（title/creator/description(custom_intro 优先)/language/
    /// date/publisher/subject(custom_tag)/封面）——重新 parse 零丢失
    #[tokio::test]
    async fn reconcile_generates_epub_full_metadata() {
        let storage = test_storage("gen").await;
        // DB 书：全字段 + 封面文件
        let book_url = "local://genbook1".to_string();
        let cover_bytes = b"\xFF\xD8\xFF\xE0fake-jpeg-cover".to_vec();
        let cover_rel = "assets/default/covers/genbook1.jpg";
        let cover_path = storage.config.storage_dir().join(cover_rel);
        std::fs::create_dir_all(cover_path.parent().unwrap()).unwrap();
        std::fs::write(&cover_path, &cover_bytes).unwrap();
        storage
            .upsert_book(
                "default",
                &Book {
                    book_url: book_url.clone(),
                    name: "元数据完整书".into(),
                    author: "作者甲".into(),
                    intro: Some("原始简介".into()),
                    custom_intro: Some("自定义简介（优先）".into()),
                    kind: Some("分类甲".into()),
                    custom_tag: Some("自定义标签".into()),
                    language: Some("zh-CN".into()),
                    publisher: Some("出版社乙".into()),
                    published_at: Some("2024-05-06".into()),
                    cover_url: Some(format!("/{cover_rel}")),
                    origin: "local".into(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        storage
            .save_chapters(
                "default",
                &book_url,
                &[
                    ("第一章".to_string(), "正文一。".to_string()),
                    ("第二章".to_string(), "正文二。".to_string()),
                ],
            )
            .await
            .unwrap();

        let changed = reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        assert!(changed);
        // epub 落书仓 + 关联
        let b = storage
            .find_book("default", &book_url)
            .await
            .unwrap()
            .unwrap();
        let epub_path = b.local_file.as_deref().expect("应生成 epub 关联");
        assert!(epub_path.ends_with("元数据完整书.epub"));
        let bytes = std::fs::read(epub_path).unwrap();
        // 重新解析：零丢失断言
        let imported = local_book::parse_epub(&bytes, local_book::DEFAULT_EPUB_TOC_MODE)
            .expect("生成的 epub 可重新解析");
        assert_eq!(imported.meta.title, "元数据完整书");
        assert_eq!(imported.meta.author, "作者甲");
        assert_eq!(
            imported.meta.description.as_deref(),
            Some("自定义简介（优先）"),
            "description 应取 custom_intro"
        );
        assert_eq!(imported.meta.language.as_deref(), Some("zh-CN"));
        assert_eq!(imported.meta.published_at.as_deref(), Some("2024-05-06"));
        assert_eq!(imported.meta.publisher.as_deref(), Some("出版社乙"));
        assert_eq!(
            imported.meta.subjects,
            vec!["自定义标签".to_string()],
            "subject 应取 custom_tag"
        );
        assert_eq!(imported.chapters.len(), 2);
        // 封面零丢失（字节一致）
        assert!(imported.cover.is_some());
        assert_eq!(imported.cover.unwrap(), cover_bytes);
        // 幂等：不再重复生成
        let changed2 = reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        assert!(!changed2);
        cleanup(storage, "gen").await;
    }

    /// 文件名冲突 → 后缀 (1)(2)…；空书名回退"本地书"
    #[tokio::test]
    async fn reconcile_epub_filename_conflict_suffix() {
        let storage = test_storage("conflict").await;
        for i in 0..2 {
            let book_url = format!("local://conflict{i}");
            storage
                .upsert_book(
                    "default",
                    &Book {
                        book_url: book_url.clone(),
                        name: "同名书".into(),
                        origin: "local".into(),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            storage
                .save_chapters("default", &book_url, &[("第一章".to_string(), "正文。".to_string())])
                .await
                .unwrap();
        }
        reconcile_namespace_dirs(&storage, "default", &no_env_dirs())
            .await
            .unwrap();
        let books = storage.list_books("default").await.unwrap();
        let files: Vec<String> = books.iter().filter_map(|b| b.local_file.clone()).collect();
        assert_eq!(files.len(), 2);
        // 两份同名书：首本 {书名}.epub，冲突本 {书名} (1).epub（集合断言——生成顺序与
        // list_books 排序无关，词序上 " (1)" 在 "." 之前）
        let names: Vec<String> = files
            .iter()
            .map(|f| f.rsplit('/').next().unwrap_or(f).to_string())
            .collect();
        assert!(
            names.contains(&"同名书.epub".to_string()),
            "应存在无后缀文件: {names:?}"
        );
        assert!(
            names.contains(&"同名书 (1).epub".to_string()),
            "冲突应加后缀: {names:?}"
        );
        cleanup(storage, "conflict").await;
    }

    /// env READER_LOCAL_BOOK_DIR 目录内的文件也参与对账（default 命名空间）
    #[tokio::test]
    async fn reconcile_env_book_dir_imports() {
        let env_dir = std::env::temp_dir().join(format!(
            "reader-localsync-env-import-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&env_dir);
        std::fs::create_dir_all(&env_dir).unwrap();
        std::env::set_var("READER_LOCAL_BOOK_DIR", &env_dir);
        let storage = test_storage("envimport").await;
        write_txt(&env_dir.join("环境书.txt"), SAMPLE1);
        let changed = reconcile_namespace(&storage, "default").await.unwrap();
        assert!(changed);
        let books = storage.list_books("default").await.unwrap();
        assert_eq!(books.len(), 1);
        assert!(books[0]
            .local_file
            .as_deref()
            .unwrap()
            .contains("环境书.txt"));
        // 非 default 命名空间不扫 env 目录
        let changed2 = reconcile_namespace(&storage, "other").await.unwrap();
        assert!(!changed2);
        cleanup(storage, "envimport").await;
    }

    /// 事件路径 → 命名空间映射
    #[test]
    fn test_namespace_of_path_mapping() {
        let data = PathBuf::from("/srv/storage/data");
        let env = PathBuf::from("/srv/books");
        assert_eq!(
            namespace_of_path(
                Path::new("/srv/storage/data/alice/books/a.txt"),
                &data,
                &data
            )
            .as_deref(),
            Some("alice")
        );
        assert_eq!(
            namespace_of_path(
                Path::new("/srv/storage/data/alice/books/sub/b.epub"),
                &data,
                &data
            )
            .as_deref(),
            Some("alice")
        );
        // 非 books 目录（opds_files/assets）不触发
        assert_eq!(
            namespace_of_path(
                Path::new("/srv/storage/data/alice/opds_files/x.txt"),
                &data,
                &data
            ),
            None
        );
        // env 根 → default
        assert_eq!(
            namespace_of_path(Path::new("/srv/books/a.txt"), &data, &env).as_deref(),
            Some("default")
        );
        // 无关路径 → None
        assert_eq!(
            namespace_of_path(Path::new("/tmp/x.txt"), &data, &data),
            None
        );
    }

    /// 文件名净化
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            sanitize_filename("a/b\\c:d*e?f\"g<h>i|j"),
            "a b c d e f g h i j"
        );
        assert_eq!(sanitize_filename("  书  "), "书");
        assert_eq!(sanitize_filename(".."), "");
    }
}
