<template>

    <div class="login-page">

        <div class="login-form"
             v-show="showLogin">
            <div>
                <p>账号</p>
                <el-input type="text"
                          v-model="account"
                          maxlength="30"
                          minlength="6"
                          @input="account = account.replace(accountPattern, '')"
                />
            </div>

            <div>
                <p>密码</p>
                <el-input type="password"
                          show-password
                          v-model="password"
                          maxlength="30"
                          minlength="8"
                          @input="password = password.replace(passwordPattern, '')"
                />
            </div>

            <div class="buttons">

                <div class="button" @click="login">登录</div>

            </div>

            <div class="toggle">
                <div class="button" @click="showLogin = false">注册账号</div>
            </div>

        </div>


        <div class="register-form"
             v-show="!showLogin">
            <div>
                <p>账号</p>
                <el-input type="text"
                          v-model="regAccount"
                          maxlength="30"
                          minlength="6"
                          @input="regAccount = regAccount.replace(accountPattern, '')"
                />
            </div>

            <div>
                <p>密码</p>
                <el-input type="password"
                          show-password
                          v-model="regPwd"
                          maxlength="30"
                          minlength="8"
                          @input="regPwd = regPwd.replace(passwordPattern, '')"
                />
            </div>

            <div>
                <p>确认密码</p>
                <el-input type="password"
                          show-password
                          v-model="regPwdConfirm"
                          maxlength="20"
                          minlength="5"
                          @input="regPwdConfirm = regPwdConfirm.replace(passwordPattern, '')"
                />
            </div>

            <div class="buttons">

                <div class="button" @click="register">注册</div>

            </div>

            <div class="toggle">
                <div class="button" @click="showLogin = true">返回登录</div>
            </div>

        </div>


    </div>


</template>

<script setup lang="ts">

import {ref} from "vue";
import {loginApi} from "../../apis/login.ts";
import {useRouter} from "vue-router";
import {setUserStore} from "../../store/userStore.ts";
import {popErr, popSuccess, popWarn} from "../../utils/message.ts";
import {registerApi} from "../../apis/register.ts";


const router = useRouter();
const showLogin = ref(true);

const account = ref('');
const password = ref('');

const regAccount = ref('');
const regPwd = ref('');
const regPwdConfirm = ref('');


const accountPattern = /[^a-zA-Z0-9_\-+=\\.]/g;
const passwordPattern = /[^a-zA-Z0-9_\-+=\\.]/g;


function login() {

    let accountVal = account.value;
    let passwordVal = password.value;

    if (checkAccountPassword(accountVal, passwordVal)) {
        return;
    }
    doLogin(accountVal, passwordVal);
}

function doLogin(account: string, password: string) {
    loginApi(account, password).then(res => {

        setUserStore(res);

        router.push({
                name: "Home"
            }
        ).then();
    }).catch(res => {
        popErr(res.msg);
    })
}

function checkAccountPassword(accountStr: string, pwdStr: string): boolean {
    if (accountStr.length == 0) {
        popWarn("账号不能为空");
        return true;
    }

    if (accountStr.length < 6) {
        popWarn("账号长度必须在 6 - 30");
        return true;
    }

    if (pwdStr.length == 0) {
        popWarn("密码不能为空");
        return true;
    }

    if (pwdStr.length < 8) {
        popWarn("密码长度必须在 8 - 30");
        return true;
    }

    return false;
}

function register() {

    let accountVal = regAccount.value;
    let passwordVal = regPwd.value;
    let passwordConfirm = regPwdConfirm.value;

    if (checkAccountPassword(accountVal, passwordVal)) {
        return;
    }

    if (passwordConfirm !== passwordVal) {
        popWarn("两次密码不一致");
        return;
    }

    registerApi(accountVal, passwordVal)
        .then(() => {
            popSuccess("注册成功");
            doLogin(accountVal, passwordVal);
        });
}


</script>

<style lang="less">

body {
    --title-bar-bgc: #263238;
    --title-bar-color: #B0BEC5;
}

</style>

<style scoped lang="less" src="./Login.less"/>