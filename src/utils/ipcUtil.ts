/**
 * Tauri IPC 工具
 */
import { invoke } from "@tauri-apps/api/core";

/**
 * 向 Tauri 后端发送命令（无返回值）
 */
export function ipcSend(channel: string): void {
    console.log('[ipcSend]', channel);
    invoke(channel).catch(err => console.error('[ipcSend error]', err));
}

/**
 * 监听 Tauri 后端的事件
 */
export function ipcOn(channel: string, listener: (...args: any[]) => void): void {
    // Tauri v2 使用事件监听
    // @ts-ignore
    const unlisten = window.__TAURI__.event.listen(channel, (event) => {
        listener(event.payload);
    });
    
    // 返回取消监听函数
    return unlisten;
}

/**
 * 调用 Tauri 后端的方法并获取返回值
 */
export function ipcInvoke(channel: string, ...args: any[]): Promise<any> {
    return invoke(channel, args[0] || {});
}
