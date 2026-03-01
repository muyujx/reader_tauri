import {searchOnTypeApi} from "../../apis/book.ts";


export interface SearchOnTypeItem {
    value: string,
    label: string,
    id?: string,
    name?: string,
}


export function searchOnType(query: string, cb: (arg: any) => void) {
    if (query.length == 0) {
        cb([]);
        return;
    }

    searchOnTypeApi(query).then(res => {

        let list = new Array<SearchOnTypeItem>();
        for (let item of res) {
            list.push({
                value: item.name,
                label: item.highlight,
                id: item.id,
                name: item.name
            });
        }

        cb(list);
    })
}