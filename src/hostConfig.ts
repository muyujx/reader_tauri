// 服务器地址 - 始终使用此地址
const SERVER_HOST = 'http://43.136.218.87';

// DEV_MODE: 是否连接本地开发服务器（仅 web 开发有效）
const DEV_MODE = import.meta.env.VITE_DEV_MODE === 'true';

// DEV_MOD: 供 addHost 函数使用，决定是否去除 /api 前缀
export const DEV_MOD = DEV_MODE;

export const SERVER_PROD_HOST = SERVER_HOST;

// CURRENT_HOST: 
// - DEV_MODE=true 时使用 localhost:5173（仅 web dev server）
// - 否则使用 SERVER_HOST（所有 Tauri 环境都用这个）
export const CURRENT_HOST = DEV_MODE ? 'http://localhost:5173' : SERVER_HOST;

// API 路径 - 获取书籍页面
export const API_BOOK_PAGE_LIST = '/api/book/page/html/page';