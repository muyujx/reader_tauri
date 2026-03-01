import {DEV_MOD, CURRENT_HOST} from "../hostConfig.ts"
import router from "../route";
import { fetch } from '@tauri-apps/plugin-http';


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

async function afterRq<T>(fetchRq: Promise<any>): Promise<T> {

    let result: ResponseModel<T>;
    try {
        result = await fetchRq;
    } catch (e) {
        console.error("request error", e);
        return Promise.reject();
    }

    // 报错需要登录
    if (result.code == 100) {

        if (toLogin) {
            toLogin = false;

            await router.push({
                name: "Login"
            });

            setTimeout(() => {
                toLogin = true;
            }, 500);

        }

        return Promise.reject("need login!");
    }

    if (result.code != 0) {
        return Promise.reject(result);
    }
    return result.data;
}

function parseUrl(path: string): string {
    // 开发模式下去掉 /api 前缀，使用本地代理
    if (DEV_MOD && path.startsWith("/api")) {
        return CURRENT_HOST + path.substring("/api".length);
    }
    // 生产模式直接拼接完整路径
    return CURRENT_HOST + path;
}

export default {

    post<T>(rqParam: RequestParam): Promise<T> {
        const url = parseUrl(rqParam.url);
        console.debug(`[http] [post] url = ${url}`);
        
        return afterRq(fetch(url, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: rqParam.body ? JSON.stringify(rqParam.body) : undefined,
        }).then((res: any) => res.json()));
    },

    get<T>(rqParam: RequestParam): Promise<T> {
        const url = parseUrl(rqParam.url);
        console.debug(`[http] [get] url = ${url}`);
        
        const queryString = rqParam.queryParam 
            ? '?' + new URLSearchParams(rqParam.queryParam).toString() 
            : '';
            
        return afterRq(fetch(url + queryString, {
            method: 'GET',
        }).then((res: any) => res.json()));
    }
}

export function addHost(path: string): string {
    // 如果已经是完整URL，直接返回
    if (path.includes("://")) {
        return path;
    }

    // 确保路径以 / 开头
    if (!path.startsWith("/")) {
        path = "/" + path;
    }

    // 开发模式下去掉 /api 前缀
    if (DEV_MOD && path.startsWith("/api")) {
        return CURRENT_HOST + path.substring("/api".length);
    }
    
    // 生产模式
    return CURRENT_HOST + path;
}
