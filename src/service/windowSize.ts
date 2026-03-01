type WindowSizeCallback = (width: number, height: number) => void;

class WindowSizeListener {

    private width: number;
    private height: number;
    private listeners = new Set<WindowSizeCallback>();

    private resizeTimeoutId: ReturnType<typeof setTimeout> | null = null;
    // 当窗口大小停止变化 200ms 后进行通知
    private readonly DEBOUNCE_DELAY = 200;

    constructor() {
        this.width = window.innerWidth;
        this.height = window.innerHeight;
        window.addEventListener('resize', this.handleResize);
    }

    private handleResize = () => {
        if (this.resizeTimeoutId !== null) {
            clearTimeout(this.resizeTimeoutId);
        }
        const newWidth = window.innerWidth;
        const newHeight = window.innerHeight;
        if (this.width !== newWidth || this.height !== newHeight) {
            this.resizeTimeoutId = setTimeout(() => {
                this.width = newWidth;
                this.height = newHeight;
                this.triggerListener();
                this.resizeTimeoutId = null;
            }, this.DEBOUNCE_DELAY);
        }
    }

    public on(callback: WindowSizeCallback) {
        if (callback == null) {
            return;
        }
        this.listeners.add(callback);
        callback(this.width, this.height);
    }

    public delete (callback: WindowSizeCallback) {
        if (callback == null) {
            return;
        }
        this.listeners.delete(callback);
    }

    private triggerListener () {
        for (let callback of this.listeners) {
            callback(this.width, this.height);
        }
    }

}

const windowSizeListener = new WindowSizeListener();
export default windowSizeListener;






