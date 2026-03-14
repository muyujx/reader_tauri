// src-tauri/src/main.rs

fn main() {
    // 获取命令行参数
    let args: Vec<String> = std::env::args().collect();

    // 如果检测到该参数，说明是构建过程，不应启动 UI，直接返回。
    if args.iter().any(|arg| arg.contains("android-studio-script")) {
        // 静默退出，避免创建窗口导致构建脚本等待超时
        return;
    }

    reader_lib::run();
}