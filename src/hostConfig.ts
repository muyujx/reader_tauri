import config from './config.json';

// 生产构建时强制使用生产服务器
const isProduction = import.meta.env.PROD;

// 生产环境直接使用生产服务器地址，硬编码不依赖 config.json
const PROD_HOST = 'http://43.136.218.87';

// 开发模式: 不是生产构建且 config.dev_mod 为 true
const devMod = !isProduction && config.dev_mod;

export const DEV_MOD = devMod;
export const SERVER_PROD_HOST = PROD_HOST;
export const SERVER_DEV_HOST = config.server_dev_host;

// 生产构建时强制使用生产服务器
export const CURRENT_HOST = isProduction ? PROD_HOST : (devMod ? SERVER_DEV_HOST : SERVER_PROD_HOST);

// API 路径 - 获取书籍页面
export const API_BOOK_PAGE_LIST = '/api/book/page/html/page';
