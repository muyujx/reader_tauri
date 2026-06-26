import rq from "./request.ts";
import {UserRole} from "../model/user.ts";

export interface LoginRes {
    userId: string,
    role: UserRole
    account: string
}

/**
 * 登录
 *
 * 登录请求经由 Rust 后端 rq_post 命令发出，响应中的 Set-Cookie
 * 会被后端 reqwest Jar 自动捕获并持久化到磁盘，后续所有请求
 * （前台 + 后台下载）都自动带 cookie，前端无需任何 cookie 处理。
 * @param account 账号
 * @param password 密码
 */
export async function loginApi(account: string, password: string): Promise<LoginRes> {
    const data = await rq.post<LoginRes>({
        url: `/api/login`,
        body: {
            account,
            password,
            clientType: 1
        }
    });

    if (data === undefined) {
        return Promise.reject("login failed");
    }
    return data;
}

/**
 * 检查当前是否登录状态
 */
export async function checkLoginApi(): Promise<LoginRes> {
    const data = await rq.post<LoginRes>({
        url: `/api/check_login`,
    });

    if (data === undefined) {
        return Promise.reject("check login failed");
    }
    return data;
}

/**
 * 登出
 */
export function logoutApi() {
    return rq.post({
        url: `/api/logout`,
    });
}