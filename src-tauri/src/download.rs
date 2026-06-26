use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{sleep, Duration};

use crate::db::DbState;
use crate::http::{get_http_client, HttpState};

/// 后端会话失效事件名：下载过程中检测到 cookie 失效时
/// 会向前端 emit 这个事件，前端可据此跳转登录页重新登录。
pub const SESSION_EXPIRED_EVENT: &str = "book_download_session_expired";

struct DownloadTask {
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
}

pub struct DownloadManager {
    tasks: Mutex<HashMap<i64, DownloadTask>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressPayload {
    pub book_id: i64,
    pub downloaded_pages: i64,
    pub total_page: i64,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    code: i64,
    data: Option<T>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct PageItem {
    content: String,
    title: String,
    page: i64,
    #[serde(rename = "topChapter")]
    top_chapter: i64,
}

#[derive(Serialize)]
struct PageRequest {
    book_id: i64,
    start_page: i64,
    page_size: i64,
}

#[tauri::command]
pub async fn book_download(
    app: AppHandle,
    db: State<'_, DbState>,
    manager: State<'_, DownloadManager>,
    http_state: State<'_, HttpState>,
    bookId: i64,
    bookName: String,
    totalPage: i64,
    coverPic: String,
    bigCoverPic: String,
    tagId: i64,
    serverHost: String,
) -> Result<bool, String> {
    info!(
        "[Download] book_download called: bookId={}, totalPage={}",
        bookId, totalPage
    );

    {
        let guard = db.0.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("Database not initialized")?;

        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO book (book_id, book_name, total_page, cover_pic, big_cover_pic, tag_id, create_time)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![bookId, bookName, totalPage, coverPic, bigCoverPic, tagId, now],
        )
        .map_err(|e| e.to_string())?;

        for i in 1..=totalPage {
            conn.execute(
                "INSERT OR IGNORE INTO book_page (book_id, page_idx, status, create_time)
                 VALUES (?1, ?2, 0, ?3)",
                rusqlite::params![bookId, i, now],
            )
            .map_err(|e| e.to_string())?;
        }

        info!("[Download] Book record created: bookId={}", bookId);
    }

    {
        let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
        if tasks.contains_key(&bookId) {
            info!("[Download] Book already downloading: bookId={}", bookId);
            return Ok(false);
        }
    }

    let paused = Arc::new(AtomicBool::new(false));
    let cancelled = Arc::new(AtomicBool::new(false));
    let task = DownloadTask {
        paused: paused.clone(),
        cancelled: cancelled.clone(),
    };

    {
        let mut tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
        tasks.insert(bookId, task);
    }

    // 使用全局带 cookie 的 HTTP Client：登录态由 reqwest Jar 维护，
// 前台请求 / 登录都已自动把 cookie 写入 jar 并持久化到磁盘，
// 后台下载直接复用即可，无需任何 cookie 注入逻辑。
let client = get_http_client(&http_state);

info!(
    "[Download] Download task registered: bookId={}, using global cookie client",
    bookId,
);

    let host = serverHost.trim_end_matches('/').to_string();

    tokio::spawn(async move {
        if let Err(e) = background_download(
            app.clone(),
            bookId,
            totalPage,
            host,
            client,
            paused,
            cancelled,
        )
        .await
        {
            info!("[Download] Background download error for bookId={}: {}", bookId, e);
        }

        {
            let download_mgr = app.state::<DownloadManager>();
            let mut tasks = download_mgr.tasks.lock().unwrap();
            tasks.remove(&bookId);
            info!("[Download] Download task cleaned up: bookId={}", bookId);
        }
    });

    Ok(true)
}

async fn background_download(
    app: AppHandle,
    book_id: i64,
    total_page: i64,
    server_host: String,
    client: reqwest::Client,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
) -> Result<(), String> {
    info!(
        "[Download] Background task start: bookId={}, serverHost={}, totalPages={}",
        book_id, server_host, total_page,
    );

    let page_url = format!("{}/api/book/page/html/page", server_host);
    let mut last_progress_pct = 0i64;

    let mut page = 1i64;
    while page <= total_page {
        if cancelled.load(Ordering::Relaxed) {
            info!("[Download] Download cancelled: bookId={}", book_id);
            break;
        }

        while paused.load(Ordering::Relaxed) {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            sleep(Duration::from_millis(500)).await;
        }

        if cancelled.load(Ordering::Relaxed) {
            break;
        }

        let req_body = PageRequest {
            book_id,
            start_page: page,
            page_size: 1,
        };

        match client.post(&page_url).json(&req_body).send().await {
            Ok(resp) => {
                let status = resp.status();

                // HTTP 层会话失效（401/403）：通知前端重新登录并终止下载
                if status == reqwest::StatusCode::UNAUTHORIZED
                    || status == reqwest::StatusCode::FORBIDDEN
                {
                    info!(
                        "[Download] Session expired (HTTP {}) for bookId={}, page={}",
                        status, book_id, page
                    );
                    let _ = app.emit(
                        SESSION_EXPIRED_EVENT,
                        serde_json::json!({ "bookId": book_id, "status": status.as_u16() }),
                    );
                    break;
                }

                if !status.is_success() {
                    info!(
                        "[Download] Server returned {} for bookId={}, page={}",
                        status, book_id, page
                    );
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                match resp.json::<ApiResponse<Vec<PageItem>>>().await {
                    Ok(api_resp) => {
                        // 业务层会话失效（code=100）：通知前端重新登录并终止下载
                        if api_resp.code == 100 {
                            info!(
                                "[Download] Session expired (API code=100) for bookId={}, page={}",
                                book_id, page
                            );
                            let _ = app.emit(
                                SESSION_EXPIRED_EVENT,
                                serde_json::json!({ "bookId": book_id, "apiCode": api_resp.code }),
                            );
                            break;
                        }

                        if api_resp.code == 0 {
                            if let Some(items) = api_resp.data {
                                for item in &items {
                                    let now = chrono::Utc::now().timestamp();
                                    let db_state = app.state::<DbState>();
                                    let guard = db_state.0.lock().map_err(|e| format!("{e}"));
                                    if let Ok(guard) = guard {
                                        if let Some(ref conn) = *guard {
                                            if let Err(e) = conn.execute(
                                                "INSERT OR REPLACE INTO book_page (book_id, page_idx, content, title, top_chapter, status, create_time)
                                                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                                                rusqlite::params![
                                                    book_id,
                                                    item.page,
                                                    item.content,
                                                    item.title,
                                                    item.top_chapter,
                                                    now,
                                                ],
                                            ) {
                                                error!("[Download] DB write failed: bookId={}, page={}, err={}", book_id, item.page, e);
                                            }
                                        }
                                    }

                                    let payload = DownloadProgressPayload {
                                        book_id,
                                        downloaded_pages: item.page,
                                        total_page,
                                    };
                                    let _ = app.emit("book_download_progress", &payload);

                                    page = item.page + 1;

                                    let pct = (page * 100) / total_page;
                                    if pct >= last_progress_pct + 10 {
                                        last_progress_pct = pct;
                                        info!(
                                            "[Download] Progress: bookId={}, {}/{} ({}%)",
                                            book_id, page - 1, total_page, pct
                                        );
                                    }
                                }
                            } else {
                                page += 1;
                            }
                        } else {
                            info!(
                                "[Download] API error for bookId={}, page={}: code={}, msg={:?}",
                                book_id, page, api_resp.code, api_resp.message
                            );
                            page += 1;
                        }
                    }
                    Err(e) => {
                        info!(
                            "[Download] JSON parse error for bookId={}, page={}: {}",
                            book_id, page, e
                        );
                        page += 1;
                    }
                }
            }
            Err(e) => {
                info!(
                    "[Download] HTTP error for bookId={}, page={}: {}",
                    book_id, page, e
                );
                sleep(Duration::from_secs(1)).await;
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    info!("[Download] Download completed: bookId={}", book_id);
    Ok(())
}

#[tauri::command]
pub fn book_get_pending_pages(
    db: State<'_, DbState>,
    bookId: i64,
) -> Result<Vec<i64>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let mut stmt = conn
        .prepare("SELECT page_idx FROM book_page WHERE book_id = ?1 AND status = 0 ORDER BY page_idx")
        .map_err(|e| e.to_string())?;

    let pages = stmt
        .query_map(rusqlite::params![bookId], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(pages)
}

#[tauri::command]
pub fn book_save_downloaded_page(
    app: AppHandle,
    db: State<'_, DbState>,
    manager: State<'_, DownloadManager>,
    book_id: i64,
    page_idx: i64,
    total_page: i64,
    content: String,
    title: String,
    top_chapter: i64,
) -> Result<bool, String> {
    {
        let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
        if let Some(task) = tasks.get(&book_id) {
            if task.cancelled.load(Ordering::Relaxed) {
                return Ok(false);
            }
        }
    }

    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO book_page (book_id, page_idx, content, title, top_chapter, status, create_time)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        rusqlite::params![book_id, page_idx, content, title, top_chapter, now],
    )
    .map_err(|e| e.to_string())?;

    let payload = DownloadProgressPayload {
        book_id,
        downloaded_pages: page_idx,
        total_page,
    };
    let _ = app.emit("book_download_progress", &payload);

    info!(
        "[Download] Page saved: bookId={}, page={}/{}",
        book_id, page_idx, total_page
    );
    Ok(true)
}

#[tauri::command]
pub fn book_is_paused(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<bool, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        Ok(task.paused.load(Ordering::Relaxed))
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn book_is_cancelled(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<bool, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        Ok(task.cancelled.load(Ordering::Relaxed))
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn book_finish_download(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<bool, String> {
    let mut tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    tasks.remove(&bookId);
    info!("[Download] Download finished: bookId={}", bookId);
    Ok(true)
}

#[tauri::command]
pub fn book_pause_download(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<bool, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        task.paused.store(true, Ordering::Relaxed);
        info!("[Download] Download paused: bookId={}", bookId);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn book_resume_download(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<bool, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        task.paused.store(false, Ordering::Relaxed);
        info!("[Download] Download resumed: bookId={}", bookId);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn book_cancel_download(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<bool, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        task.cancelled.store(true, Ordering::Relaxed);
        info!("[Download] Download cancelled: bookId={}", bookId);
        Ok(true)
    } else {
        Ok(false)
    }
}
