/**
 * 平台判断工具
 * 通过 navigator.userAgent 判断当前运行平台
 */

/** 平台枚举 */
export enum Platform {
    /** Windows 桌面端 */
    Windows = 'Windows',
    /** macOS 桌面端 */
    MacOS = 'MacOS',
    /** Linux 桌面端 */
    Linux = 'Linux',
    /** Android 移动端 */
    Android = 'Android',
    /** iOS 移动端 */
    iOS = 'iOS',
    /** 未知平台 */
    Unknown = 'Unknown',
}

const ua: string = navigator.userAgent || '';

/** 当前平台 */
export const currentPlatform: Platform = detectPlatform();

/** 是否为桌面端（Windows / macOS / Linux） */
export function isPc(): boolean {
    return (
        currentPlatform === Platform.Windows ||
        currentPlatform === Platform.MacOS ||
        currentPlatform === Platform.Linux
    );
}

/** 是否为移动端（Android / iOS） */
export function isMobile(): boolean {
    return (
        currentPlatform === Platform.Android ||
        currentPlatform === Platform.iOS
    );
}

/** 检测当前平台 */
function detectPlatform(): Platform {
    if (ua.includes('Android')) {
        return Platform.Android;
    }
    if (/iPhone|iPad|iPod/.test(ua)) {
        return Platform.iOS;
    }
    if (ua.includes('Windows')) {
        return Platform.Windows;
    }
    if (ua.includes('Macintosh') || ua.includes('Mac OS')) {
        return Platform.MacOS;
    }
    if (ua.includes('Linux')) {
        return Platform.Linux;
    }
    return Platform.Unknown;
}
