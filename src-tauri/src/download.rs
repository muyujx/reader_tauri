use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{sleep, Duration};

use crate::db::{DbState, DownloadResult};
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

/// 恢复下载的结果
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeResult {
    pub success: bool,
    pub book_id: i64,
    pub resumed: bool,
}

/// 通用简单结果（暂停/取消等操作）
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleResult {
    pub success: bool,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentsItemApi {
    level: i64,
    start_page: i64,
    label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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
) -> Result<DownloadResult, String> {
    info!(
        "[Download] book_download called: bookId={}, totalPage={}",
        bookId, totalPage
    );

    // 1) 更新/插入书籍元数据，并为所有页补建 status=0 的占位行（OR IGNORE
    //    保证已有下载完成的页 status=1 不会被覆盖），实现"已下载页跳过"。
    // 2) 一次性查出尚未下载的页（pending），后台任务只遍历这些页，
    //    避免对已下载页重复发起 HTTP 请求。
    let pending_pages: Vec<i64> = {
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

        // 拉 status=0 的页（待下载），按页号升序
        let mut stmt = conn
            .prepare("SELECT page_idx FROM book_page WHERE book_id = ?1 AND status = 0 ORDER BY page_idx")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![bookId], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;
        let mut pages = Vec::new();
        for r in rows {
            if let Ok(p) = r {
                pages.push(p);
            }
        }
        pages
    };

    info!(
        "[Download] Pending pages for bookId={}: {} (total={})",
        bookId,
        pending_pages.len(),
        totalPage,
    );

    // 已无待下载页：直接视为完成，不创建任务
    if pending_pages.is_empty() {
        info!(
            "[Download] All pages already downloaded: bookId={}, totalPage={}",
            bookId, totalPage
        );
        info!(
            "[Download] emit(all_downloaded): bookId={}, downloadedPages={}, totalPage={}",
            bookId, totalPage, totalPage
        );
        match app.emit(
            "book_download_progress",
            &DownloadProgressPayload {
                book_id: bookId,
                downloaded_pages: totalPage,
                total_page: totalPage,
            },
        ) {
            Ok(_) => {
                info!("[Download] emit_ok: bookId={}, all_pages_already_downloaded", bookId);
            }
            Err(e) => error!("[Download] emit_ERR: bookId={}, err={}", bookId, e),
        }
        return Ok(DownloadResult {
            success: false,
            book_id: bookId,
        });
    }

    {
        let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
        if tasks.contains_key(&bookId) {
            info!("[Download] Book already downloading: bookId={}", bookId);
            return Ok(DownloadResult {
                success: false,
                book_id: bookId,
            });
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
        "[Download] Download task registered: bookId={}, pending={}, using global cookie client",
        bookId,
        pending_pages.len(),
    );

    let host = serverHost.trim_end_matches('/').to_string();
    // 已下载完成的页数（任务起始基线），用于 emit 进度时给出累计 downloaded_pages
    let downloaded_base = totalPage - pending_pages.len() as i64;

    tokio::spawn(async move {
        if let Err(e) = background_download(
            app.clone(),
            bookId,
            totalPage,
            host,
            client,
            paused,
            cancelled,
            pending_pages,
            downloaded_base,
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

    Ok(DownloadResult {
        success: true,
        book_id: bookId,
    })
}

async fn background_download(
    app: AppHandle,
    book_id: i64,
    total_page: i64,
    server_host: String,
    client: reqwest::Client,
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    pending_pages: Vec<i64>,
    downloaded_base: i64,
) -> Result<(), String> {
    info!(
        "[Download] Background task start: bookId={}, serverHost={}, totalPages={}, pending={}, alreadyDownloaded={}",
        book_id, server_host, total_page, pending_pages.len(), downloaded_base,
    );

    // 下载目录
    let contents_url = format!("{}/api/book/info/get/contents", server_host);
    match client.get(&contents_url).query(&[("bookId", book_id)]).send().await {
        Ok(resp) => {
            if let Ok(api_resp) = resp.json::<ApiResponse<Vec<ContentsItemApi>>>().await {
                if api_resp.code == 0 {
                    if let Some(items) = api_resp.data {
                        save_contents_to_db(&app, book_id, &items);
                        info!("[Download] Contents saved: bookId={}, count={}", book_id, items.len());
                    }
                } else {
                    warn!("[Download] Contents API error: bookId={}, code={}", book_id, api_resp.code);
                }
            }
        }
        Err(e) => warn!("[Download] Contents HTTP error: bookId={}, err={}", book_id, e),
    }

    let page_url = format!("{}/api/book/page/html/page", server_host);
    let mut last_progress_pct = 0i64;
    // 本任务已成功下完的页数（不含 downloaded_base），用于推算 emit 的累计 downloaded_pages
    let mut done_in_task = 0i64;
    let pending_len = pending_pages.len();

    for page in pending_pages {
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

        info!(
            "[Download] http_request: bookId={}, page={}, url={}",
            book_id, page, page_url
        );
        match client.post(&page_url).json(&req_body).send().await {
            Ok(resp) => {
                let status = resp.status();
                info!(
                    "[Download] http_response: bookId={}, page={}, status={}",
                    book_id, page, status
                );

                // HTTP 层会话失效（401/403）：通知前端重新登录并终止下载
                if status == reqwest::StatusCode::UNAUTHORIZED
                    || status == reqwest::StatusCode::FORBIDDEN
                {
                    info!(
                        "[Download] Session expired (HTTP {}) for bookId={}, page={}",
                        status, book_id, page
                    );
                    match app.emit(
                        SESSION_EXPIRED_EVENT,
                        serde_json::json!({ "bookId": book_id, "status": status.as_u16() }),
                    ) {
                        Ok(_) => {}
                        Err(e) => error!("[Download] emit session_expired failed: bookId={}, err={}", book_id, e),
                    }
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

                // 先读取原始响应文本，便于排查 API 返回格式问题
                let resp_text = match resp.text().await {
                    Ok(t) => t,
                    Err(e) => {
                        info!("[Download] resp_read_ERR: bookId={}, page={}, err={}", book_id, page, e);
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };
                info!(
                    "[Download] resp_body: bookId={}, page={}, body={}",
                    book_id, page, resp_text.chars().take(500).collect::<String>()
                );

                match serde_json::from_str::<ApiResponse<Vec<PageItem>>>(&resp_text) {
                    Ok(api_resp) => {
                        info!(
                            "[Download] api_parse_ok: bookId={}, page={}, code={}",
                            book_id, page, api_resp.code
                        );
                        // 业务层会话失效（code=100）：通知前端重新登录并终止下载
                        if api_resp.code == 100 {
                            info!(
                                "[Download] Session expired (API code=100) for bookId={}, page={}",
                                book_id, page
                            );
                            match app.emit(
                                SESSION_EXPIRED_EVENT,
                                serde_json::json!({ "bookId": book_id, "apiCode": api_resp.code }),
                            ) {
                                Ok(_) => {}
                                Err(e) => error!("[Download] emit session_expired failed: bookId={}, err={}", book_id, e),
                            }
                            break;
                        }

                        if api_resp.code == 0 {
                            if let Some(items) = api_resp.data {
                                if items.is_empty() {
                                    warn!(
                                        "[Download] api_empty_data: bookId={}, page={}, code=0 but no items",
                                        book_id, page
                                    );
                                }
                                for item in &items {
                                    //落库标记 status=1；INSERT OR REPLACE 覆盖之前可能存在的 status=0 占位行
                                    let save_ok = save_page_to_db(&app, book_id, item);
                                    if save_ok {
                                        done_in_task += 1;
                                        let downloaded_pages = downloaded_base + done_in_task;
                                        info!(
                                            "[Download] page_saved: bookId={}, page={}, downloadedPages={}/{}, base={}, doneInTask={}",
                                            book_id, item.page, downloaded_pages, total_page, downloaded_base, done_in_task
                                        );
                                        match app.emit(
                                            "book_download_progress",
                                            &DownloadProgressPayload {
                                                book_id,
                                                downloaded_pages,
                                                total_page,
                                            },
                                        ) {
                                            Ok(_) => {
                                                info!(
                                                    "[Download] emit_ok: bookId={}, downloadedPages={}, totalPage={}",
                                                    book_id, downloaded_pages, total_page
                                                );
                                            }
                                            Err(e) => error!("[Download] emit_ERR: bookId={}, err={}", book_id, e),
                                        }

                                        let pct = if total_page > 0 {
                                            (downloaded_pages * 100) / total_page
                                        } else {
                                            0
                                        };
                                        if pct >= last_progress_pct + 10 {
                                            last_progress_pct = pct;
                                            info!(
                                                "[Download] Progress: bookId={}, {}/{} ({}%)",
                                                book_id, downloaded_pages, total_page, pct
                                            );
                                        }
                                    } else {
                                        warn!(
                                            "[Download] save_page_FAILED: bookId={}, page={}",
                                            book_id, item.page
                                        );
                                    }
                                }
                            } else {
                                warn!(
                                    "[Download] api_data_none: bookId={}, page={}, code=0 but data is None",
                                    book_id, page
                                );
                            }
                        } else {
                            info!(
                                "[Download] API error for bookId={}, page={}: code={}, msg={:?}",
                                book_id, page, api_resp.code, api_resp.message
                            );
                        }
                    }
                    Err(e) => {
                        info!(
                            "[Download] JSON parse error for bookId={}, page={}: {}",
                            book_id, page, e
                        );
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

    let final_downloaded = downloaded_base + done_in_task;
    info!(
        "[Download] Download loop done: bookId={}, total={}, pending={}, finalDownloaded={}/{}, doneInTask={}",
        book_id, total_page, pending_len, final_downloaded, total_page, done_in_task
    );
    Ok(())
}

/// 把目录写入数据库
fn save_contents_to_db(app: &AppHandle, book_id: i64, items: &[ContentsItemApi]) {
    let db_state = app.state::<DbState>();
    let guard = match db_state.0.lock() {
        Ok(g) => g,
        Err(e) => {
            error!("[Download] Contents DB lock failed: bookId={}, err={}", book_id, e);
            return;
        }
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => {
            error!("[Download] Contents DB not initialized: bookId={}", book_id);
            return;
        }
    };

    for item in items {
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO book_contents (book_id, level, start_page, label)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![book_id, item.level, item.start_page, item.label],
        ) {
            error!(
                "[Download] Contents save failed: bookId={}, startPage={}, err={}",
                book_id, item.start_page, e
            );
        }
    }
}

/// 把单页内容写入数据库（status=1），返回是否写入成功。
/// 抽出来便于在循环里复用并集中处理 DB 错误日志。
fn save_page_to_db(app: &AppHandle, book_id: i64, item: &PageItem) -> bool {
    let now = chrono::Utc::now().timestamp();
    let db_state = app.state::<DbState>();
    let guard = db_state.0.lock();
    match guard {
        Ok(guard) => {
            if let Some(ref conn) = *guard {
                match conn.execute(
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
                    Ok(_) => true,
                    Err(e) => {
                        error!(
                            "[Download] DB write failed: bookId={}, page={}, err={}",
                            book_id, item.page, e
                        );
                        false
                    }
                }
            } else {
                error!("[Download] DB not initialized when saving page: bookId={}", book_id);
                false
            }
        }
        Err(e) => {
            error!("[Download] DB lock failed: bookId={}, err={}", book_id, e);
            false
        }
    }
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
) -> Result<DownloadResult, String> {
    {
        let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
        if let Some(task) = tasks.get(&book_id) {
            if task.cancelled.load(Ordering::Relaxed) {
                return Ok(DownloadResult {
                    success: false,
                    book_id,
                });
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

    // 从数据库查询实际已下载完成的页数，避免 page_idx 误导进度
    let downloaded_pages: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM book_page WHERE book_id = ?1 AND status = 1",
            rusqlite::params![book_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let payload = DownloadProgressPayload {
        book_id,
        downloaded_pages,
        total_page,
    };
    info!(
        "[Download] emit(save_downloaded_page): bookId={}, downloadedPages={}, totalPage={}, pageIdx={}",
        book_id, downloaded_pages, total_page, page_idx
    );
    match app.emit("book_download_progress", &payload) {
        Ok(_) => {
            info!("[Download] emit_ok: bookId={}, save_downloaded_page", book_id);
        }
        Err(e) => error!("[Download] emit_ERR: bookId={}, err={}", book_id, e),
    }

    info!(
        "[Download] Page saved: bookId={}, page={}/{}, downloaded={}",
        book_id, page_idx, total_page, downloaded_pages
    );
    Ok(DownloadResult {
        success: true,
        book_id,
    })
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
) -> Result<SimpleResult, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        task.paused.store(true, Ordering::Relaxed);
        info!("[Download] Download paused: bookId={}", bookId);
        Ok(SimpleResult { success: true })
    } else {
        Ok(SimpleResult { success: false })
    }
}

#[tauri::command]
pub fn book_resume_download(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<ResumeResult, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        task.paused.store(false, Ordering::Relaxed);
        info!("[Download] Download resumed: bookId={}", bookId);
        Ok(ResumeResult {
            success: true,
            book_id: bookId,
            resumed: true,
        })
    } else {
        Ok(ResumeResult {
            success: false,
            book_id: bookId,
            resumed: false,
        })
    }
}

#[tauri::command]
pub fn book_cancel_download(
    manager: State<'_, DownloadManager>,
    bookId: i64,
) -> Result<SimpleResult, String> {
    let tasks = manager.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&bookId) {
        task.cancelled.store(true, Ordering::Relaxed);
        info!("[Download] Download cancelled: bookId={}", bookId);
        Ok(SimpleResult { success: true })
    } else {
        Ok(SimpleResult { success: false })
    }
}
