import {createApp, h, Ref, watch} from "vue";
import {ElIcon, ElPopover} from "element-plus";
import {InfoFilled} from "@element-plus/icons-vue";

export function addPopover(curPageItem: Ref) {

    watch(curPageItem, () => {
        const noteQuery = '#raw_page .note';

        const noteList = document.querySelectorAll(noteQuery);

        if (noteList.length != 0) {
            // @ts-ignore
            for (let noteNode of noteList) {
                // @ts-ignore
                const noteContent = noteNode.attributes.data.value;
                createApp({
                    render() {
                        return h(ElPopover, {
                            placement: "top",
                            trigger: "hover",
                            effect: "dark",
                            width: "400",
                            content: noteContent,
                            popperClass: "notePopover"
                        }, {
                            reference() {
                                return h(ElIcon,
                                    {
                                        style: 'width: 100%; height: 100%;'
                                    },
                                    {
                                        default() {
                                            return h(InfoFilled);
                                        }
                                    });
                            }
                        });
                    }
                }).mount(noteNode);
            }
        }
    }, {flush: "post"});
}
