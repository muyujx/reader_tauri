import config from './config.json';

// 使用配置文件中的 dev_mod 决定使用哪个地址
// dev_mod: true -> 使用 server_dev_host (本地开发服务器)
// dev_mod: false -> 使用 server_prod_host (生产服务器)

export const DEV_MOD = config.dev_mod;
export const SERVER_PROD_HOST = config.server_prod_host;
export const SERVER_DEV_HOST = config.server_dev_host;

// 根据 config 中的 dev_mod 选择 host
export const CURRENT_HOST = DEV_MOD ? SERVER_DEV_HOST : SERVER_PROD_HOST;

// API 路径 - 获取书籍页面
export const API_BOOK_PAGE_LIST = '/api/book/page/html/page';
