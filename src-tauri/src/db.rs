#![allow(non_snake_case)]

use log::info;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
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

    info!("[DB] Database initialized successfully");
    Ok(conn)
}

// 命令实现
#[tauri::command]
pub fn book_download(
    db: State<DbState>,
    bookId: i64,
    bookName: String,
    totalPage: i64,
    coverPic: String,
    bigCoverPic: String,
    tagId: i64,
) -> Result<DownloadResult, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO book (book_id, book_name, total_page, cover_pic, big_cover_pic, tag_id, create_time)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![bookId, bookName, totalPage, coverPic, bigCoverPic, tagId, now],
    ).map_err(|e| e.to_string())?;

    for i in 1..=totalPage {
        conn.execute(
            "INSERT OR IGNORE INTO book_page (book_id, page_idx, status, create_time)
             VALUES (?1, ?2, 0, ?3)",
            params![bookId, i, now],
        )
        .map_err(|e| e.to_string())?;
    }

    info!("[DB] Book downloaded: bookId={}", bookId);
    Ok(DownloadResult {
        success: true,
        book_id: bookId,
    })
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
    db: State<DbState>,
    bookId: i64,
    page: i64,
) -> Result<Option<PageItem>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let result = conn.query_row(
        "SELECT content, title, page_idx, top_chapter FROM book_page WHERE book_id = ?1 AND page_idx = ?2",
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
        Ok(item) => Ok(Some(item)),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn book_get_info(db: State<DbState>, bookId: i64) -> Result<Option<BookInfo>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let result = conn.query_row(
        "SELECT book_id, book_name, total_page, cover_pic, big_cover_pic, tag_id, read_page, last_read_time, reading_cost, create_time 
         FROM book WHERE book_id = ?1",
        params![bookId],
        |row| {
            Ok(BookInfo {
                book_id: row.get(0)?,
                book_name: row.get(1)?,
                total_page: row.get(2)?,
                cover_pic: row.get(3)?,
                big_cover_pic: row.get(4)?,
                tag_id: row.get(5)?,
                read_page: row.get(6)?,
                last_read_time: row.get(7)?,
                reading_cost: row.get(8)?,
                create_time: row.get(9)?,
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
pub fn book_delete(db: State<DbState>, bookId: i64) -> Result<bool, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    conn.execute("DELETE FROM book_page WHERE book_id = ?1", params![bookId])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM book WHERE book_id = ?1", params![bookId])
        .map_err(|e| e.to_string())?;

    info!("[DB] Book deleted: bookId={}", bookId);
    Ok(true)
}

#[tauri::command]
pub fn book_get_all_list(db: State<DbState>) -> Result<Vec<BookInfo>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn.prepare(
        "SELECT book_id, book_name, total_page, cover_pic, big_cover_pic, tag_id, read_page, last_read_time, reading_cost, create_time 
         FROM book ORDER BY create_time DESC"
    ).map_err(|e| e.to_string())?;

    let books = stmt
        .query_map([], |row| {
            Ok(BookInfo {
                book_id: row.get(0)?,
                book_name: row.get(1)?,
                total_page: row.get(2)?,
                cover_pic: row.get(3)?,
                big_cover_pic: row.get(4)?,
                tag_id: row.get(5)?,
                read_page: row.get(6)?,
                last_read_time: row.get(7)?,
                reading_cost: row.get(8)?,
                create_time: row.get(9)?,
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
        "SELECT book_id, book_name, total_page, cover_pic, big_cover_pic, tag_id, read_page, last_read_time, reading_cost, create_time 
         FROM book ORDER BY create_time DESC LIMIT ?1 OFFSET ?2"
    ).map_err(|e| e.to_string())?;

    let books = stmt
        .query_map(params![pageSize, offset], |row| {
            Ok(BookInfo {
                book_id: row.get(0)?,
                book_name: row.get(1)?,
                total_page: row.get(2)?,
                cover_pic: row.get(3)?,
                big_cover_pic: row.get(4)?,
                tag_id: row.get(5)?,
                read_page: row.get(6)?,
                last_read_time: row.get(7)?,
                reading_cost: row.get(8)?,
                create_time: row.get(9)?,
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
) -> Result<bool, String> {
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
    Ok(true)
}

#[tauri::command]
pub fn book_save_page(
    db: State<DbState>,
    bookId: i64,
    pageIdx: i64,
    content: String,
    title: String,
    topChapter: i64,
) -> Result<bool, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO book_page (book_id, page_idx, content, title, top_chapter, status, create_time)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        params![bookId, pageIdx, content, title, topChapter, now],
    ).map_err(|e| e.to_string())?;

    Ok(true)
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
