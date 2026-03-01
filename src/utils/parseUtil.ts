/**
 * 解析 int 值
 * 解析失败返回默认值
 * @param str string
 * @param defaultVal 默认值
 */
export function parseIntOrDefault(str: string, defaultVal: number): number {

    if (str == null) {
        return defaultVal;
    }

    const resVal = parseInt(str);
    return isNaN(resVal) ? defaultVal : resVal;
}

