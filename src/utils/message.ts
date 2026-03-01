import {ElMessage} from "element-plus";


export function popInfo(msg: string) {
    ElMessage.info(msg);
}

export function popSuccess(msg: string) {
    ElMessage.success(msg);
}

export function popWarn(msg: string) {
    ElMessage.warning(msg);
}

export function popErr(msg: string) {
    ElMessage.error(msg);
}

