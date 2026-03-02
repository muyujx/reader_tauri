use log::info;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewWindowBuilder};

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
fn window_minimize(window: tauri::Window) {
    if let Err(e) = window.minimize() {
        eprintln!("minimize error: {}", e);
    }
}

#[tauri::command]
fn window_maximize(window: tauri::Window) {
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
fn window_close(window: tauri::Window) {
    info!("window_close called");
    if let Err(e) = window.close() {
        eprintln!("close error: {}", e);
    }
}

#[tauri::command]
fn is_maximized(window: tauri::Window) -> bool {
    window.is_maximized().unwrap_or(false)
}

// 配置相关命令
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .setup(|app| {
            info!("[App] Starting application...");

            // 获取保存的窗口大小
            let win_size = config_get_win_size(app.handle().clone());
            info!("[App] Loaded window size: {}x{}", win_size[0], win_size[1]);

            // 从配置创建窗口（可以动态修改配置）
            let mut window_config = app.config().app.windows[0].clone();
            window_config.width = win_size[0] as f64;
            window_config.height = win_size[1] as f64;

            info!(
                "[App] Creating window with size: {}x{}",
                window_config.width, window_config.height
            );

            // 使用 from_config 手动创建窗口
            let _window =
                WebviewWindowBuilder::from_config(app.handle(), &window_config)?.build()?;

            info!("[App] Window created successfully");

            // 初始化数据库
            let conn = init_db(app.handle()).expect("Failed to initialize database");
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
            get_start_win_size,
            set_start_win_size,
            get_root_cache_dir,
            change_root_cache_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
