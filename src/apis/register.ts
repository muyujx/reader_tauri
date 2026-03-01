import rq from "./request.ts";
import {LoginRes} from "./login.ts";

/**
 * 注册
 * @param account 账号
 * @param password 密码
 */
export function registerApi(account: string, password: string): Promise<LoginRes> {
    return rq.post({
        url: `/api/register`,
        body: {
            account,
            password,
        }
    });
}
