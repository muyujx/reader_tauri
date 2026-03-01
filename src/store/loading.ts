import {defineStore} from "pinia";


let loadingCount = 0

export const loadingStore = defineStore('loading', {
    state: () => ({isLoading: false}),

    actions: {
        show() {
            loadingCount++
            this.isLoading = true
        },
        hide() {
            loadingCount = Math.max(loadingCount - 1, 0)
            if (loadingCount === 0) {

                setTimeout(() => {
                    if (loadingCount == 0) {
                        this.isLoading = false
                    }
                }, 200);

            }
        }
    }
})
