//! JSON → SQLite 一次性迁移（legacy storage/data → SQLite）
//!
//! 触发条件：`storage/data/users.json` 存在 且 users 表为空。
//! 迁移前自动备份 `storage/data/` → `storage/backup-before-migrate-{ts}/`。
//!
//! 迁移内容：
//! - `storage/data/users.json`（Map<username, User>）→ users 表（user_namespace = username）
//! - `storage/data/{ns}/bookshelf.json`（ns = default 或各用户名）→ books 表（user_namespace = ns）
//! - bookSource.json / rssSource.json → book_sources / rss_sources 表
//! - bookmark.json / replaceRule.json / txtTocRule.json / httpTTS.json / bookGroup.json / userConfig.json
//!   → bookmarks / replace_rules / txt_toc_rules / http_tts_list / book_groups / user_config 表
//! 每类幂等：目标表非空即跳过（表空才迁）；bookmarks 带 raw_json 原文保底（legacy content 不丢）。

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::model::{Book, BookSource, User};
use crate::storage::Storage;

/// 启动时检测并执行 JSON → SQLite 迁移（幂等：users 表非空即跳过）
pub async fn migrate_if_needed(storage: &Storage) -> Result<()> {
    let data_dir = storage.config.storage_dir().join("data");
    let users_path = data_dir.join("users.json");
    // 历史迁移缺陷补全：早期版本迁移书架时漏写 books.toc_url（raw_json 保留了
    // 原始 tocUrl），导致网络书有正文缓存但拿不到目录。每次启动扫描一次，幂等。
    let backfilled = backfill_toc_url_from_raw(&storage.pool).await?;
    if backfilled > 0 {
        tracing::info!("补全迁移书籍 toc_url：{backfilled} 本（从 raw_json 恢复 tocUrl）");
    }
    // 管理员 default 残留个人数据回迁：管理员默认使用本人账号命名空间，
    // default 仅作系统配置层；若 default 中混入个人数据（书架/进度等），
    // 启动时归位到管理员本人命名空间，幂等。
    let admin_personal_moved = migrate_admin_default_personal_data_back(&storage.pool).await?;
    if admin_personal_moved > 0 {
        tracing::info!("管理员 default 残留个人数据归位本人命名空间：{admin_personal_moved} 行");
    }
    if !users_path.exists() {
        tracing::info!(
            "未发现 legacy JSON 数据（{} 不存在），跳过迁移",
            users_path.display()
        );
        return Ok(());
    }
    use std::io::Write;
    eprintln!(
        "
======================== 检测到旧版数据，正在迁移 ========================
          首次启动迁移：书籍/书源/规则/章节缓存等将导入 SQLite
          迁移期间请勿关闭窗口/进程（书架较大时可能需要数分钟——大书架会先备份原数据，
          备份阶段无日志属正常，请耐心等待）
          完成后会自动进入正常服务
        ====================================================================
"
    );
    std::io::stderr().flush().ok();
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&storage.pool)
        .await?;
    if user_count > 0 {
        tracing::info!("users 表已有 {} 条记录，跳过 JSON 迁移", user_count);
        // 补迁书源：book_sources 空且 data 目录有 bookSource.json 时导入（生产数据同步场景）
        let src_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM book_sources")
            .fetch_one(&storage.pool)
            .await?;
        if src_count == 0 {
            let namespaces = scan_source_namespaces(&data_dir);
            if !namespaces.is_empty() {
                match migrate_book_sources(&storage.pool, &data_dir, &namespaces).await {
                    Ok(n) => tracing::info!("补迁书源：{} 个（命名空间 {:?}）", n, namespaces),
                    Err(e) => tracing::warn!("补迁书源失败：{e}"),
                }
            }
        }
        // 补迁 RSS 源：rss_sources 空且 data 目录有 rssSource.json 时导入
        let rss_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rss_sources")
            .fetch_one(&storage.pool)
            .await?;
        if rss_count == 0 {
            let namespaces = scan_rss_namespaces(&data_dir);
            if !namespaces.is_empty() {
                match migrate_rss_sources(&storage.pool, &data_dir, &namespaces).await {
                    Ok(n) => tracing::info!("补迁 RSS 源：{} 个（命名空间 {:?}）", n, namespaces),
                    Err(e) => tracing::warn!("补迁 RSS 源失败：{e}"),
                }
            }
        }
        // 补迁书签：bookmarks 空且 data 目录有 bookmark.json 时导入（函数内幂等：表非空跳过）
        let namespaces = scan_namespaces_for(&data_dir, "bookmark.json");
        if !namespaces.is_empty() {
            match migrate_bookmarks(&storage.pool, &data_dir, &namespaces).await {
                Ok(n) => tracing::info!("补迁书签：{} 条（命名空间 {:?}）", n, namespaces),
                Err(e) => tracing::warn!("补迁书签失败：{e}"),
            }
        }
        // 补迁替换规则：replace_rules 空且 data 目录有 replaceRule.json 时导入
        let namespaces = scan_namespaces_for(&data_dir, "replaceRule.json");
        if !namespaces.is_empty() {
            match migrate_replace_rules(&storage.pool, &data_dir, &namespaces).await {
                Ok(n) => tracing::info!("补迁替换规则：{} 条（命名空间 {:?}）", n, namespaces),
                Err(e) => tracing::warn!("补迁替换规则失败：{e}"),
            }
        }
        // 补迁 TXT 目录规则：txt_toc_rules 空且 data 目录有 txtTocRule.json 时导入
        let namespaces = scan_namespaces_for(&data_dir, "txtTocRule.json");
        if !namespaces.is_empty() {
            match migrate_txt_toc_rules(&storage.pool, &data_dir, &namespaces).await {
                Ok(n) => tracing::info!("补迁 TXT 目录规则：{} 条（命名空间 {:?}）", n, namespaces),
                Err(e) => tracing::warn!("补迁 TXT 目录规则失败：{e}"),
            }
        }
        // 补迁 HttpTTS：http_tts_list 空且 data 目录有 httpTTS.json 时导入
        let namespaces = scan_namespaces_for(&data_dir, "httpTTS.json");
        if !namespaces.is_empty() {
            match migrate_http_tts(&storage.pool, &data_dir, &namespaces).await {
                Ok(n) => tracing::info!("补迁 HttpTTS：{} 个（命名空间 {:?}）", n, namespaces),
                Err(e) => tracing::warn!("补迁 HttpTTS 失败：{e}"),
            }
        }
        // 补迁分组：book_groups 空且 data 目录有 bookGroup.json 时导入
        let namespaces = scan_namespaces_for(&data_dir, "bookGroup.json");
        if !namespaces.is_empty() {
            match migrate_book_groups(&storage.pool, &data_dir, &namespaces).await {
                Ok(n) => tracing::info!("补迁分组：{} 个（命名空间 {:?}）", n, namespaces),
                Err(e) => tracing::warn!("补迁分组失败：{e}"),
            }
        }
        // 补迁用户配置：user_config 空且 data 目录有 userConfig.json 时导入
        let namespaces = scan_namespaces_for(&data_dir, "userConfig.json");
        if !namespaces.is_empty() {
            match migrate_user_configs(&storage.pool, &data_dir, &namespaces).await {
                Ok(n) => tracing::info!("补迁用户配置：{} 项（命名空间 {:?}）", n, namespaces),
                Err(e) => tracing::warn!("补迁用户配置失败：{e}"),
            }
        }
        // 补迁章节缓存：data/{ns}/{书名}_{作者}/{md5(book_url)}/{index}.txt → book_chapters
        //（历史上只迁了书籍元数据——正文缓存缺失导致书架有书但无已缓存章节）
        let mut need_tip = false;
        let namespaces = scan_namespaces_for(&data_dir, "bookshelf.json");
        if !namespaces.is_empty() {
            need_tip = true;
            match migrate_chapter_cache(&storage.pool, &data_dir, &namespaces).await {
                Ok(n) => tracing::info!("补迁章节缓存：{n} 章（命名空间 {:?}）", namespaces),
                Err(e) => tracing::warn!("补迁章节缓存失败：{e}"),
            }
        }
        if need_tip {
            use std::io::Write;
            eprintln!(
                "
================ 迁移完成（可正常使用） ================
  书籍/书源/规则/章节缓存已迁移至 SQLite
==================================================
"
            );
            std::io::stderr().flush().ok();
        }
        return Ok(());
    }

    // 1. 迁移前备份 storage/data → storage/backup-before-migrate-{ts}/
    let ts = Utc::now().format("%Y%m%d%H%M%S");
    let backup_dir = storage
        .config
        .storage_dir()
        .join(format!("backup-before-migrate-{ts}"));
    tracing::info!(
        "正在备份旧数据到 {}（大书架可能需要数分钟，请勿中断）",
        backup_dir.display()
    );
    copy_dir_recursive(&data_dir, &backup_dir)
        .with_context(|| format!("备份 storage/data → {} 失败", backup_dir.display()))?;
    tracing::info!("已备份 storage/data → {}", backup_dir.display());

    // 2. users.json → users 表
    let usernames = migrate_users(&storage.pool, &users_path).await?;

    // 3. 各命名空间 bookshelf.json → books 表（ns = default + 各用户名）
    let mut namespaces: Vec<String> = Vec::with_capacity(usernames.len() + 1);
    namespaces.push("default".to_string());
    namespaces.extend(usernames.iter().cloned());
    let book_count = migrate_bookshelves(&storage.pool, &data_dir, &namespaces).await?;

    // 3.5 章节正文缓存：data/{ns}/{书名}_{作者}/{md5}/{index}.txt → book_chapters
    let chapter_count = migrate_chapter_cache(&storage.pool, &data_dir, &namespaces).await?;
    tracing::info!("迁移章节缓存：{chapter_count} 章");

    // 4. 各命名空间 bookSource.json → book_sources 表（ns = default + 各用户名）
    let source_count = migrate_book_sources(&storage.pool, &data_dir, &namespaces).await?;

    // 5. 各命名空间 rssSource.json → rss_sources 表（ns = default + 各用户名）
    let rss_count = migrate_rss_sources(&storage.pool, &data_dir, &namespaces).await?;

    // 6. 各命名空间 bookmark.json → bookmarks 表（legacy 字段 content/time 映射 + raw_json 保底）
    let bookmark_count = migrate_bookmarks(&storage.pool, &data_dir, &namespaces).await?;

    // 7. 各命名空间 replaceRule.json → replace_rules 表（legacy 无 id → uuid）
    let replace_count = migrate_replace_rules(&storage.pool, &data_dir, &namespaces).await?;

    // 8. 各命名空间 txtTocRule.json → txt_toc_rules 表（legacy id 为 Long → 字符串化）
    let txt_toc_count = migrate_txt_toc_rules(&storage.pool, &data_dir, &namespaces).await?;

    // 9. 各命名空间 httpTTS.json → http_tts_list 表（legacy Long id 忽略，url 为主键）
    let http_tts_count = migrate_http_tts(&storage.pool, &data_dir, &namespaces).await?;

    // 10. 各命名空间 bookGroup.json → book_groups 表（legacy id 保留）
    let group_count = migrate_book_groups(&storage.pool, &data_dir, &namespaces).await?;

    // 11. 各命名空间 userConfig.json → user_config 表（{键:值} 对象 或 [{key,value}] 数组）
    let config_count = migrate_user_configs(&storage.pool, &data_dir, &namespaces).await?;

    tracing::info!(
        "JSON→SQLite 迁移完成：{} 个用户，{} 本书，{} 个书源，{} 个 RSS 源，{} 个书签，{} 条替换规则，{} 条 TXT 目录规则，{} 个 HttpTTS，{} 个分组，{} 项用户配置（备份：{}）",
        usernames.len(),
        book_count,
        source_count,
        rss_count,
        bookmark_count,
        replace_count,
        txt_toc_count,
        http_tts_count,
        group_count,
        config_count,
        backup_dir.display()
    );
    Ok(())
}

/// 管理员 default 残留个人数据回迁：把 default 中属于个人数据表的行移回
/// 管理员本人命名空间。
///
/// 设计：管理员默认使用本人账号命名空间（书架/进度/书签等），default 仅作为
/// 可手动进入的系统配置层（公用书源等配置）。历史版本曾把管理员 username
/// 命名空间整体并入 default，这里只回迁个人数据表；配置类表（书源/替换规则/
/// 订阅/RSS 源/TXT 目录规则/HttpTTS/分组）保留在 default 供系统配置。
/// 幂等：本人命名空间已有相同业务键时保留本人行并清理 default 残留行。
async fn migrate_admin_default_personal_data_back(pool: &SqlitePool) -> Result<usize> {
    let admins: Vec<String> = sqlx::query_scalar(
        "SELECT username FROM users WHERE is_admin = 1 AND username != 'default' \
         ORDER BY created_at, username",
    )
    .fetch_all(pool)
    .await?;
    let Some(username) = admins.first() else {
        return Ok(0);
    };

    // (表名, 业务冲突键)：仅个人数据表；配置类表不自动回迁
    const PERSONAL_TABLES: &[(&str, &[&str])] = &[
        ("books", &["book_url"]),
        ("book_source_cookies", &["source_url"]),
        ("bookmarks", &["book_url", "title"]),
        ("reading_stats", &["book_url", "date"]),
        ("rss_articles", &["url"]),
        ("user_config", &["ns"]),
    ];

    let mut tx = pool.begin().await?;
    let mut total = 0usize;
    for (table, keys) in PERSONAL_TABLES {
        let conflict_where = keys
            .iter()
            .map(|k| format!("d.{k} = t.{k}"))
            .collect::<Vec<_>>()
            .join(" AND ");
        let moved_sql = format!(
            "UPDATE {table} SET user_namespace = ?1 \
             WHERE user_namespace = 'default' AND rowid IN ( \
               SELECT t.rowid FROM {table} t \
               WHERE t.user_namespace = 'default' AND NOT EXISTS ( \
                 SELECT 1 FROM {table} d \
                 WHERE d.user_namespace = ?2 AND {conflict_where} \
               ) \
             )"
        );
        let moved = sqlx::query(&moved_sql)
            .bind(username)
            .bind(username)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        let drop_sql = format!(
            "DELETE FROM {table} \
             WHERE user_namespace = 'default' AND rowid NOT IN ( \
               SELECT rowid FROM {table} WHERE user_namespace = ?1 \
             )"
        );
        let dropped = sqlx::query(&drop_sql)
            .bind(username)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        total += (moved + dropped) as usize;
    }
    tx.commit().await?;
    Ok(total)
}

/// 从 books.raw_json 恢复漏写的 toc_url（旧迁移版本未写 toc_url 字段）。
/// 仅更新 toc_url 为空且有原始 tocUrl 的记录；返回补全数量。
async fn backfill_toc_url_from_raw(pool: &SqlitePool) -> Result<usize> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT book_url, user_namespace, raw_json FROM books \
         WHERE toc_url = '' AND raw_json IS NOT NULL AND raw_json != ''",
    )
    .fetch_all(pool)
    .await?;
    if rows.is_empty() {
        return Ok(0);
    }
    let mut tx = pool.begin().await?;
    let mut count = 0usize;
    for (book_url, ns, raw) in rows {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        let Some(toc_url) = value
            .get("tocUrl")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let updated = sqlx::query(
            "UPDATE books SET toc_url = ?1 WHERE book_url = ?2 AND user_namespace = ?3 AND toc_url = ''",
        )
        .bind(&toc_url)
        .bind(&book_url)
        .bind(&ns)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        count += updated as usize;
        if updated > 0 {
            tracing::debug!("补全 toc_url [{ns}] {book_url} → {toc_url}");
        }
    }
    tx.commit().await?;
    Ok(count)
}

/// users.json（Map<username, User>）→ users 表（全字段 + raw_json 原文保底）；返回迁移的用户名列表
async fn migrate_users(pool: &SqlitePool, path: &Path) -> Result<Vec<String>> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("读取 {} 失败", path.display()))?;
    let user_map: HashMap<String, serde_json::Value> =
        serde_json::from_str(&text).with_context(|| format!("解析 {} 失败", path.display()))?;

    let mut usernames = Vec::with_capacity(user_map.len());
    let mut tx = pool.begin().await?;
    for (key, value) in user_map {
        let mut user: User = match serde_json::from_value(value.clone()) {
            Ok(u) => u,
            Err(e) => {
                tracing::warn!("解析用户 {} 失败（{}），保留 raw_json", key, e);
                User {
                    username: key.clone(),
                    ..Default::default()
                }
            }
        };
        if user.username.is_empty() {
            user.username = key;
        }
        // user_namespace = username（用户数据命名空间）
        user.user_namespace = user.username.clone();
        // raw_json：原始 JSON 全量保底（未知字段不丢）
        user.raw_json = Some(value.to_string());
        // token_map：JSON 字符串（legacy Map<String, Long>）
        let token_map_json = user.token_map.as_ref().map(|v| v.to_string());
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO users
                (username, password, salt, token, token_map, enable_webdav, enable_local_store,
                 enable_book_source, enable_rss_source, book_source_limit, book_limit,
                 is_admin, last_login_at, created_at, user_namespace, raw_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            "#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.salt)
        .bind(&user.token)
        .bind(&token_map_json)
        .bind(user.enable_webdav)
        .bind(user.enable_local_store)
        .bind(user.enable_book_source)
        .bind(user.enable_rss_source)
        .bind(user.book_source_limit)
        .bind(user.book_limit)
        .bind(user.is_admin)
        .bind(user.last_login_at)
        .bind(user.created_at)
        .bind(&user.user_namespace)
        .bind(&user.raw_json)
        .execute(&mut *tx)
        .await?;
        usernames.push(user.username.clone());
        tracing::info!("迁移用户: {}", user.username);
    }
    tx.commit().await?;
    Ok(usernames)
}

/// 迁移章节正文缓存：`data/{ns}/{书名}_{作者}/{md5(book_url)}/{index}.txt` → book_chapters 表。
///
/// legacy 章节缓存布局：
/// - `{md5(book_url)}.json`：章节目录（list of {url, title, index, ...}——index 为章节序号）
/// - `{md5(book_url)}/{index}.txt`：章节正文（已剥离 HTML 的纯文本）
///
/// 幂等：书在 book_chapters 已有行则跳过（避免重复导入覆盖用户新缓存）。
/// 返回导入的章节总数。
async fn migrate_chapter_cache(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let mut total = 0usize;
    for ns in namespaces {
        let ns_dir = data_dir.join(ns);
        if !ns_dir.is_dir() {
            continue;
        }
        let books: Vec<(String,)> =
            sqlx::query_as("SELECT book_url FROM books WHERE user_namespace = ?1")
                .bind(ns)
                .fetch_all(pool)
                .await?;
        for (book_url,) in books {
            if book_url.trim().is_empty() {
                continue;
            }
            // 幂等：已有章节行则跳过（不覆盖用户新缓存）
            let has: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM book_chapters WHERE book_url = ?1")
                    .bind(&book_url)
                    .fetch_one(pool)
                    .await?;
            if has > 0 {
                continue;
            }
            let hex = crate::util::md5::md5_encode(&book_url);
            // 遍历用户目录下所有 {书名}_{作者} 子目录，找 {hex}.json（含换源前的旧缓存目录）
            let Ok(rd) = std::fs::read_dir(&ns_dir) else {
                continue;
            };
            for entry in rd.flatten() {
                let dir = entry.path();
                if !dir.is_dir() {
                    continue;
                }
                let toc_path = dir.join(format!("{hex}.json"));
                if !toc_path.exists() {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&toc_path) else {
                    continue;
                };
                let toc: Vec<serde_json::Value> = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let cache_dir = dir.join(&hex);
                if !cache_dir.is_dir() {
                    continue;
                }
                let mut inserted = 0usize;
                let mut tx = pool.begin().await?;
                for ch in toc {
                    let Some(idx) = ch.get("index").and_then(|v| v.as_i64()) else {
                        continue;
                    };
                    if idx < 0 {
                        continue;
                    }
                    let title = ch
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let txt_path = cache_dir.join(format!("{idx}.txt"));
                    let Ok(content) = std::fs::read_to_string(&txt_path) else {
                        continue;
                    };
                    sqlx::query(
                        "INSERT OR REPLACE INTO book_chapters (book_url, chapter_index, title, content, user_namespace) VALUES (?1, ?2, ?3, ?4, ?5)",
                    )
                    .bind(&book_url)
                    .bind(idx)
                    .bind(&title)
                    .bind(&content)
                    .bind(ns)
                    .execute(&mut *tx)
                    .await?;
                    inserted += 1;
                }
                tx.commit().await?;
                if inserted > 0 {
                    tracing::info!(
                        "迁移章节缓存 [{ns}]《{}》：{} 章",
                        dir.file_name().unwrap_or_default().to_string_lossy(),
                        inserted
                    );
                    total += inserted;
                }
            }
        }
    }
    Ok(total)
}

/// 各命名空间 bookshelf.json → books 表；返回迁移的书籍总数
async fn migrate_bookshelves(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("bookshelf.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 bookshelf.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let books: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        for value in books {
            let mut book: Book = match serde_json::from_value(value.clone()) {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("解析书籍失败（{}），跳过", e);
                    continue;
                }
            };
            if book.book_url.trim().is_empty() {
                continue; // 无主键的脏数据跳过
            }
            // raw_json：每本书原始 JSON 全量保底（未知字段不丢）
            book.raw_json = Some(value.to_string());
            book.user_namespace = ns.clone();
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO books
                    (book_url, name, author, origin, origin_name, toc_url, kind, custom_tag, cover_url,
                     custom_cover_url, intro, custom_intro, charset, type, group_name,
                     latest_chapter_title, latest_chapter_time, last_check_time, last_check_count,
                     total_chapter_num, dur_chapter_title, dur_chapter_index, dur_chapter_pos,
                     dur_chapter_time, word_count, can_update, order_num, origin_order,
                     use_replace_rule, variable, read_config, is_in_shelf, cbz, display_cover,
                     display_intro, local_epub, local_pdf, pdf, split_long_chapter,
                     last_check_error, info_html, toc_html, user_namespace, created_at, raw_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                        ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27,
                        ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40,
                        ?41, ?42, ?43, ?44, ?45)
                "#,
            )
            .bind(&book.book_url)
            .bind(&book.name)
            .bind(&book.author)
            .bind(&book.origin)
            .bind(&book.origin_name)
            .bind(&book.toc_url)
            .bind(&book.kind)
            .bind(&book.custom_tag)
            .bind(&book.cover_url)
            .bind(&book.custom_cover_url)
            .bind(&book.intro)
            .bind(&book.custom_intro)
            .bind(&book.charset)
            .bind(book.book_type)
            .bind(book.group)
            .bind(&book.latest_chapter_title)
            .bind(book.latest_chapter_time)
            .bind(book.last_check_time)
            .bind(book.last_check_count)
            .bind(book.total_chapter_num)
            .bind(&book.dur_chapter_title)
            .bind(book.dur_chapter_index)
            .bind(book.dur_chapter_pos)
            .bind(book.dur_chapter_time)
            .bind(&book.word_count)
            .bind(book.can_update)
            .bind(book.order)
            .bind(book.origin_order)
            .bind(book.use_replace_rule)
            .bind(&book.variable)
            .bind(book.read_config.as_ref().map(|v| v.to_string()))
            .bind(book.is_in_shelf)
            .bind(book.cbz)
            .bind(&book.display_cover)
            .bind(&book.display_intro)
            .bind(book.local_epub)
            .bind(book.local_pdf)
            .bind(book.pdf)
            .bind(book.split_long_chapter)
            .bind(&book.last_check_error)
            .bind(&book.info_html)
            .bind(&book.toc_html)
            .bind(&book.user_namespace)
            .bind(0i64) // created_at：迁移数据时间未知，置 0（顺序由 rowid 保持）
            .bind(&book.raw_json)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移书架 [{ns}]：{} 本", count);
        total += count;
    }
    Ok(total)
}

/// 递归拷贝目录（备份用）
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// 各命名空间 bookSource.json → book_sources 表（全字段 + raw_json 保底）；返回迁移的书源总数
async fn migrate_book_sources(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("bookSource.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 bookSource.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let sources: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        for value in sources {
            let mut src: BookSource = match serde_json::from_value(value.clone()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("解析书源失败（{}），跳过", e);
                    continue;
                }
            };
            if src.book_source_url.trim().is_empty() {
                continue; // 无主键的脏数据跳过
            }
            src.raw_json = Some(value.to_string());
            src.user_namespace = ns.clone();
            let val = |v: &Option<serde_json::Value>| v.as_ref().map(|x| x.to_string());
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO book_sources
                    (book_source_url, book_source_name, book_source_group, book_source_type,
                     book_url_pattern, custom_order, enabled, enabled_explore, enabled_cookie_jar,
                     concurrent_rate, js_lib, header, proxy_url, login_url, login_ui, login_check_js, login_js,
                     book_source_comment, variable_comment, last_update_time, respond_time,
                     weight, explore_url, search_url, rule_explore, rule_search, rule_book_info,
                     rule_toc, rule_content, rule_related, search_rule, explore_rule, book_info_rule, toc_rule,
                     content_rule, key, tag, logger, variable, user_namespace, raw_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                        ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29,
                        ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41)
                "#,
            )
            .bind(&src.book_source_url)
            .bind(&src.book_source_name)
            .bind(&src.book_source_group)
            .bind(src.book_source_type)
            .bind(&src.book_url_pattern)
            .bind(src.custom_order)
            .bind(src.enabled)
            .bind(src.enabled_explore)
            .bind(src.enabled_cookie_jar)
            .bind(&src.concurrent_rate)
            .bind(&src.js_lib)
            .bind(&src.header)
            .bind(&src.proxy_url)
            .bind(&src.login_url)
            .bind(&src.login_ui)
            .bind(&src.login_check_js)
            .bind(&src.login_js)
            .bind(&src.book_source_comment)
            .bind(&src.variable_comment)
            .bind(src.last_update_time)
            .bind(src.respond_time)
            .bind(src.weight)
            .bind(&src.explore_url)
            .bind(&src.search_url)
            .bind(val(&src.rule_explore))
            .bind(val(&src.rule_search))
            .bind(val(&src.rule_book_info))
            .bind(val(&src.rule_toc))
            .bind(val(&src.rule_content))
            .bind(val(&src.rule_related))
            .bind(val(&src.search_rule))
            .bind(val(&src.explore_rule))
            .bind(val(&src.book_info_rule))
            .bind(val(&src.toc_rule))
            .bind(val(&src.content_rule))
            .bind(&src.key)
            .bind(&src.tag)
            .bind(val(&src.logger))
            .bind(val(&src.variable))
            .bind(&src.user_namespace)
            .bind(&src.raw_json)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移书源 [{ns}]：{} 个", count);
        total += count;
    }
    Ok(total)
}

/// 扫描 data 目录中含 bookSource.json 的命名空间
fn scan_source_namespaces(data_dir: &Path) -> Vec<String> {
    let mut namespaces = Vec::new();
    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for e in entries.flatten() {
            if !e.path().is_dir() {
                continue;
            }
            if e.path().join("bookSource.json").exists() {
                if let Some(name) = e.file_name().to_str() {
                    namespaces.push(name.to_string());
                }
            }
        }
    }
    namespaces
}

/// 扫描 data 目录中含 rssSource.json 的命名空间
fn scan_rss_namespaces(data_dir: &Path) -> Vec<String> {
    let mut namespaces = Vec::new();
    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for e in entries.flatten() {
            if !e.path().is_dir() {
                continue;
            }
            if e.path().join("rssSource.json").exists() {
                if let Some(name) = e.file_name().to_str() {
                    namespaces.push(name.to_string());
                }
            }
        }
    }
    namespaces
}

/// 各命名空间 rssSource.json → rss_sources 表（raw_json 原文保底）；返回迁移的 RSS 源总数
async fn migrate_rss_sources(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("rssSource.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 rssSource.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let sources: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        for value in sources {
            let mut src: crate::model::RssSource = match serde_json::from_value(value.clone()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("解析 RSS 源失败（{}），跳过", e);
                    continue;
                }
            };
            if src.source_url.trim().is_empty() {
                continue; // 无主键的脏数据跳过
            }
            if src.source_name.is_empty() {
                src.source_name = src.source_url.clone();
            }
            src.raw_json = Some(value.to_string());
            src.user_namespace = ns.clone();
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO rss_sources
                    (rss_source_url, rss_source_name, rss_source_group, enabled,
                     user_namespace, raw_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
            )
            .bind(&src.source_url)
            .bind(&src.source_name)
            .bind(&src.source_group)
            .bind(src.enabled)
            .bind(&src.user_namespace)
            .bind(&src.raw_json)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移 RSS 源 [{ns}]：{} 个", count);
        total += count;
    }
    Ok(total)
}

/// 扫描 data 目录中含指定 legacy 文件的命名空间
fn scan_namespaces_for(data_dir: &Path, file: &str) -> Vec<String> {
    let mut namespaces = Vec::new();
    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for e in entries.flatten() {
            if !e.path().is_dir() {
                continue;
            }
            if e.path().join(file).exists() {
                if let Some(name) = e.file_name().to_str() {
                    namespaces.push(name.to_string());
                }
            }
        }
    }
    namespaces
}

/// 各命名空间 bookmark.json（legacy：Bookmark 实体 JSON——{time, bookName, bookAuthor,
/// chapterIndex, chapterPos, chapterName, bookText, content}，无 bookUrl）
/// → bookmarks 表：book_url ← bookName（legacy 无 URL，书名+作者为最稳定标识；作者为空时仅书名）、
/// title ← chapterName（同章多书签 → 追加 @time 消歧，避免主键折叠丢数据）、
/// paragraph_index ← chapterPos、chapter_index ← chapterIndex、created_at ← time；
/// raw_json 原文保底（bookText/content/bookAuthor 等不丢）。
/// 幂等：bookmarks 表非空即跳过（表空才迁）。
async fn migrate_bookmarks(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        tracing::info!("bookmarks 表已有 {} 条记录，跳过书签迁移", existing);
        return Ok(0);
    }
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("bookmark.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 bookmark.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let items: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        // 主键 (book_url, title) 去重表：同章多书签 → title 追加 @time 消歧
        let mut used_keys = std::collections::HashSet::new();
        for value in items {
            let get_str = |k: &str| {
                value
                    .get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let book_name = get_str("bookName");
            let book_author = get_str("bookAuthor");
            let chapter_name = get_str("chapterName");
            let book_text = get_str("bookText");
            if book_name.trim().is_empty() {
                tracing::warn!("书签缺 bookName，跳过");
                continue;
            }
            // legacy 无 bookUrl：书名（+作者）为稳定标识
            let book_url = if book_author.trim().is_empty() {
                book_name.clone()
            } else {
                format!("{book_name}::{book_author}")
            };
            // title ← chapterName；空则回退 bookText 截断；仍空则占位
            let base_title = if !chapter_name.trim().is_empty() {
                chapter_name.clone()
            } else if !book_text.trim().is_empty() {
                book_text.chars().take(40).collect()
            } else {
                "书签".to_string()
            };
            let time = value.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
            let mut title = base_title.clone();
            let mut suffix = 0u64;
            while !used_keys.insert((book_url.clone(), title.clone())) {
                suffix += 1;
                title = if suffix == 1 {
                    format!("{base_title}@{time}")
                } else {
                    format!("{base_title}@{time}#{suffix}")
                };
            }
            // legacy：chapterPos → paragraph_index、chapterIndex、time（毫秒时间戳）
            let paragraph_index = value
                .get("chapterPos")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let chapter_index = value
                .get("chapterIndex")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let content = get_str("content");
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO bookmarks
                    (book_url, title, book_name, book_author, paragraph_index, chapter_index,
                     chapter_name, book_text, content, created_at, user_namespace, raw_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
            )
            .bind(&book_url)
            .bind(&title)
            .bind(&book_name)
            .bind(&book_author)
            .bind(paragraph_index)
            .bind(chapter_index)
            .bind(&chapter_name)
            .bind(&book_text)
            .bind(&content)
            .bind(time)
            .bind(ns)
            .bind(value.to_string())
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移书签 [{ns}]：{} 条", count);
        total += count;
    }
    Ok(total)
}

/// 各命名空间 replaceRule.json（legacy：ReplaceRule 实体 JSON——{id, name, group, pattern,
/// replacement, scope, scopeTitle, scopeContent, isEnabled, isRegex, timeoutMillisecond, order}）
/// → replace_rules 表：find ← pattern、replace ← replacement、enable ← isEnabled、
/// order_num ← order；legacy id 为 Long → 字符串化，缺失补 uuid（对齐 saveReplaceRule/restore 语义）。
/// 幂等：replace_rules 表非空即跳过（表空才迁）。
async fn migrate_replace_rules(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM replace_rules")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        tracing::info!("replace_rules 表已有 {} 条记录，跳过替换规则迁移", existing);
        return Ok(0);
    }
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("replaceRule.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 replaceRule.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let items: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        for value in items {
            let get_str = |k: &str| {
                value
                    .get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let name = get_str("name");
            // legacy 真实字段为 pattern（非 find）
            let find = get_str("pattern");
            if name.trim().is_empty() || find.trim().is_empty() {
                tracing::warn!("替换规则缺 name/pattern，跳过");
                continue;
            }
            // legacy id 为 Long（数字）→ 统一字符串 id，缺失补 uuid
            let mut id = match value.get("id") {
                Some(v) => match v.as_str() {
                    Some(s) => s.to_string(),
                    None => v.to_string(),
                },
                None => String::new(),
            };
            if id.trim().is_empty() {
                id = uuid::Uuid::new_v4().simple().to_string();
            }
            // legacy 真实字段为 isEnabled（JsonProperty 注解）；enabled/enable 兼容变体
            let enabled = value
                .get("isEnabled")
                .or_else(|| value.get("enabled"))
                .or_else(|| value.get("enable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let order = value
                .get("order")
                .or_else(|| value.get("orderNum"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let get_bool =
                |k: &str, default: bool| value.get(k).and_then(|v| v.as_bool()).unwrap_or(default);
            let opt_str = |k: &str| {
                value
                    .get(k)
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            };
            let group = opt_str("group");
            let scope = opt_str("scope");
            let scope_title = get_bool("scopeTitle", false);
            let scope_content = get_bool("scopeContent", true);
            let is_regex = get_bool("isRegex", false);
            let timeout_millisecond = value
                .get("timeoutMillisecond")
                .and_then(|v| v.as_i64())
                .unwrap_or(3000);
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO replace_rules
                    (id, name, group_name, find, replace, scope, scope_title, scope_content,
                     is_regex, timeout_millisecond, enable, order_num, user_namespace)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
            )
            .bind(&id)
            .bind(&name)
            .bind(&group)
            .bind(&find)
            .bind(get_str("replacement"))
            .bind(&scope)
            .bind(scope_title)
            .bind(scope_content)
            .bind(is_regex)
            .bind(timeout_millisecond)
            .bind(enabled)
            .bind(order)
            .bind(ns)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移替换规则 [{ns}]：{} 条", count);
        total += count;
    }
    Ok(total)
}

/// 各命名空间 txtTocRule.json（legacy：[{id(Long),name,rule,serialNumber,enable}]）
/// → txt_toc_rules 表；legacy id 为 Long → 字符串化，缺失补 uuid。
/// 幂等：txt_toc_rules 表非空即跳过（表空才迁）。
async fn migrate_txt_toc_rules(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM txt_toc_rules")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        tracing::info!(
            "txt_toc_rules 表已有 {} 条记录，跳过 TXT 目录规则迁移",
            existing
        );
        return Ok(0);
    }
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("txtTocRule.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 txtTocRule.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let items: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        for value in items {
            let get_str = |k: &str| {
                value
                    .get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let name = get_str("name");
            let rule = get_str("rule");
            if name.trim().is_empty() || rule.trim().is_empty() {
                tracing::warn!("TXT 目录规则缺 name/rule，跳过");
                continue;
            }
            // legacy id 为 Long（数字）→ 字符串化；缺失补 uuid
            let mut id = match value.get("id") {
                Some(v) => match v.as_str() {
                    Some(s) => s.to_string(),
                    None => v.to_string(),
                },
                None => String::new(),
            };
            if id.trim().is_empty() {
                id = uuid::Uuid::new_v4().simple().to_string();
            }
            // enable/enabled 双兼容（legacy 变体）
            let enable = value
                .get("enable")
                .or_else(|| value.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let serial_number = value
                .get("serialNumber")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO txt_toc_rules
                    (id, name, rule, enable, serial_number, user_namespace)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
            )
            .bind(&id)
            .bind(&name)
            .bind(&rule)
            .bind(enable)
            .bind(serial_number)
            .bind(ns)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移 TXT 目录规则 [{ns}]：{} 条", count);
        total += count;
    }
    Ok(total)
}

/// 各命名空间 httpTTS.json（legacy：[{id(Long),name,url,type}]）
/// → http_tts_list 表（url 主键；legacy Long id 忽略——与模型 HttpTts 一致）。
/// 幂等：http_tts_list 表非空即跳过（表空才迁）。
async fn migrate_http_tts(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM http_tts_list")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        tracing::info!(
            "http_tts_list 表已有 {} 条记录，跳过 HttpTTS 迁移",
            existing
        );
        return Ok(0);
    }
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("httpTTS.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 httpTTS.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let items: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        for value in items {
            let get_str = |k: &str| {
                value
                    .get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let url = get_str("url");
            let name = get_str("name");
            if url.trim().is_empty() || name.trim().is_empty() {
                tracing::warn!("HttpTTS 缺 url/name，跳过");
                continue;
            }
            let tts_type = value.get("type").and_then(|v| v.as_i64()).unwrap_or(0);
            let enabled_cookie_jar = value
                .get("enabledCookieJar")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let last_update_time = value
                .get("lastUpdateTime")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let opt_str = |k: &str| {
                value
                    .get(k)
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            };
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO http_tts_list
                    (url, name, type, content_type, concurrent_rate, login_url, login_ui,
                     header, js_lib, enabled_cookie_jar, login_check_js, last_update_time, user_namespace)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
            )
            .bind(&url)
            .bind(&name)
            .bind(tts_type)
            .bind(opt_str("contentType"))
            .bind(opt_str("concurrentRate"))
            .bind(opt_str("loginUrl"))
            .bind(opt_str("loginUi"))
            .bind(opt_str("header"))
            .bind(opt_str("jsLib"))
            .bind(enabled_cookie_jar)
            .bind(opt_str("loginCheckJs"))
            .bind(last_update_time)
            .bind(ns)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移 HttpTTS [{ns}]：{} 个", count);
        total += count;
    }
    Ok(total)
}

/// 各命名空间 bookGroup.json（legacy：BookGroup 实体 JSON——{groupId, groupName, cover, order, show}）
/// → book_groups 表：id ← groupId、name ← groupName、order_num ← order（legacy id 保留，
/// books.group_name 引用不变）；cover/show 无对应列 → 不迁移。
/// 幂等：book_groups 表非空即跳过（表空才迁）。
async fn migrate_book_groups(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM book_groups")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        tracing::info!("book_groups 表已有 {} 条记录，跳过分组迁移", existing);
        return Ok(0);
    }
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("bookGroup.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 bookGroup.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let items: Vec<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        for value in items {
            // legacy 真实字段为 groupName/groupId（非 name/id）
            let name = value
                .get("groupName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if name.trim().is_empty() {
                tracing::warn!("分组缺 groupName，跳过");
                continue;
            }
            let id = value.get("groupId").and_then(|v| v.as_i64()).unwrap_or(0);
            let order = value.get("order").and_then(|v| v.as_i64()).unwrap_or(0);
            let cover = value
                .get("cover")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            let show = value.get("show").and_then(|v| v.as_bool()).unwrap_or(true);
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO book_groups
                    (id, name, cover, show, order_num, user_namespace)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
            )
            .bind(id)
            .bind(&name)
            .bind(&cover)
            .bind(show)
            .bind(order)
            .bind(ns)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }
        tx.commit().await?;
        tracing::info!("迁移分组 [{ns}]：{} 个", count);
        total += count;
    }
    Ok(total)
}

/// 各命名空间 userConfig.json（{键: 值} 对象 或 [{key,value}] 数组）
/// → user_config 表（(user_namespace, ns) 双主键；字符串值原样存，其余 JSON 序列化）。
/// 幂等：user_config 表非空即跳过（表空才迁）。
async fn migrate_user_configs(
    pool: &SqlitePool,
    data_dir: &Path,
    namespaces: &[String],
) -> Result<usize> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_config")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        tracing::info!("user_config 表已有 {} 条记录，跳过用户配置迁移", existing);
        return Ok(0);
    }
    let mut total = 0usize;
    for ns in namespaces {
        let path = data_dir.join(ns).join("userConfig.json");
        if !path.exists() {
            tracing::debug!("{ns} 无 userConfig.json，跳过");
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("读取 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let value: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("解析 {} 失败（{}），跳过该命名空间", path.display(), e);
                continue;
            }
        };
        let mut count = 0usize;
        let mut tx = pool.begin().await?;
        let raw_of = |v: &serde_json::Value| match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        match value {
            serde_json::Value::Object(map) => {
                for (key, v) in map {
                    sqlx::query(
                        r#"
                        INSERT OR REPLACE INTO user_config (user_namespace, ns, config, updated_at)
                        VALUES (?1, ?2, ?3, 0)
                        "#,
                    )
                    .bind(ns)
                    .bind(&key)
                    .bind(raw_of(&v))
                    .execute(&mut *tx)
                    .await?;
                    count += 1;
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    let Some(key) = item.get("key").and_then(|k| k.as_str()) else {
                        tracing::warn!("用户配置数组项缺 key，跳过");
                        continue;
                    };
                    let raw = match item.get("value") {
                        Some(v) => raw_of(v),
                        None => String::new(),
                    };
                    sqlx::query(
                        r#"
                        INSERT OR REPLACE INTO user_config (user_namespace, ns, config, updated_at)
                        VALUES (?1, ?2, ?3, 0)
                        "#,
                    )
                    .bind(ns)
                    .bind(key)
                    .bind(raw)
                    .execute(&mut *tx)
                    .await?;
                    count += 1;
                }
            }
            _ => {
                tracing::warn!("{} 既非对象也非数组，跳过该命名空间", path.display());
                tx.rollback().await?;
                continue;
            }
        }
        tx.commit().await?;
        tracing::info!("迁移用户配置 [{ns}]：{} 项", count);
        total += count;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Bookmark;
    use crate::storage::init;
    use crate::AppConfig;

    /// 独立临时目录（避免污染真实 storage/reader.db）
    fn test_dir(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("reader-migrate-test-{}-{tag}", std::process::id()))
    }

    /// 构造最小 legacy data 目录（users.json + 各命名空间 legacy 文件）
    fn write_legacy_files(data_dir: &Path) {
        let default = data_dir.join("default");
        std::fs::create_dir_all(&default).unwrap();
        std::fs::create_dir_all(data_dir.join("alice")).unwrap();
        // 书籍 + 章节正文缓存（legacy 布局：{书名}_{作者}/{md5(book_url)}.json 目录 + {md5}/{index}.txt 正文）
        let book_url = "三体::刘慈欣";
        std::fs::write(
            default.join("bookshelf.json"),
            format!(
                r#"[{{"bookUrl":"{book_url}","name":"三体","author":"刘慈欣","tocUrl":"https://book.com/santi/toc","durChapterIndex":1,"durChapterTitle":"第一章 起点"}}]"#
            ),
        )
        .unwrap();
        let hex = crate::util::md5::md5_encode(book_url);
        let book_dir = default.join("三体_刘慈欣");
        std::fs::create_dir_all(book_dir.join(&hex)).unwrap();
        std::fs::write(
            book_dir.join(format!("{hex}.json")),
            r#"[{"index":0,"title":"第一章 起点","url":"https://c/1"},
               {"index":1,"title":"第二章 面壁","url":"https://c/2"}]"#,
        )
        .unwrap();
        std::fs::write(book_dir.join(&hex).join("0.txt"), "正文一。").unwrap();
        std::fs::write(book_dir.join(&hex).join("1.txt"), "正文二。").unwrap();
        // 书签：legacy Bookmark 实体 JSON（无 bookUrl；同章两个书签——验证主键消歧）
        std::fs::write(
            default.join("bookmark.json"),
            r#"[
                {"time":1700000000000,"bookName":"三体","bookAuthor":"刘慈欣","chapterIndex":1,"chapterPos":120,"chapterName":"第一章 起点","bookText":"这是书签内容","content":"备注A"},
                {"time":1700000001000,"bookName":"三体","bookAuthor":"刘慈欣","chapterIndex":1,"chapterPos":300,"chapterName":"第一章 起点","bookText":"第二个书签","content":"备注B"}
            ]"#,
        )
        .unwrap();
        // 替换规则：legacy ReplaceRule 实体 JSON（pattern/replacement/isEnabled；id 为 Long）
        std::fs::write(
            default.join("replaceRule.json"),
            r#"[
                {"id":1700000000000,"name":"去广告","group":"通用","pattern":"广告","replacement":"","scope":"content","scopeTitle":false,"scopeContent":true,"isEnabled":true,"isRegex":true,"timeoutMillisecond":5000,"order":1},
                {"id":1700000001000,"name":"净化","pattern":"旧排版","replacement":"新排版","isEnabled":false,"order":2}
            ]"#,
        )
        .unwrap();
        // TXT 目录规则：legacy id 为 Long（数字）
        std::fs::write(
            default.join("txtTocRule.json"),
            r#"[
                {"id":1,"name":"章节","rule":"^第.+章$","serialNumber":0,"enable":true},
                {"id":2,"name":"卷","rule":"^卷.+","serialNumber":1,"enable":false}
            ]"#,
        )
        .unwrap();
        // HttpTTS：legacy HttpTTS 实体 JSON（无 type 字段；contentType/header 等保底不迁）
        std::fs::write(
            default.join("httpTTS.json"),
            r#"[
                {"id":1,"name":"在线TTS","url":"https://tts.example.com/synth","contentType":"audio/mpeg","concurrentRate":"0","loginUrl":"https://tts.example.com/login","loginUi":"[{\"type\":\"input\"}]","header":"{\"X-Token\":\"a\"}","jsLib":"lib.js","enabledCookieJar":true,"loginCheckJs":"java.ajax('x')","lastUpdateTime":1700000000000},
                {"id":2,"name":"本地引擎","url":"local://engine","lastUpdateTime":1700000001000}
            ]"#,
        )
        .unwrap();
        // 分组：legacy BookGroup 实体 JSON（groupId/groupName）
        std::fs::write(
            default.join("bookGroup.json"),
            r#"[
                {"groupId":1,"groupName":"玄幻","cover":"https://covers/玄幻.png","order":0,"show":true},
                {"groupId":2,"groupName":"言情","cover":null,"order":1,"show":false}
            ]"#,
        )
        .unwrap();
        // 用户配置：default 用 {键:值} 对象；alice 用 [{key,value}] 数组
        std::fs::write(
            default.join("userConfig.json"),
            r#"{"readerSetting":"{\"fontSize\":18}","theme":"dark"}"#,
        )
        .unwrap();
        std::fs::write(
            data_dir.join("alice/userConfig.json"),
            r#"[{"key":"font","value":"16"}]"#,
        )
        .unwrap();
    }

    /// 初始化存储（init 会自动执行 migrate_if_needed）
    async fn setup(tag: &str, with_legacy: bool) -> Storage {
        let dir = test_dir(tag);
        let _ = std::fs::remove_dir_all(&dir);
        let data = dir.join("storage/data");
        std::fs::create_dir_all(data.join("default")).unwrap();
        std::fs::write(
            data.join("users.json"),
            r#"{"alice":{"username":"alice","enableLocalStore":true}}"#,
        )
        .unwrap();
        if with_legacy {
            write_legacy_files(&data);
        }
        let mut config = AppConfig::from_env();
        config.work_dir = dir.to_string_lossy().into_owned();
        init(&config).await.expect("存储初始化失败")
    }

    async fn cleanup(storage: Storage, tag: &str) {
        storage.pool.close().await;
        let _ = std::fs::remove_dir_all(test_dir(tag));
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .unwrap()
    }

    /// 六类 legacy 文件全量迁移 + 字段映射断言
    #[tokio::test]
    async fn test_migrate_legacy_all_types() {
        let storage = setup("all", true).await;
        let pool = &storage.pool;

        // 用户 + 各表行数
        assert_eq!(count(pool, "users").await, 1);
        assert_eq!(count(pool, "bookmarks").await, 2);
        assert_eq!(count(pool, "replace_rules").await, 2);
        assert_eq!(count(pool, "txt_toc_rules").await, 2);
        assert_eq!(count(pool, "http_tts_list").await, 2);
        assert_eq!(count(pool, "book_groups").await, 2);
        assert_eq!(count(pool, "user_config").await, 3); // default 2 + alice 1

        // 书架：legacy tocUrl 必须写入 books.toc_url（历史缺陷回归）
        let toc_url: String = sqlx::query_scalar("SELECT toc_url FROM books WHERE book_url = ?1")
            .bind("三体::刘慈欣")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(toc_url, "https://book.com/santi/toc");

        // 书签：bookName/bookAuthor → book_url（书名::作者）；chapterName → title；
        // chapterPos → paragraph_index；time → created_at；同章第二枚书签 title 追加 @time 消歧
        let (
            chapter_index,
            created_at,
            paragraph_index,
            book_name,
            book_author,
            chapter_name,
            book_text,
            content,
        ): (i64, i64, i64, String, String, String, String, String) = sqlx::query_as(
            "SELECT chapter_index, created_at, paragraph_index, book_name, book_author,              chapter_name, book_text, content              FROM bookmarks WHERE book_url = ?1 AND title = ?2",
        )
        .bind("三体::刘慈欣")
        .bind("第一章 起点")
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(chapter_index, 1);
        assert_eq!(created_at, 1700000000000);
        assert_eq!(paragraph_index, 120, "chapterPos 应映射 paragraph_index");
        assert_eq!(book_name, "三体");
        assert_eq!(book_author, "刘慈欣");
        assert_eq!(chapter_name, "第一章 起点");
        assert_eq!(book_text, "这是书签内容");
        assert_eq!(
            content, "备注A",
            "legacy Bookmark.content 应入列而非仅 raw_json"
        );
        let raw: String = sqlx::query_scalar("SELECT raw_json FROM bookmarks WHERE title = ?1")
            .bind("第一章 起点")
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(
            raw.contains("这是书签内容") && raw.contains("备注A"),
            "legacy bookText/content 应保底在 raw_json: {raw}"
        );
        // 同章第二个书签：title 消歧后缀，bookText/content 均保留
        let (para2, raw2): (i64, String) =
            sqlx::query_as("SELECT paragraph_index, raw_json FROM bookmarks WHERE title = ?1")
                .bind("第一章 起点@1700000001000")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(para2, 300);
        assert!(
            raw2.contains("第二个书签") && raw2.contains("备注B"),
            "第二枚书签原文应保留: {raw2}"
        );
        let ns: String =
            sqlx::query_scalar("SELECT user_namespace FROM bookmarks WHERE title = ?1")
                .bind("第一章 起点")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(ns, "default");

        // 替换规则：legacy Long id → 字符串化；pattern/isEnabled 映射 find/enable
        let id: String = sqlx::query_scalar("SELECT id FROM replace_rules WHERE name = ?1")
            .bind("去广告")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(id, "1700000000000", "legacy Long id 应字符串化保留");
        let (find, enable, order_num): (String, i64, i64) =
            sqlx::query_as("SELECT find, enable, order_num FROM replace_rules WHERE name = ?1")
                .bind("净化")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(find, "旧排版", "legacy pattern 应映射 find");
        assert_eq!((enable, order_num), (0, 2), "isEnabled 应映射 enable");
        // legacy 扩展字段：去广告 全字段入列；净化 缺省字段用默认值
        let (group, scope, scope_title, scope_content, is_regex, timeout): (
            Option<String>,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
        ) = sqlx::query_as(
            "SELECT group_name, scope, scope_title, scope_content, is_regex, timeout_millisecond              FROM replace_rules WHERE name = ?1",
        )
        .bind("去广告")
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(group.as_deref(), Some("通用"));
        assert_eq!(scope.as_deref(), Some("content"));
        assert_eq!(
            (scope_title, scope_content, is_regex, timeout),
            (0, 1, 1, 5000)
        );
        let (scope2, scope_title2, scope_content2, is_regex2, timeout2): (
            Option<String>,
            i64,
            i64,
            i64,
            i64,
        ) = sqlx::query_as(
            "SELECT scope, scope_title, scope_content, is_regex, timeout_millisecond              FROM replace_rules WHERE name = ?1",
        )
        .bind("净化")
        .fetch_one(pool)
        .await
        .unwrap();
        assert!(scope2.is_none());
        assert_eq!(
            (scope_title2, scope_content2, is_regex2, timeout2),
            (0, 1, 0, 3000)
        );

        // TXT 目录规则：legacy Long id → 字符串化
        let (id, serial_number, enable): (String, i64, i64) =
            sqlx::query_as("SELECT id, serial_number, enable FROM txt_toc_rules WHERE name = ?1")
                .bind("章节")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(id, "1");
        assert_eq!((serial_number, enable), (0, 1));

        // HttpTTS：url 主键 + type + legacy 扩展字段全部入列
        let (
            name,
            tts_type,
            content_type,
            concurrent_rate,
            login_url,
            login_ui,
            header,
            js_lib,
            enabled_cookie_jar,
            login_check_js,
            last_update_time,
        ): (String, i64, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, bool, Option<String>, i64) =
            sqlx::query_as(
                "SELECT name, type, content_type, concurrent_rate, login_url, login_ui,                  header, js_lib, enabled_cookie_jar, login_check_js, last_update_time                  FROM http_tts_list WHERE url = ?1",
            )
            .bind("https://tts.example.com/synth")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(name, "在线TTS");
        assert_eq!(tts_type, 0);
        assert_eq!(content_type.as_deref(), Some("audio/mpeg"));
        assert_eq!(concurrent_rate.as_deref(), Some("0"));
        assert_eq!(login_url.as_deref(), Some("https://tts.example.com/login"));
        assert_eq!(login_ui.as_deref(), Some(r#"[{"type":"input"}]"#));
        assert_eq!(header.as_deref(), Some(r#"{"X-Token":"a"}"#));
        assert_eq!(js_lib.as_deref(), Some("lib.js"));
        assert!(enabled_cookie_jar);
        assert_eq!(login_check_js.as_deref(), Some("java.ajax('x')"));
        assert_eq!(last_update_time, 1700000000000);
        let last2: i64 =
            sqlx::query_scalar("SELECT last_update_time FROM http_tts_list WHERE url = ?1")
                .bind("local://engine")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(last2, 1700000001000);

        // 分组：groupId/groupName 映射 id/name + cover/show 入列
        let (id, order_num, cover, show): (i64, i64, Option<String>, bool) =
            sqlx::query_as("SELECT id, order_num, cover, show FROM book_groups WHERE name = ?1")
                .bind("玄幻")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!((id, order_num), (1, 0));
        assert_eq!(cover.as_deref(), Some("https://covers/玄幻.png"));
        assert!(show);
        let (id2, name2, show2): (i64, String, bool) =
            sqlx::query_as("SELECT id, name, show FROM book_groups WHERE id = ?1")
                .bind(2)
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(id2, 2);
        assert_eq!(name2, "言情", "legacy groupName 应映射 name");
        assert!(!show2, "legacy show=false 应入列");

        // 用户配置：对象 map（default）+ 数组（alice）
        let config: String = sqlx::query_scalar(
            "SELECT config FROM user_config WHERE user_namespace = ?1 AND ns = ?2",
        )
        .bind("default")
        .bind("readerSetting")
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(config, r#"{"fontSize":18}"#);
        let config: String = sqlx::query_scalar(
            "SELECT config FROM user_config WHERE user_namespace = ?1 AND ns = ?2",
        )
        .bind("alice")
        .bind("font")
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(config, "16");

        cleanup(storage, "all").await;
    }

    /// 历史缺陷补全：旧版本迁移漏写 toc_url，raw_json 保留 tocUrl 时应能批量恢复
    #[tokio::test]
    async fn test_backfill_toc_url_from_raw() {
        let storage = setup("toc-backfill", false).await;
        let pool = &storage.pool;
        // 模拟旧迁移产物：toc_url 为空但 raw_json 有 tocUrl
        sqlx::query(
            "INSERT INTO books (book_url, name, toc_url, raw_json, user_namespace) \
             VALUES (?1, ?2, '', ?3, 'default')",
        )
        .bind("https://legacy/book/1")
        .bind("旧书一")
        .bind(r#"{"bookUrl":"https://legacy/book/1","tocUrl":"https://legacy/book/1/toc"}"#)
        .execute(pool)
        .await
        .unwrap();
        // raw_json 无 tocUrl 的记录不应误补
        sqlx::query(
            "INSERT INTO books (book_url, name, toc_url, raw_json, user_namespace) \
             VALUES (?1, ?2, '', ?3, 'default')",
        )
        .bind("https://legacy/book/2")
        .bind("旧书二")
        .bind(r#"{"bookUrl":"https://legacy/book/2"}"#)
        .execute(pool)
        .await
        .unwrap();

        let n = backfill_toc_url_from_raw(pool).await.unwrap();
        assert_eq!(n, 1, "只有含 tocUrl 的记录应补全");
        let toc: String = sqlx::query_scalar("SELECT toc_url FROM books WHERE book_url = ?1")
            .bind("https://legacy/book/1")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(toc, "https://legacy/book/1/toc");
        let toc2: String = sqlx::query_scalar("SELECT toc_url FROM books WHERE book_url = ?1")
            .bind("https://legacy/book/2")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(toc2, "", "无 tocUrl 的记录保持原样");
        // 幂等：再跑一次不再更新
        assert_eq!(backfill_toc_url_from_raw(pool).await.unwrap(), 0);
        cleanup(storage, "toc-backfill").await;
    }

    /// 管理员 default 残留个人数据回迁本人命名空间：个人表移动、冲突保留本人、
    /// 配置类表保留 default、幂等
    #[tokio::test]
    async fn test_admin_default_personal_data_back() {
        let storage = setup("adminns", false).await;
        sqlx::query(
            "INSERT INTO users (username, password, salt, is_admin, user_namespace) \
             VALUES ('transwarp', 'x', 'x', 1, 'transwarp')",
        )
        .execute(&storage.pool)
        .await
        .unwrap();
        // 测试库 init 会把最早用户 alice 提升为管理员；迁移回迁只针对唯一管理员
        sqlx::query("UPDATE users SET is_admin = 0 WHERE username != 'transwarp'")
            .execute(&storage.pool)
            .await
            .unwrap();
        // default 残留个人书：独有书（应回迁）、同名书（本人已有则保留本人）
        sqlx::query(
            "INSERT INTO books (book_url, name, user_namespace) VALUES \
             ('https://default-only/a', 'default残留书', 'default'), \
             ('https://same/a', 'default同名书', 'default')",
        )
        .execute(&storage.pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO books (book_url, name, user_namespace) \
             VALUES ('https://same/a', '本人书', 'transwarp')",
        )
        .execute(&storage.pool)
        .await
        .unwrap();
        // 配置类表（公用书源）保留在 default，不随个人数据回迁
        sqlx::query(
            "INSERT INTO book_sources (book_source_url, book_source_name, user_namespace) \
             VALUES ('https://src/pub', '公用源', 'default')",
        )
        .execute(&storage.pool)
        .await
        .unwrap();

        let moved = migrate_admin_default_personal_data_back(&storage.pool)
            .await
            .unwrap();
        assert!(moved >= 2, "应回迁独有书并清理冲突残留，实际 {moved}");

        let books: Vec<(String, String)> = sqlx::query_as(
            "SELECT book_url, name FROM books WHERE user_namespace = 'transwarp' ORDER BY book_url",
        )
        .fetch_all(&storage.pool)
        .await
        .unwrap();
        assert_eq!(
            books,
            vec![
                (
                    "https://default-only/a".to_string(),
                    "default残留书".to_string()
                ),
                ("https://same/a".to_string(), "本人书".to_string()),
            ]
        );
        let default_leftover: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM books WHERE user_namespace = 'default'")
                .fetch_one(&storage.pool)
                .await
                .unwrap();
        assert_eq!(default_leftover, 0, "default 不应残留个人书架");

        let public_sources: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM book_sources WHERE user_namespace = 'default'",
        )
        .fetch_one(&storage.pool)
        .await
        .unwrap();
        assert_eq!(public_sources, 1, "公用书源保留在 default");

        // 幂等：第二次无操作
        assert_eq!(
            migrate_admin_default_personal_data_back(&storage.pool)
                .await
                .unwrap(),
            0
        );
        cleanup(storage, "adminns").await;
    }

    /// 幂等：重复执行迁移不产生重复数据（表非空即跳过）
    /// 章节正文缓存迁移：{书名}_{作者}/{md5(book_url)}/{index}.txt → book_chapters
    #[tokio::test]
    async fn test_migrate_chapter_cache() {
        let storage = setup("chap", true).await;
        migrate_if_needed(&storage).await.unwrap();
        let pool = &storage.pool;
        assert_eq!(count(pool, "book_chapters").await, 2, "两章正文都应迁入");
        let (title, content): (String, String) = sqlx::query_as(
            "SELECT title, content FROM book_chapters WHERE book_url = ?1 AND chapter_index = 0",
        )
        .bind("三体::刘慈欣")
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(title, "第一章 起点");
        assert_eq!(content, "正文一。");
        // 幂等：再跑一次不重复导入
        migrate_if_needed(&storage).await.unwrap();
        assert_eq!(count(pool, "book_chapters").await, 2);
        cleanup(storage, "chap").await;
    }

    #[tokio::test]
    async fn test_migrate_idempotent() {
        let storage = setup("idem", true).await;
        migrate_if_needed(&storage).await.unwrap();
        migrate_if_needed(&storage).await.unwrap();
        let pool = &storage.pool;
        assert_eq!(count(pool, "users").await, 1);
        assert_eq!(count(pool, "bookmarks").await, 2);
        assert_eq!(count(pool, "replace_rules").await, 2);
        assert_eq!(count(pool, "txt_toc_rules").await, 2);
        assert_eq!(count(pool, "http_tts_list").await, 2);
        assert_eq!(count(pool, "book_groups").await, 2);
        assert_eq!(count(pool, "user_config").await, 3);
        cleanup(storage, "idem").await;
    }

    /// 表非空才迁：bookmarks 已有数据 → 跳过书签迁移，其余类型照常补迁
    #[tokio::test]
    async fn test_migrate_skips_nonempty_table() {
        let storage = setup("skip", false).await; // 无 legacy 文件：迁移空转
        storage
            .save_bookmark(
                "default",
                &Bookmark {
                    book_url: "https://b.com/book".into(),
                    title: "已有书签".into(),
                    created_at: 1,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        // 写入 legacy 文件后手动再跑迁移（users 非空 → 补迁分支）
        write_legacy_files(&storage.config.storage_dir().join("data"));
        migrate_if_needed(&storage).await.unwrap();
        let pool = &storage.pool;
        assert_eq!(
            count(pool, "bookmarks").await,
            1,
            "bookmarks 非空应跳过迁移"
        );
        let title: String = sqlx::query_scalar("SELECT title FROM bookmarks")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(title, "已有书签", "不应混入 legacy 书签");
        assert_eq!(count(pool, "replace_rules").await, 2);
        assert_eq!(count(pool, "txt_toc_rules").await, 2);
        assert_eq!(count(pool, "http_tts_list").await, 2);
        assert_eq!(count(pool, "book_groups").await, 2);
        assert_eq!(count(pool, "user_config").await, 3);
        cleanup(storage, "skip").await;
    }
}
