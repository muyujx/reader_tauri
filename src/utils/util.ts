/**
 * 二分查找
 * @param arr 排好序的数组
 * @param target 目标值
 * @param com 比较方法
 */
export function binarySearch<T, S>(arr: T[], target: S, com: (a: T, b: S) => number): number {

    let l = 0, r = arr.length - 1;

    let res: number = -1;

    while (l <= r) {
        let mid = l + Math.floor((r - l) / 2);
        if (com(arr[mid], target) <= 0) {
            res = mid;
            l = mid + 1;
        } else {
            r = mid - 1;
        }
    }


    return res;
}


/**
 * 二分查找第一个大于的项
 * @param arr 排好序的数组
 * @param target 目标值
 * @param com 比较方法
 */
export function binarySearchCeil<T, S>(arr: T[], target: S, com: (a: T, b: S) => number): T | null {
    let idx = binarySearch(arr, target, com);

    if (idx == -1) {
        return null;
    }

    if (com(arr[idx], target) == 0) {
        return arr[idx];
    }

    if (idx + 1 == arr.length) {
        return null;
    }

    return arr[idx + 1];
}


