// src-tauri/src/lib.rs
use log::{error, info};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewWindowBuilder};

mod config;
mod db;
mod download;
mod http;

use config::{
    get_cache_dir as config_get_cache_dir, get_win_size as config_get_win_size, save_cache_dir,
    set_win_size as config_set_win_size,
};
use db::{
    book_delete, book_get_all_list, book_get_download_progress, book_get_info,
    book_get_list_by_page, book_get_local_contents, book_get_local_image, book_get_page,
    book_is_downloaded, book_save_page, book_update_read_progress, init_db, set_cache_dir, DbState,
};
use download::{
    book_cancel_download, book_download, book_finish_download, book_get_pending_pages,
    book_is_cancelled, book_is_paused, book_pause_download, book_resume_download,
    book_save_downloaded_page, DownloadManager,
};
use http::{rq_get, rq_post, HttpState};

// ================= 窗口控制命令 =================

#[cfg(desktop)]
#[tauri::command]
fn window_minimize(window: tauri::WebviewWindow) {
    if let Err(e) = window.minimize() {
        error!("[Window] minimize error: {}", e);
    }
}

#[cfg(not(desktop))]
#[tauri::command]
fn window_minimize(_window: tauri::WebviewWindow) {}

#[cfg(desktop)]
#[tauri::command]
fn window_maximize(window: tauri::WebviewWindow) {
    match window.is_maximized() {
        Ok(true) => {
            if let Err(e) = window.unmaximize() {
                error!("[Window] unmaximize error: {}", e);
            }
        }
        Ok(false) => {
            if let Err(e) = window.maximize() {
                error!("[Window] maximize error: {}", e);
            }
        }
        Err(e) => error!("[Window] is_maximized error: {}", e),
    }
}

#[cfg(not(desktop))]
#[tauri::command]
fn window_maximize(_window: tauri::WebviewWindow) {}

#[cfg(desktop)]
#[tauri::command]
fn window_close(window: tauri::WebviewWindow) {
    info!("window_close called");
    if let Err(e) = window.close() {
        error!("[Window] close error: {}", e);
    }
}

#[cfg(not(desktop))]
#[tauri::command]
fn window_close(_window: tauri::WebviewWindow) {}

#[cfg(desktop)]
#[tauri::command]
fn is_maximized(window: tauri::WebviewWindow) -> bool {
    window.is_maximized().unwrap_or(false)
}

#[cfg(not(desktop))]
#[tauri::command]
fn is_maximized(_window: tauri::WebviewWindow) -> bool {
    false
}

// ================= 配置相关命令 =================

#[tauri::command]
fn get_start_win_size(app: AppHandle) -> Vec<i32> {
    config_get_win_size(app)
}

#[tauri::command]
fn set_start_win_size(app: AppHandle, width: i32, height: i32) -> Vec<i32> {
    config_set_win_size(app, width, height);
    vec![width, height]
}

#[tauri::command]
fn get_root_cache_dir(app: AppHandle) -> String {
    config_get_cache_dir(app)
}

#[tauri::command]
fn change_root_cache_dir(app: AppHandle, dir: String) -> Result<bool, String> {
    save_cache_dir(&app, &dir)?;
    info!("[Config] Cache dir changed to: {}", dir);
    Ok(true)
}

// ================= 主入口逻辑 =================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 日志策略（前后端分工）：
    //   - 后端 (Rust)：统一走 tauri-plugin-log。
    //       debug 构建：Stdout + 日志文件，IDE console 实时可见，事后也能翻文件。
    //       release 构建：只写日志文件（{app_log_dir}/Reader.log），避免弹控制台窗口。
    //     级别：dev=Info，release=Info（release 保留 Info 以便用户侧问题事后排查）。
    //   - 前端 (TS)：始终输出浏览器 console，不写文件；dev 全量、release 只剩 warn/error
    //     （见 src/utils/log.ts）。排查前端问题用 webview devtools。
    //
    // 注：dev 下能看到 IDE console 的前提是程序走 console 子系统，见 main.rs
    //     的 windows_subsystem 策略（release 才切到 windows GUI 子系统）。
    let log_targets = if cfg!(debug_assertions) {
        vec![
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: None }),
        ]
    } else {
        vec![tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::LogDir { file_name: None },
        )]
    };

    let log_level = log::LevelFilter::Info;

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets(log_targets)
                .level(log_level)
                .build(),
        )
        .setup(|app| {
            info!("[App] Starting application setup...");

            #[cfg(desktop)]
            {
                info!("[App] Detected Desktop environment.");

                // 1. 获取用户上次保存的窗口大小
                let win_size = config_get_win_size(app.handle().clone());
                info!("[App] Restoring window size: {}x{}", win_size[0], win_size[1]);

                // 2. 克隆默认配置并修改大小
                let mut window_config = app.config().app.windows[0].clone();
                window_config.width = win_size[0] as f64;
                window_config.height = win_size[1] as f64;

                // 确保标签名存在，默认为 "main"
                if window_config.label.is_empty() {
                    window_config.label = "main".to_string();
                }

                info!(
                    "[App] Creating window with config: {}x{} (Label: {})",
                    window_config.width, window_config.height, window_config.label
                );

                // 3. 手动构建窗口
                // 因为 main.rs 已经过滤了构建脚本，这里不会在构建时执行，所以安全
                let _window = WebviewWindowBuilder::from_config(app.handle(), &window_config)?
                    .build()?;

                info!("[App] Window created successfully.");
            }

            #[cfg(mobile)]
            {
                info!("[App] Detected Mobile environment. Skipping manual window creation (handled by OS).");
                // 如果 tauri.conf.json 中 create: false 且移动端也需要手动创建，
                // 可以在这里添加类似的 WebviewWindowBuilder 代码，但不需要恢复桌面坐标。
            }

            // ================= 通用逻辑：初始化数据库 =================
            info!("[App] Initializing database...");
            let conn = init_db(app.handle()).expect("Failed to initialize database");
            app.manage(DbState(Mutex::new(Some(conn))));
            info!("[App] Database initialized.");

            // 初始化下载管理器
            app.manage(DownloadManager::new());
            info!("[App] Download manager initialized.");

            // 初始化统一网络层（带 cookie 持久化的全局 reqwest Client）
            let http_state = HttpState::new(app.handle()).expect("Failed to init HttpState");
            app.manage(http_state);
            info!("[App] Http state initialized.");

            info!("[App] Setup completed successfully.");
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            window_minimize,
            window_maximize,
            window_close,
            is_maximized,
            // 统一网络请求
            rq_post,
            rq_get,
            book_download,
            book_get_download_progress,
            book_get_pending_pages,
            book_save_downloaded_page,
            book_finish_download,
            book_is_paused,
            book_is_cancelled,
            book_pause_download,
            book_resume_download,
            book_cancel_download,
            book_get_page,
            book_get_info,
            book_is_downloaded,
            book_delete,
            book_get_all_list,
            book_get_list_by_page,
            book_update_read_progress,
            book_save_page,
            set_cache_dir,
            book_get_local_image,
            book_get_local_contents,
            get_start_win_size,
            set_start_win_size,
            get_root_cache_dir,
            change_root_cache_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}