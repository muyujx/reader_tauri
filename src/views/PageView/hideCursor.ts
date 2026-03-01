import {onUnmounted} from "vue";

/**
 * 当鼠标闲置超过指定时间后隐藏鼠标
 *
 * @param qStr document.querySelector 参数
 * @param delaySecond 闲置多少秒后隐藏鼠标
 */
export function hideCurSorIfIdle(qStr: string, delaySecond: number): void {

    let timeout: number = 0;
    const delay = delaySecond * 1000;
    let lastCursor: string = "";
    let hideState: boolean = false;

    function resetTimeout() {
        if (timeout) {
            clearTimeout(timeout);
        }

        // 恢复鼠标
        restoreCursor();

        // @ts-ignore
        timeout = setTimeout(() => {
            // 隐藏鼠标
            hideCursor();
        }, delay);
    }

    function hideCursor() {
        let doc = document.querySelector(qStr) as HTMLElement;
        if (doc == null) {
            return;
        }

        lastCursor = doc.style.cursor;
        doc.style.cursor = "none";
        hideState = true;
    }

    function restoreCursor() {
        if (!hideState) {
            return;
        }

        let doc = document.querySelector(qStr) as HTMLElement;
        if (doc == null) {
            return;
        }

        doc.style.cursor = lastCursor;
        lastCursor = "";
        hideState = false;
    }

    window.addEventListener('mousemove', resetTimeout);

    onUnmounted(() => {
        window.removeEventListener('mousemove', resetTimeout);
    });

    resetTimeout();

}


