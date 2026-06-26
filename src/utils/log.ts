/**
 * 前端统一日志工具
 *
 * 策略：
 *  - 所有日志始终输出到浏览器 console（不在前端写文件），方便开发
 *    时使用 webview devtools 排查问题。
 *  - dev 构建（vite dev / tauri dev）：全部级别都打印。
 *  - release 构建（vite build / tauri build）：静默 debug/info，仅保留
 *    warn/error，避免线上发布版在用户浏览器里刷大量日志。
 *
 * 与 Rust 后端的日志策略配合：
 *  - 后端 dev：Stdout + 日志文件（IDE console / 终端可见，见 src-tauri/src/main.rs
 *    中 windows_subsystem 策略）。
 *  - 后端 release：只写应用日志目录的文件。
 * 二者分工清晰：前端排查看浏览器 console，后端排查看 IDE console（dev）
 * 或日志文件（release）。
 */

// Vite 注入的环境标志：PROD/DEV 在构建期确定，tree-shaking 友好
const isProd = import.meta.env.PROD === true;

// 模块级前缀，便于在浏览器 console 中快速过滤业务日志
const TAG = '[Reader]';

export const log = {
    debug(...args: any[]): void {
        if (isProd) return;
        console.debug(TAG, ...args);
    },
    info(...args: any[]): void {
        if (isProd) return;
        console.info(TAG, ...args);
    },
    warn(...args: any[]): void {
        console.warn(TAG, ...args);
    },
    error(...args: any[]): void {
        console.error(TAG, ...args);
    },
};

export default log;