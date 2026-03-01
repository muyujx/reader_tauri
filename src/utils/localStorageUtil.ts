const localStorage = window.localStorage;

export function setLocalStorage(key: any, val: any) {
    localStorage.setItem(key.toString(), val.toString());
}

export function getLocalStorage(key: string, defaultValue: string): string {
    const val = localStorage.getItem(key);
    return val == null ? defaultValue : val;
}


/**
 * 获取对应的 localStorage 并且转换成 int
 *
 * @param key localStorage key
 * @param defaultValue 如果获取不成功返回默认值
 */
export function getLocalStorageInt(key: any, defaultValue: number): number {

    const val = localStorage.getItem(key.toString());

    if (val == null) {
        return defaultValue;
    }

    const intVal = parseInt(val);
    return isNaN(intVal) ? defaultValue : intVal;
}

/**
 * 获取对应的 localStorage 并且转换成 float
 *
 * @param key localStorage key
 * @param defaultValue 如果获取不成功返回默认值
 */
export function getLocalStorageNumber(key: any, defaultValue: number): number {
    const val = localStorage.getItem(key.toString());
    if (val == null) {
        return defaultValue;
    }
    return Number.parseFloat(val);
}

/**
 * 获取对应的 localStorage 并且转换成 boolean
 *
 * @param key localStorage key
 * @param defaultValue 如果获取不成功返回默认值
 */
export function getLocalStorageBoolean(key: string, defaultValue: boolean): boolean {

    const val = localStorage.getItem(key);
    if (val == null) {
        return defaultValue;
    }

    return val == 'true';
}
