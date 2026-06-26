import {DEV_MOD, CURRENT_HOST} from "../hostConfig"
import router from "../route";
import { ipcInvoke } from "../utils/ipcUtil.ts";

export class OfflineError extends Error {
    constructor() {
        super('offline');
        this.name = 'OfflineError';
    }
}

export interface ResponseModel<T> {
    data: T,
    code: number,
    message: string
}

interface RequestParam {
    url: string,
    body?: any
    queryParam?: any
}


let toLogin = true;

async function afterRq<T>(result: ResponseModel<T>): Promise<T> {
    // 报错需要登录
    if (result.code == 100) {
        if (toLogin) {
            toLogin = false;
            await router.push({ name: "Login" });
            setTimeout(() => { toLogin = true; }, 500);
        }
        return Promise.reject("need login!");
    }

    if (result.code != 0) {
        return Promise.reject(result);
    }
    return result.data;
}

function parseUrl(path: string): string {
    return CURRENT_HOST + path;
}

export default {

    /**
     * POST 请求：统一走 Rust 后端 rq_post 命令。
     *
     * 所有请求复用后端全局带 cookie 的 reqwest Client，
     * 登录态由后端 Jar 维护并持久化到磁盘，前端无需关心 cookie。
     */
    async post<T>(rqParam: RequestParam): Promise<T> {
        if (import.meta.env.VITE_DEBUG_OFFLINE === 'true') {
            return Promise.reject(new OfflineError());
        }

        const url = parseUrl(rqParam.url);
        const res = await ipcInvoke('rq_post', {
            url,
            body: rqParam.body ? JSON.stringify(rqParam.body) : undefined,
        }) as { status: number, body: string };

        if (res.status < 200 || res.status >= 300) {
            return Promise.reject(`HTTP ${res.status}`);
        }

        const result = JSON.parse(res.body) as ResponseModel<T>;
        return afterRq(result);
    },

    /**
     * GET 请求：统一走 Rust 后端 rq_get 命令。
     */
    async get<T>(rqParam: RequestParam): Promise<T> {
        if (import.meta.env.VITE_DEBUG_OFFLINE === 'true') {
            return Promise.reject(new OfflineError());
        }

        const url = parseUrl(rqParam.url);
        const queryString = rqParam.queryParam
            ? '?' + new URLSearchParams(rqParam.queryParam).toString()
            : '';

        const res = await ipcInvoke('rq_get', {
            url: url + queryString,
        }) as { status: number, body: string };

        if (res.status < 200 || res.status >= 300) {
            return Promise.reject(`HTTP ${res.status}`);
        }

        const result = JSON.parse(res.body) as ResponseModel<T>;
        return afterRq(result);
    },

    }

export function addHost(path: string): string {
    if (path.includes("://")) {
        return path;
    }
    if (!path.startsWith("/")) {
        path = "/" + path;
    }
    if (DEV_MOD && path.startsWith("/api")) {
        return CURRENT_HOST + path.substring("/api".length);
    }
    return CURRENT_HOST + path;
}