/**
 * Tauri IPC 工具
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import log from "./log";

/**
 * 向 Tauri 后端发送命令（无返回值）
 */
export function ipcSend(channel: string): void {
    log.info('[ipcSend]', channel);
    invoke(channel).catch(err => log.error('[ipcSend error]', channel, err));
}

/**
 * 监听 Tauri 后端的事件
 */
export function ipcOn(channel: string, listener: (...args: any[]) => void): void {
    log.debug('[ipcOn] registering listener for:', channel);
    listen(channel, (event) => {
        log.debug('[ipcOn] event:', channel, event.payload);
        listener(event.payload);
    }).then(() => {
        log.debug('[ipcOn] listener registered for:', channel);
    }).catch(err => {
        log.error('[ipcOn] register failed:', channel, err);
    });
}

/**
 * 调用 Tauri 后端的方法并获取返回值
 */
export function ipcInvoke(channel: string, ...args: any[]): Promise<any> {
    // dev 下打印每一次 IPC 调用，便于在浏览器 console 定位"点了下载没反应"类问题
    log.info('[ipcInvoke]', channel, args[0] ?? {});
    return invoke(channel, args[0] || {}).catch(err => {
        log.error('[ipcInvoke error]', channel, err);
        return Promise.reject(err);
    });
}