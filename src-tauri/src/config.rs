use log::info;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

// 默认窗口大小（仅首次启动使用）
const DEFAULT_WIN_WIDTH: i32 = 1500;
const DEFAULT_WIN_HEIGHT: i32 = 1000;

/// 获取配置文件路径
fn get_config_path(app: &AppHandle) -> Option<PathBuf> {
    let config_dir = app.path().app_config_dir().ok()?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).ok()?;
    }

    let path = config_dir.join("config.json");
    info!("[Config] Config path: {:?}", path);
    Some(path)
}

/// 读取配置
fn load_config_value(app: &AppHandle, key: &str) -> Option<String> {
    let config_path = get_config_path(app)?;

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                return json
                    .get(key)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

/// 保存配置
fn save_config_value(app: &AppHandle, key: &str, value: &str) -> Result<(), String> {
    let config_path = get_config_path(app).ok_or("Failed to get config path")?;

    let mut json = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str::<serde_json::Value>(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    json[key] = serde_json::json!(value);

    let content = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, content).map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取窗口大小
#[tauri::command]
pub fn get_win_size(app: AppHandle) -> Vec<i32> {
    let width = load_config_value(&app, "winWidth")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WIN_WIDTH);
    let height = load_config_value(&app, "winHeight")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WIN_HEIGHT);

    info!("[Config] get_win_size returning: {}x{}", width, height);
    vec![width, height]
}

/// 设置窗口大小
#[tauri::command]
pub fn set_win_size(app: AppHandle, width: i32, height: i32) {
    let _ = save_config_value(&app, "winWidth", &width.to_string());
    let _ = save_config_value(&app, "winHeight", &height.to_string());
    info!("[Config] set_win_size saved: {}x{}", width, height);
}

/// 获取默认缓存目录
fn get_default_cache_dir(app: &AppHandle) -> String {
    app.path()
        .app_data_dir()
        .map(|p| p.join("reader").join("cache").to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("reader")
                .join("cache")
                .to_string_lossy()
                .to_string()
        })
}

#[tauri::command]
pub fn get_cache_dir(app: AppHandle) -> String {
    if let Some(dir) = load_config_value(&app, "cacheDir") {
        return dir;
    }
    get_default_cache_dir(&app)
}

pub fn save_cache_dir(app: &AppHandle, dir: &str) -> Result<(), String> {
    save_config_value(app, "cacheDir", dir)
}
