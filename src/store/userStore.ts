import {defineStore} from "pinia";
import {UserRole} from "../model/user.ts";
import {LoginRes} from "../apis/login.ts";

export interface UserInfo {
    userId: string,
    role: UserRole,
    account: string,
}

export const userStore = defineStore('userInfo', (): UserInfo => {
    return {
        userId: '',
        role: UserRole.User,
        account: ''
    }
})

export function setUserStore(res: LoginRes) {
    let userInfo = userStore();
    userInfo.userId = res.userId;
    userInfo.role = res.role;
    userInfo.account = res.account
}