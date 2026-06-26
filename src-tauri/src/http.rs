//! 统一的 Rust 网络层：全局带 cookie 的 reqwest::Client + cookie 持久化。
//!
//! 设计目标：
//! - 前端所有请求统一通过 `rq_post` / `rq_get` 这两个 Tauri 命令发起，
//!   不再使用 `@tauri-apps/plugin-http`。
//! - 后端维护一个带 `reqwest::cookie::Jar` 的全局 `reqwest::Client`，
//!   登录响应里的 Set-Cookie 会被 reqwest 自动捕获进 jar，后续所有
//!   请求（包括后台下载）都自动带上 cookie，无需前端转发。
//! - cookie 同时持久化到 `app_data_dir/cookies.json`，应用重启后
//!   自动恢复登录态，真正实现"cookie 一直都在，不依赖登录流程"。
//!
//! 持久化方式：每次响应里出现 Set-Cookie 时，把原始 Set-Cookie 字符串
//! 连同其 URL 保存到 json 文件；启动时读取该文件，对每条调用
//! `Jar::add_cookie_str` 注入。这样无需引入额外的 cookie_store crate。

use log::{error, info};
use reqwest::cookie::Jar;
// reqwest 重导出 url::Url，避免把 url 作为直接依赖
use reqwest::Url;
use reqwest::header::{HeaderMap, HeaderValue, SET_COOKIE};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// 全局网络状态：带 cookie 的 reqwest::Client + 其 cookie jar。
///
/// `client` 与 `jar` 在应用启动时构造一次，整个生命周期共享。
/// 后台下载任务通过 `get_client` 取同一个 client，自动带登录态。
pub struct HttpState {
    pub client: reqwest::Client,
    pub jar: Arc<Jar>,
    cookie_file: PathBuf,
}

/// 持久化到磁盘的单条 cookie 记录
#[derive(Serialize, Deserialize, Clone)]
struct CookieRecord {
    /// 与 cookie 关联的完整 URL（含 scheme+host），用于 add_cookie_str
    url: String,
    /// 原始 Set-Cookie 字符串
    raw: String,
}

impl HttpState {
    /// 在 setup 阶段构造：加载持久化 cookie -> 构造 client。
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

        let cookie_file = app_dir.join("cookies.json");
        let jar = Arc::new(Jar::default());

        // 加载已持久化的 cookie 到 jar
        if cookie_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&cookie_file) {
                if let Ok(records) = serde_json::from_str::<Vec<CookieRecord>>(&content) {
                    for rec in &records {
                        if let Ok(url) = Url::parse(&rec.url) {
                            jar.add_cookie_str(&rec.raw, &url);
                        }
                    }
                    info!("[Http] Loaded {} persisted cookie(s)", records.len());
                }
            }
        }

        // 构造带 cookie jar 的全局 client
        let client = reqwest::Client::builder()
            .cookie_provider(jar.clone())
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        info!("[Http] HttpState initialized, cookie_file={:?}", cookie_file);
        Ok(Self {
            client,
            jar,
            cookie_file,
        })
    }

    /// 把响应里的 Set-Cookie 持久化到磁盘（便于下次启动恢复登录态）
    fn persist_cookies(&self, url: &str, headers: &HeaderMap) {
        let set_cookies: Vec<&HeaderValue> = headers.get_all(SET_COOKIE).iter().collect();
        if set_cookies.is_empty() {
            return;
        }
        // 记录本次响应里出现的 Set-Cookie 条数（for 循环会消费迭代器，先保存）
        let new_count = set_cookies.len();

        // 读取已有记录，合并新 cookie 后整体回写
        let mut records: Vec<CookieRecord> = if self.cookie_file.exists() {
            std::fs::read_to_string(&self.cookie_file)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        for v in set_cookies {
            if let Ok(raw) = v.to_str() {
                records.push(CookieRecord {
                    url: url.to_string(),
                    raw: raw.to_string(),
                });
            }
        }

        if let Ok(json) = serde_json::to_string_pretty(&records) {
            if let Err(e) = std::fs::write(&self.cookie_file, json) {
                error!("[Http] Failed to persist cookies: {}", e);
            } else {
                info!("[Http] Persisted {} cookie(s)", new_count);
            }
        }
    }
}

/// 供下载等后台任务取全局 client（自动带 cookie）
pub fn get_http_client(state: &HttpState) -> reqwest::Client {
    state.client.clone()
}

// ============ Tauri 命令：通用网络请求 ============

/// 统一的响应结构：status + body 文本。前端再自行 JSON.parse。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetResponse {
    pub status: u16,
    pub body: String,
    #[serde(skip)]
    _phantom: (),
}

/// POST 请求
#[tauri::command]
pub async fn rq_post(
    state: State<'_, HttpState>,
    url: String,
    body: Option<String>,
) -> Result<NetResponse, String> {
    info!("[Http] rq_post url={}", url);

    let mut req = state.client.post(&url);
    if let Some(b) = body {
        req = req.header("Content-Type", "application/json").body(b);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status().as_u16();
    // 在消费 body 前抓取 Set-Cookie 持久化
    state.persist_cookies(&url, resp.headers());
    let text = resp
        .text()
        .await
        .map_err(|e| format!("read body failed: {}", e))?;

    Ok(NetResponse {
        status,
        body: text,
        _phantom: (),
    })
}

/// GET 请求
#[tauri::command]
pub async fn rq_get(state: State<'_, HttpState>, url: String) -> Result<NetResponse, String> {
    info!("[Http] rq_get url={}", url);

    let resp = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status().as_u16();
    state.persist_cookies(&url, resp.headers());
    let text = resp
        .text()
        .await
        .map_err(|e| format!("read body failed: {}", e))?;

    Ok(NetResponse {
        status,
        body: text,
        _phantom: (),
    })
}