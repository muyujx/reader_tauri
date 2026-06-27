use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{sleep, Duration};

use crate::db::{batch_insert_image_records, resolve_image_local_path, DbState, DownloadResult};
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
    pub total_images: i64,
    pub downloaded_images: i64,
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
    // 3) 将封面图片 URL 写入 book_image 表。
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

        // 封面图片 URL 写入 book_image
        let cover_urls = build_cover_urls(&coverPic, &bigCoverPic);
        if !cover_urls.is_empty() {
            batch_insert_image_records(conn, bookId, &cover_urls, &app)?;
            info!(
                "[Download] Cover images registered: bookId={}, count={}",
                bookId,
                cover_urls.len()
            );
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
                total_images: 0,
                downloaded_images: 0,
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
                                    // 先提取并下载页面中的图片，全部完成后再标记页面
                                    let image_urls = extract_image_urls(&item.content);
                                    if !image_urls.is_empty() {
                                        record_image_urls(&app, book_id, &image_urls);
                                    }

                                    // 下载本页关联的所有待下载图片
                                    let images_ok = download_page_images(
                                        &app, book_id, &server_host, &client,
                                        &paused, &cancelled,
                                    ).await;

                                    if images_ok {
                                        let save_ok = save_page_to_db(&app, book_id, item);
                                        if save_ok {
                                            done_in_task += 1;
                                            let downloaded_pages = downloaded_base + done_in_task;
                                            let (total_images, downloaded_images) = get_image_stats(&app, book_id);
                                            info!(
                                                "[Download] page_saved: bookId={}, page={}, downloadedPages={}/{}, images={}/{}",
                                                book_id, item.page, downloaded_pages, total_page,
                                                downloaded_images, total_images
                                            );
                                            match app.emit(
                                                "book_download_progress",
                                                &DownloadProgressPayload {
                                                    book_id,
                                                    downloaded_pages,
                                                    total_page,
                                                    total_images,
                                                    downloaded_images,
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
                                    } else {
                                        warn!(
                                            "[Download] page_images_failed: bookId={}, page={}",
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

    // 下载剩余可能未下载的图片（如封面等未关联到任何页面的图片）
    let remaining = download_remaining_images(
        &app, book_id, &server_host, &client, &paused, &cancelled,
    )
    .await;
    if remaining > 0 {
        info!(
            "[Download] Remaining images downloaded: bookId={}, count={}",
            book_id, remaining
        );
    }

    Ok(())
}

/// 下载本页关联的所有待下载图片（已在 book_image 中且 status=0 的）
/// 返回所有图片是否成功下载
async fn download_page_images(
    app: &AppHandle,
    book_id: i64,
    server_host: &str,
    client: &reqwest::Client,
    paused: &Arc<AtomicBool>,
    cancelled: &Arc<AtomicBool>,
) -> bool {
    let urls = get_pending_urls_from_db(app, book_id);
    if urls.is_empty() {
        return true;
    }

    let mut all_ok = true;
    for url in &urls {
        if cancelled.load(Ordering::Relaxed) {
            return false;
        }
        while paused.load(Ordering::Relaxed) {
            if cancelled.load(Ordering::Relaxed) {
                return false;
            }
            sleep(Duration::from_millis(500)).await;
        }

        if !download_image_file(app, book_id, url, server_host, client).await {
            all_ok = false;
        }
        sleep(Duration::from_millis(50)).await;
    }
    all_ok
}

/// 下载所有尚未下载的图片（封面等），返回下载数量
async fn download_remaining_images(
    app: &AppHandle,
    book_id: i64,
    server_host: &str,
    client: &reqwest::Client,
    paused: &Arc<AtomicBool>,
    cancelled: &Arc<AtomicBool>,
) -> i64 {
    let mut downloaded = 0i64;
    loop {
        let urls = get_pending_urls_from_db(app, book_id);
        if urls.is_empty() {
            break;
        }
        for url in &urls {
            if cancelled.load(Ordering::Relaxed) {
                return downloaded;
            }
            while paused.load(Ordering::Relaxed) {
                if cancelled.load(Ordering::Relaxed) {
                    return downloaded;
                }
                sleep(Duration::from_millis(500)).await;
            }
            if download_image_file(app, book_id, url, server_host, client).await {
                downloaded += 1;
            }
            sleep(Duration::from_millis(50)).await;
        }
    }
    downloaded
}

/// 从 book_image 表读取待下载图片 URL 列表
fn get_pending_urls_from_db(app: &AppHandle, book_id: i64) -> Vec<String> {
    let db_state = app.state::<DbState>();
    let guard = match db_state.0.lock() {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        "SELECT image_url FROM book_image WHERE book_id = ?1 AND status = 0 ORDER BY id",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let urls = match stmt.query_map(rusqlite::params![book_id], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect::<Vec<String>>(),
        Err(_) => Vec::new(),
    };
    urls
}

/// 从 book_image 表获取图片总数和已下载数
fn get_image_stats(app: &AppHandle, book_id: i64) -> (i64, i64) {
    let db_state = app.state::<DbState>();
    let guard = match db_state.0.lock() {
        Ok(g) => g,
        Err(_) => return (0, 0),
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => return (0, 0),
    };

    let total = conn
        .query_row(
            "SELECT COUNT(*) FROM book_image WHERE book_id = ?1",
            rusqlite::params![book_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let done = conn
        .query_row(
            "SELECT COUNT(*) FROM book_image WHERE book_id = ?1 AND status = 1",
            rusqlite::params![book_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    (total, done)
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
        total_images: 0,
        downloaded_images: 0,
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

// ================= 图片下载相关函数 =================

/// 构建封面图片 URL 列表（过滤空字符串）
fn build_cover_urls(cover_pic: &str, big_cover_pic: &str) -> Vec<String> {
    let mut urls = Vec::new();
    if !cover_pic.is_empty() {
        urls.push(cover_pic.to_string());
    }
    if !big_cover_pic.is_empty() && big_cover_pic != cover_pic {
        urls.push(big_cover_pic.to_string());
    }
    urls
}

/// 从 HTML 内容中提取所有 <img> 标签的 src 属性
fn extract_image_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    // 查找所有 src="..." 或 src='...' 模式
    let mut remaining = html;
    while let Some(start) = remaining.find("src=\"") {
        let after = &remaining[start + 5..];
        if let Some(end) = after.find('"') {
            let url = &after[..end];
            if !url.is_empty() && url.starts_with('/') {
                urls.push(url.to_string());
            }
            remaining = &after[end + 1..];
        } else {
            break;
        }
    }
    remaining = html;
    while let Some(start) = remaining.find("src='") {
        let after = &remaining[start + 5..];
        if let Some(end) = after.find('\'') {
            let url = &after[..end];
            if !url.is_empty() && url.starts_with('/') {
                urls.push(url.to_string());
            }
            remaining = &after[end + 1..];
        } else {
            break;
        }
    }
    info!("[Download] extract_image_urls: found {} image URLs: {:?}", urls.len(), urls);
    urls
}

/// 将图片 URL 写入 book_image 表（INSERT OR IGNORE 去重）
fn record_image_urls(app: &AppHandle, book_id: i64, urls: &[String]) {
    let db_state = app.state::<DbState>();
    let guard = match db_state.0.lock() {
        Ok(g) => g,
        Err(e) => {
            error!(
                "[Download] Image record DB lock failed: bookId={}, err={}",
                book_id, e
            );
            return;
        }
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => {
            error!(
                "[Download] Image record DB not initialized: bookId={}",
                book_id
            );
            return;
        }
    };

    if let Err(e) = batch_insert_image_records(conn, book_id, urls, app) {
        error!(
            "[Download] Image record insert failed: bookId={}, err={}",
            book_id, e
        );
    }
}

/// 下载单张图片到本地文件系统
async fn download_image_file(
    app: &AppHandle,
    book_id: i64,
    image_url: &str,
    server_host: &str,
    client: &reqwest::Client,
) -> bool {
    // 构建下载 URL（完整远程地址）
    let download_url = if image_url.starts_with("http") {
        image_url.to_string()
    } else {
        format!("{}{}", server_host.trim_end_matches('/'), image_url)
    };

    info!(
        "[Download] Downloading image: bookId={}, url={}",
        book_id, download_url
    );

    match client.get(&download_url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                warn!(
                    "[Download] Image HTTP {} for bookId={}, url={}",
                    resp.status(),
                    book_id,
                    download_url
                );
                return false;
            }

            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    warn!(
                        "[Download] Image read failed: bookId={}, url={}, err={}",
                        book_id, download_url, e
                    );
                    return false;
                }
            };

            // 解析本地存储路径
            let local_path = match resolve_image_local_path(app, image_url) {
                Ok(p) => p,
                Err(e) => {
                    error!(
                        "[Download] Image path resolve failed: bookId={}, url={}, err={}",
                        book_id, download_url, e
                    );
                    return false;
                }
            };

            // 确保父目录存在
            if let Some(parent) = local_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    error!(
                        "[Download] Image dir create failed: {:?}, err={}",
                        parent, e
                    );
                    return false;
                }
            }

            // 写入文件
            if let Err(e) = std::fs::write(&local_path, &bytes) {
                error!(
                    "[Download] Image write failed: {:?}, err={}",
                    local_path, e
                );
                return false;
            }

            // 更新数据库状态
            mark_image_done(app, book_id, image_url, &local_path);
            true
        }
        Err(e) => {
            warn!(
                "[Download] Image HTTP error: bookId={}, url={}, err={}",
                book_id, download_url, e
            );
            false
        }
    }
}

/// 标记图片下载完成
fn mark_image_done(app: &AppHandle, book_id: i64, image_url: &str, local_path: &std::path::Path) {
    let db_state = app.state::<DbState>();
    let guard = match db_state.0.lock() {
        Ok(g) => g,
        Err(e) => {
            error!("[Download] mark_image_done DB lock failed: {}", e);
            return;
        }
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => return,
    };

    let now = chrono::Utc::now().timestamp();
    let local_path_str = local_path.to_string_lossy().to_string();
    info!(
        "[Download] mark_image_done: bookId={}, image_url='{}', local_path='{}'",
        book_id, image_url, local_path_str
    );
    if let Err(e) = conn.execute(
        "UPDATE book_image SET status = 1, local_path = ?1, create_time = ?2
         WHERE book_id = ?3 AND image_url = ?4",
        rusqlite::params![
            local_path_str,
            now,
            book_id,
            image_url
        ],
    ) {
        error!(
            "[Download] mark_image_done DB update failed: bookId={}, url={}, err={}",
            book_id, image_url, e
        );
    }
}
