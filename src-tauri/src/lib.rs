use log::info;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Window};

mod config;
mod db;

use config::{
    get_cache_dir as config_get_cache_dir, get_win_size as config_get_win_size, save_cache_dir,
    set_win_size as config_set_win_size,
};
use db::{
    book_delete, book_download, book_get_all_list, book_get_download_progress, book_get_info,
    book_get_list_by_page, book_get_local_image, book_get_page, book_is_downloaded, book_save_page,
    book_update_read_progress, init_db, set_cache_dir, DbState,
};

#[tauri::command]
fn window_minimize(window: Window) {
    if let Err(e) = window.minimize() {
        eprintln!("minimize error: {}", e);
    }
}

#[tauri::command]
fn window_maximize(window: Window) {
    match window.is_maximized() {
        Ok(true) => {
            if let Err(e) = window.unmaximize() {
                eprintln!("unmaximize error: {}", e);
            }
        }
        Ok(false) => {
            if let Err(e) = window.maximize() {
                eprintln!("maximize error: {}", e);
            }
        }
        Err(e) => eprintln!("is_maximized error: {}", e),
    }
}

#[tauri::command]
fn window_close(window: Window) {
    info!("window_close called");
    if let Err(e) = window.close() {
        eprintln!("close error: {}", e);
    }
}

#[tauri::command]
fn is_maximized(window: Window) -> bool {
    window.is_maximized().unwrap_or(false)
}

// 配置相关命令
#[tauri::command]
fn get_start_win_size() -> Vec<i32> {
    config_get_win_size()
}

#[tauri::command]
fn set_start_win_size(width: i32, height: i32) -> Vec<i32> {
    config_set_win_size(width, height);
    vec![width, height]
}

#[tauri::command]
fn get_root_cache_dir(app: AppHandle) -> String {
    config_get_cache_dir(app)
}

#[tauri::command]
fn change_root_cache_dir(dir: String) -> Result<bool, String> {
    // 保存到配置文件
    save_cache_dir(&dir)?;
    // 返回成功
    info!("[Config] Cache dir changed to: {}", dir);
    Ok(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .setup(|app| {
            info!("[App] Starting application...");

            // 初始化数据库
            let app_handle = app.handle();
            let conn = init_db(app_handle).expect("Failed to initialize database");
            app.manage(DbState(Mutex::new(Some(conn))));

            info!("[App] Application started successfully");
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            window_minimize,
            window_maximize,
            window_close,
            is_maximized,
            book_download,
            book_get_download_progress,
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
            // 配置相关
            get_start_win_size,
            set_start_win_size,
            get_root_cache_dir,
            change_root_cache_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
