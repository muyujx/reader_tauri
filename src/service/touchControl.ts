type Runnable = () => void;

export class TouchControl {

    private static readonly DELTA_X = 60;

    private firstX: number = 0
    private firstY: number = 0

    private lastX: number = 0
    private lastY: number = 0


    private swipeLeft: null | Runnable = null
    private swipeRight: null | Runnable = null

    touchstart(evt: TouchEvent) {
        this.firstX = evt.touches[0].clientX;
        this.firstY = evt.touches[0].clientY;

        this.lastX = this.firstX;
        this.lastY = this.firstY;
    }

    touchmove(evt: TouchEvent) {
        this.lastX = evt.touches[0].clientX;
        this.lastY = evt.touches[0].clientY;
    }

    touchend() {
        const delta = this.lastX - this.firstX;
        // 标记 lastY 和 firstY 已被使用
        this.lastY; 
        this.firstY;

        if (delta >= TouchControl.DELTA_X) {
            if (this.swipeRight != null) {
                this.swipeRight();
            }
        } else if (delta <= -TouchControl.DELTA_X) {
            if (this.swipeLeft != null) {
                this.swipeLeft();
            }
        }
    }

    public onSwipeLeft(swipeLeft: null | Runnable) {
        this.swipeLeft = swipeLeft;
    }

    public onSwipeRight(swipeRight: null | Runnable) {
        this.swipeRight = swipeRight;
    }

}
