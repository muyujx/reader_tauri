import rq from "./request.ts";
import {UserRole} from "../model/user.ts";

export interface LoginRes {
    userId: string,
    role: UserRole
    account: string
}


/**
 * 登录
 * @param account 账号
 * @param password 密码
 */
export function loginApi(account: string, password: string): Promise<LoginRes> {
    return rq.post({
        url: `/api/login`,
        body: {
            account,
            password,
            clientType: 1
        }
    });
}

/**
 * 检查当前是否登录状态
 */
export function checkLoginApi(): Promise<LoginRes> {
    return rq.post({
        url: `/api/check_login`,
    });
}

/**
 * 登出
 */
export function logoutApi() {
    return rq.post({
        url: `/api/logout`,
    });
}