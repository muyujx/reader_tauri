export function secondsToTimeStr(seconds: number): string {

    const resSeconds = seconds % 60;
    seconds = Math.floor(seconds / 60);
    const minutes = seconds % 60;
    seconds = Math.floor(seconds / 60);
    const hours = seconds;
    let res = "";
    if (hours > 0) {
        res += hours + "时";
    }
    if (minutes > 0) {
        res += minutes + "分";
    }
    if (resSeconds > 0) {
        res += resSeconds + "秒";
    }

    return res;
}
