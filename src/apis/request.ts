import {DEV_MOD, CURRENT_HOST} from "../hostConfig"
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

    post<T>(rqParam: RequestParam): Promise<T> {
        const url = parseUrl(rqParam.url);
        
        return fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: rqParam.body ? JSON.stringify(rqParam.body) : undefined,
        }).then((res: any) => res.json())
          .then((result: ResponseModel<T>) => afterRq(result));
    },

    get<T>(rqParam: RequestParam): Promise<T> {
        const url = parseUrl(rqParam.url);
        const queryString = rqParam.queryParam 
            ? '?' + new URLSearchParams(rqParam.queryParam).toString() 
            : '';
            
        return fetch(url + queryString, {
            method: 'GET',
        }).then((res: any) => res.json())
          .then((result: ResponseModel<T>) => afterRq(result));
    }
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
