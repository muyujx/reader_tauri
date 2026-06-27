#![allow(non_snake_case)]

use log::info;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

// 缓存目录状态
pub struct CacheDirState(pub Mutex<Option<PathBuf>>);

// 数据库状态
pub struct DbState(pub Mutex<Option<Connection>>);

// 获取缓存目录
pub fn get_cache_dir(state: &CacheDirState) -> PathBuf {
    let guard = state.0.lock().unwrap();
    guard.clone().unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("reader")
            .join("cache")
    })
}

#[tauri::command]
pub fn set_cache_dir(state: State<CacheDirState>, dir: String) -> Result<bool, String> {
    let path = PathBuf::from(&dir);
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = Some(path.clone());
    info!("[DB] Cache dir set to: {}", dir);
    Ok(true)
}

#[tauri::command]
pub fn get_cache_dir_cmd(state: State<CacheDirState>) -> Result<String, String> {
    let dir = get_cache_dir(&state);
    Ok(dir.to_string_lossy().to_string())
}

// 数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookInfo {
    pub book_id: i64,
    pub book_name: String,
    pub total_page: i64,
    pub cover_pic: Option<String>,
    pub big_cover_pic: Option<String>,
    pub tag_id: Option<i64>,
    pub read_page: Option<i64>,
    pub last_read_time: Option<i64>,
    pub reading_cost: Option<i64>,
    pub create_time: Option<i64>,
    // 已下载完成的页数（book_page.status=1 的行数），供下载列表页展示下载进度
    pub downloaded_pages: Option<i64>,
    // 下载完成百分比（0~100 整数），前端进度条直接消费
    pub progress: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookPage {
    pub id: i64,
    pub book_id: i64,
    pub page_idx: i64,
    pub content: Option<String>,
    pub title: Option<String>,
    pub top_chapter: Option<i64>,
    pub status: i64,
    pub create_time: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentsItem {
    pub level: i64,
    pub start_page: i64,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub exists: bool,
    pub downloaded_pages: i64,
    pub total_page: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageItem {
    pub content: String,
    pub title: String,
    pub page: i64,
    pub top_chapter: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub success: bool,
    pub book_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookListResult {
    pub content: Vec<BookInfo>,
    pub total: i64,
    pub total_page: i64,
}

// 初始化数据库
pub fn init_db(app: &AppHandle) -> Result<Connection, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let db_path = app_dir.join("book.db");
    info!("[DB] Database path: {:?}", db_path);

    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS book (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL UNIQUE,
            book_name TEXT NOT NULL,
            total_page INTEGER NOT NULL,
            cover_pic TEXT,
            big_cover_pic TEXT,
            tag_id INTEGER,
            read_page INTEGER DEFAULT 0,
            last_read_time INTEGER DEFAULT 0,
            reading_cost INTEGER DEFAULT 0,
            create_time INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS book_page (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL,
            page_idx INTEGER NOT NULL,
            content TEXT,
            title TEXT,
            top_chapter INTEGER,
            status INTEGER NOT NULL DEFAULT 0,
            create_time INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            UNIQUE(book_id, page_idx)
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_book_page_book_id ON book_page(book_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_book_page_idx ON book_page(book_id, page_idx)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS book_contents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL,
            level INTEGER NOT NULL,
            start_page INTEGER NOT NULL,
            label TEXT NOT NULL,
            UNIQUE(book_id, start_page)
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_book_contents_book_id ON book_contents(book_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS book_image (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id INTEGER NOT NULL,
            image_url TEXT NOT NULL,
            local_path TEXT,
            status INTEGER NOT NULL DEFAULT 0,
            create_time INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            UNIQUE(book_id, image_url)
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_book_image_book_id ON book_image(book_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    info!("[DB] Database initialized successfully");
    Ok(conn)
}

#[tauri::command]
pub fn book_get_download_progress(
    db: State<DbState>,
    bookId: i64,
) -> Result<DownloadProgress, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM book WHERE book_id = ?1",
            params![bookId],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !exists {
        return Ok(DownloadProgress {
            exists: false,
            downloaded_pages: 0,
            total_page: 0,
        });
    }

    let total_page: i64 = conn
        .query_row(
            "SELECT total_page FROM book WHERE book_id = ?1",
            params![bookId],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let downloaded_pages: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM book_page WHERE book_id = ?1 AND status = 1",
            params![bookId],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(DownloadProgress {
        exists: true,
        downloaded_pages,
        total_page,
    })
}

#[tauri::command]
pub fn book_get_page(
    _app: AppHandle,
    db: State<DbState>,
    bookId: i64,
    page: i64,
    useLocalImages: Option<bool>,
) -> Result<Option<PageItem>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let result = conn.query_row(
        "SELECT content, title, page_idx, top_chapter FROM book_page WHERE book_id = ?1 AND page_idx = ?2 AND status = 1",
        params![bookId, page],
        |row| {
            let content: Option<String> = row.get(0)?;
            let title: Option<String> = row.get(1)?;
            let page_idx: i64 = row.get(2)?;
            let top_chapter: Option<i64> = row.get(3)?;
            
            Ok(PageItem {
                content: content.unwrap_or_default(),
                title: title.unwrap_or_default(),
                page: page_idx,
                top_chapter: top_chapter.unwrap_or(0),
            })
        },
    );

    match result {
        Ok(mut item) => {
            if useLocalImages.unwrap_or(false) {
                let image_map = get_local_image_map(conn, bookId)?;
                if !image_map.is_empty() {
                    item.content = replace_image_urls(&item.content, &image_map);
                }
            }
            Ok(Some(item))
        }
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn book_get_info(
    _app: AppHandle,
    db: State<DbState>,
    bookId: i64,
    useLocalImages: Option<bool>,
) -> Result<Option<BookInfo>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let result = conn.query_row(
        // 顺带统计已下载完成的页数（status=1），供阅读页判断是否已下载完整。
        "SELECT b.book_id, b.book_name, b.total_page, b.cover_pic, b.big_cover_pic, b.tag_id,
                b.read_page, b.last_read_time, b.reading_cost, b.create_time,
                (SELECT COUNT(*) FROM book_page p WHERE p.book_id = b.book_id AND p.status = 1) AS downloaded_pages
         FROM book b WHERE b.book_id = ?1",
        params![bookId],
        |row| {
            let total_page: i64 = row.get(2)?;
            let downloaded_pages: i64 = row.get::<_, i64>(10)?;
            // 进度百分比：0~100 整数，total_page<=0 时记 0 避免除零
            let progress = if total_page > 0 {
                (downloaded_pages * 100) / total_page
            } else {
                0
            };
            let mut cover_pic: Option<String> = row.get(3)?;
            let mut big_cover_pic: Option<String> = row.get(4)?;

            if useLocalImages.unwrap_or(false) {
                if let Some(ref cp) = cover_pic {
                    cover_pic = Some(replace_cover_url(cp));
                }
                if let Some(ref bcp) = big_cover_pic {
                    big_cover_pic = Some(replace_cover_url(bcp));
                }
            }

            Ok(BookInfo {
                book_id: row.get(0)?,
                book_name: row.get(1)?,
                total_page,
                cover_pic,
                big_cover_pic,
                tag_id: row.get(5)?,
                read_page: row.get(6)?,
                last_read_time: row.get(7)?,
                reading_cost: row.get(8)?,
                create_time: row.get(9)?,
                downloaded_pages: Some(downloaded_pages),
                progress: Some(progress),
            })
        },
    );

    match result {
        Ok(info) => Ok(Some(info)),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn book_is_downloaded(db: State<DbState>, bookId: i64) -> Result<bool, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM book WHERE book_id = ?1",
            params![bookId],
            |row| row.get(0),
        )
        .unwrap_or(false);

    Ok(exists)
}

#[tauri::command]
pub fn book_delete(
    app: AppHandle,
    db: State<DbState>,
    bookId: i64,
) -> Result<DownloadResult, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    conn.execute("DELETE FROM book_page WHERE book_id = ?1", params![bookId])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM book WHERE book_id = ?1", params![bookId])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM book_image WHERE book_id = ?1", params![bookId])
        .map_err(|e| e.to_string())?;

    // 删除下载的图片文件
    {
        let mut stmt = conn
            .prepare("SELECT local_path FROM book_image WHERE book_id = ?1")
            .map_err(|e| e.to_string())?;
        let paths: Vec<String> = stmt
            .query_map(params![bookId], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        for path in &paths {
            let p = std::path::Path::new(path);
            if p.exists() {
                let _ = std::fs::remove_file(p);
            }
        }
        // 尝试清理空目录（从文件路径向上遍历删除空目录）
        for path in &paths {
            let p = std::path::Path::new(path);
            let mut parent = p.parent();
            while let Some(dir) = parent {
                if dir.starts_with(get_downloaded_images_dir(&app).unwrap_or_default()) {
                    if dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(false) {
                        let _ = std::fs::remove_dir(dir);
                    }
                }
                parent = dir.parent();
            }
        }
    }
    conn.execute("DELETE FROM book_image WHERE book_id = ?1", params![bookId])
        .map_err(|e| e.to_string())?;

    info!("[DB] Book deleted: bookId={}", bookId);
    Ok(DownloadResult {
        success: true,
        book_id: bookId,
    })
}

#[tauri::command]
pub fn book_get_all_list(db: State<DbState>) -> Result<Vec<BookInfo>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn.prepare(
        // 关联子查询统计已下载完成的页数（status=1），一次性返回 downloaded_pages，
        // 前端下载列表页据此显示下载进度，无需再发起额外请求。
        "SELECT b.book_id, b.book_name, b.total_page, b.cover_pic, b.big_cover_pic, b.tag_id,
                b.read_page, b.last_read_time, b.reading_cost, b.create_time,
                (SELECT COUNT(*) FROM book_page p WHERE p.book_id = b.book_id AND p.status = 1) AS downloaded_pages
         FROM book b
         ORDER BY b.create_time DESC"
    ).map_err(|e| e.to_string())?;

    let books = stmt
        .query_map([], |row| {
            let total_page: i64 = row.get(2)?;
            let downloaded_pages: i64 = row.get::<_, i64>(10)?;
            // 进度百分比：0~100 整数，total_page<=0 时记 0 避免除零
            let progress = if total_page > 0 {
                (downloaded_pages * 100) / total_page
            } else {
                0
            };
            Ok(BookInfo {
                book_id: row.get(0)?,
                book_name: row.get(1)?,
                total_page,
                cover_pic: row.get(3)?,
                big_cover_pic: row.get(4)?,
                tag_id: row.get(5)?,
                read_page: row.get(6)?,
                last_read_time: row.get(7)?,
                reading_cost: row.get(8)?,
                create_time: row.get(9)?,
                downloaded_pages: Some(downloaded_pages),
                progress: Some(progress),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for book in books {
        if let Ok(b) = book {
            result.push(b);
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn book_get_list_by_page(
    db: State<DbState>,
    page: i64,
    pageSize: i64,
) -> Result<BookListResult, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM book", [], |row| row.get(0))
        .unwrap_or(0);

    let offset = (page - 1) * pageSize;

    let mut stmt = conn.prepare(
        // 关联子查询统计已下载完成的页数（status=1），分页查询同时拿到 downloaded_pages。
        "SELECT b.book_id, b.book_name, b.total_page, b.cover_pic, b.big_cover_pic, b.tag_id,
                b.read_page, b.last_read_time, b.reading_cost, b.create_time,
                (SELECT COUNT(*) FROM book_page p WHERE p.book_id = b.book_id AND p.status = 1) AS downloaded_pages
         FROM book b
         ORDER BY b.create_time DESC LIMIT ?1 OFFSET ?2"
    ).map_err(|e| e.to_string())?;

    let books = stmt
        .query_map(params![pageSize, offset], |row| {
            let total_page: i64 = row.get(2)?;
            let downloaded_pages: i64 = row.get::<_, i64>(10)?;
            // 进度百分比：0~100 整数，total_page<=0 时记 0 避免除零
            let progress = if total_page > 0 {
                (downloaded_pages * 100) / total_page
            } else {
                0
            };
            Ok(BookInfo {
                book_id: row.get(0)?,
                book_name: row.get(1)?,
                total_page,
                cover_pic: row.get(3)?,
                big_cover_pic: row.get(4)?,
                tag_id: row.get(5)?,
                read_page: row.get(6)?,
                last_read_time: row.get(7)?,
                reading_cost: row.get(8)?,
                create_time: row.get(9)?,
                downloaded_pages: Some(downloaded_pages),
                progress: Some(progress),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut content = Vec::new();
    for book in books {
        if let Ok(b) = book {
            content.push(b);
        }
    }

    let total_page = (total + pageSize - 1) / pageSize;

    Ok(BookListResult {
        content,
        total,
        total_page,
    })
}

#[tauri::command]
pub fn book_update_read_progress(
    db: State<DbState>,
    bookId: i64,
    readPage: i64,
    readingCost: i64,
) -> Result<DownloadResult, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "UPDATE book SET read_page = ?1, last_read_time = ?2, reading_cost = ?3 WHERE book_id = ?4",
        params![readPage, now, readingCost, bookId],
    )
    .map_err(|e| e.to_string())?;

    info!(
        "[DB] Read progress updated: bookId={}, page={}, cost={}",
        bookId, readPage, readingCost
    );
    Ok(DownloadResult {
        success: true,
        book_id: bookId,
    })
}

#[tauri::command]
pub fn book_save_page(
    db: State<DbState>,
    bookId: i64,
    pageIdx: i64,
    content: String,
    title: String,
    topChapter: i64,
) -> Result<DownloadResult, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO book_page (book_id, page_idx, content, title, top_chapter, status, create_time)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        params![bookId, pageIdx, content, title, topChapter, now],
    ).map_err(|e| e.to_string())?;

    Ok(DownloadResult {
        success: true,
        book_id: bookId,
    })
}

#[tauri::command]
pub fn book_get_local_image(
    app: tauri::AppHandle,
    bookId: i64,
    imageUrl: String,
) -> Result<Option<String>, String> {
    info!(
        "[Image] Loading local image: bookId={}, url={}",
        bookId, imageUrl
    );

    // 获取应用数据目录
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = app_dir.join("reader").join("cache");
    let resource_dir = cache_dir.join("resource").join(bookId.to_string());

    info!("[Image] Checking cache dir: {:?}", resource_dir);

    // 从 URL 提取文件路径
    let url_path = if imageUrl.starts_with("/resource") {
        imageUrl.trim_start_matches("/resource").to_string()
    } else if imageUrl.contains("resource") {
        let idx = imageUrl.find("resource").unwrap_or(0);
        imageUrl[idx + 8..].to_string()
    } else {
        imageUrl
    };

    info!("[Image] URL path: {}", url_path);

    // 构建本地文件路径
    let local_path = resource_dir.join(url_path.trim_start_matches('/'));

    info!("[Image] Local path: {:?}", local_path);

    // 读取文件
    if !local_path.exists() {
        info!("[Image] File not found in cache");
        return Ok(None);
    }

    let data = std::fs::read(&local_path).map_err(|e| e.to_string())?;
    let base64 = base64_encode(&data);

    // 根据文件扩展名确定 MIME 类型
    let mime_type = local_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        })
        .unwrap_or("image/jpeg");

    info!("[Image] Successfully loaded image");
    Ok(Some(format!("data:{};base64,{}", mime_type, base64)))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(CHARS[b0 >> 2] as char);
        result.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[tauri::command]
pub fn book_get_local_contents(
    db: State<DbState>,
    bookId: i64,
) -> Result<Vec<ContentsItem>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT level, start_page, label FROM book_contents WHERE book_id = ?1 ORDER BY start_page")
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(params![bookId], |row| {
            Ok(ContentsItem {
                level: row.get(0)?,
                start_page: row.get(1)?,
                label: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

// ================= 图片下载相关命令 =================

/// 获取已下载图片的 URL → 相对路径 映射
fn get_local_image_map(conn: &Connection, book_id: i64) -> Result<HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT image_url, local_path FROM book_image WHERE book_id = ?1 AND status = 1")
        .map_err(|e| e.to_string())?;

    let map: HashMap<String, String> = stmt
        .query_map(params![book_id], |row| {
            let url: String = row.get(0)?;
            let path: String = row.get(1)?;
            Ok((url, path))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(map)
}

/// 从图片 URL 提取相对路径（去掉 /resource 前缀）
pub fn extract_url_path(image_url: &str) -> String {
    let trimmed = if image_url.starts_with("/resource") {
        image_url.trim_start_matches("/resource")
    } else if image_url.contains("resource") {
        let idx = image_url.find("resource").unwrap_or(0);
        &image_url[idx + 8..]
    } else {
        image_url
    };
    trimmed.trim_start_matches('/').to_string()
}

/// 构建本地图片协议 URL
/// 格式: localimg://{urlPath}，urlPath 是去掉 /resource 前缀后的路径
fn build_local_image_url(url_path: &str) -> String {
    format!("localimg://{}", url_path)
}

/// 将 HTML 中的远程图片 URL 替换为本地图片协议 URL
fn replace_image_urls(html: &str, image_map: &HashMap<String, String>) -> String {
    let mut result = html.to_string();
    for image_url in image_map.keys() {
        let url_path = extract_url_path(image_url);
        let local_url = build_local_image_url(&url_path);
        result = result.replace(image_url.as_str(), &local_url);
    }
    result
}

/// 将封面图 URL 替换为本地协议 URL
fn replace_cover_url(cover_url: &str) -> String {
    if cover_url.is_empty() {
        return cover_url.to_string();
    }
    let url_path = extract_url_path(cover_url);
    build_local_image_url(&url_path)
}

/// 获取下载图片的目录路径
pub fn get_downloaded_images_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_dir.join("reader").join("downloaded"))
}

/// 解析图片 URL 得到本地存储路径
pub fn resolve_image_local_path(app: &AppHandle, image_url: &str) -> Result<PathBuf, String> {
    let url_path = extract_url_path(image_url);
    let dir = get_downloaded_images_dir(app)?;
    Ok(dir.join(url_path))
}

/// 插入或更新图片下载记录
#[tauri::command]
pub fn book_save_image_record(
    app: AppHandle,
    db: State<DbState>,
    bookId: i64,
    imageUrl: String,
) -> Result<bool, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let local_path = resolve_image_local_path(&app, &imageUrl).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR IGNORE INTO book_image (book_id, image_url, local_path, status, create_time)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![bookId, imageUrl, local_path.to_string_lossy().to_string(), now],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 更新图片下载状态为已下载
#[tauri::command]
pub fn book_mark_image_downloaded(
    app: AppHandle,
    db: State<DbState>,
    bookId: i64,
    imageUrl: String,
) -> Result<bool, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().timestamp();
    let local_path = resolve_image_local_path(&app, &imageUrl).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE book_image SET status = 1, local_path = ?1, create_time = ?2
         WHERE book_id = ?3 AND image_url = ?4",
        params![local_path.to_string_lossy().to_string(), now, bookId, imageUrl],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// 获取待下载的图片列表
#[tauri::command]
pub fn book_get_pending_images(
    db: State<DbState>,
    bookId: i64,
) -> Result<Vec<String>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT image_url FROM book_image WHERE book_id = ?1 AND status = 0 ORDER BY id")
        .map_err(|e| e.to_string())?;

    let urls = stmt
        .query_map(params![bookId], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(urls)
}

/// 批量插入图片下载记录
pub fn batch_insert_image_records(
    conn: &Connection,
    book_id: i64,
    image_urls: &[String],
    app: &AppHandle,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    for url in image_urls {
        let local_path = resolve_image_local_path(app, url)?;
        conn.execute(
            "INSERT OR IGNORE INTO book_image (book_id, image_url, local_path, status, create_time)
             VALUES (?1, ?2, ?3, 0, ?4)",
            params![book_id, url, local_path.to_string_lossy().to_string(), now],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
