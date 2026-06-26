// src-tauri/src/main.rs
//
// windows_subsystem 策略：
//   - dev 构建（cargo tauri dev / IDE 调试）：使用默认的 console 子系统，
//     标准输出会被附加到启动它的终端（IDE 控制台 / cargo 进程），因此
//     tauri-plugin-log 的 Stdout target 能在 IDE console 实时看到日志。
//   - release 构建：使用 windows GUI 子系统，避免弹出黑色控制台窗口，
//     此时所有日志只写日志文件（见 lib.rs 的日志策略）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
