import {onBeforeUnmount} from "vue";

type ScreeResize = (fullScreen: boolean, width: number, height: number) => void;


export class ScreenResizeListener {

    private readonly functions = new Array<ScreeResize>();

    private readonly listener;

    constructor() {

        this.listener = (_evt?: Event) => {
            let full = false
            if (window.screen.height == document.body.clientHeight &&
                window.screen.width == document.body.clientWidth) {
                full = true;
            }
            this.resize(full, document.body.clientWidth, document.body.clientHeight);
        }

        window.addEventListener("resize", this.listener);

        onBeforeUnmount(() => {
            this.remove();
        });
    }

    private resize(fullScreen: boolean, width: number, height: number) {
        for (let item of this.functions) {
            item(fullScreen, width, height);
        }
    }


    private remove() {
        window.removeEventListener("resize", this.listener);
    }

    public addListener(func: ScreeResize) {
        this.functions.push(func);
        this.listener();
    }

}