<template>

    <Transition name="fade">

        <div class="loading_container"
             v-show="loading.isLoading"
        >
            <div class="loading">
                <div></div>
                <div></div>
                <div></div>
                <div></div>
            </div>
        </div>

    </Transition>

</template>

<script setup lang="ts">

import {loadingStore} from "../store/loading.ts";

const loading = loadingStore();

</script>

<style scoped lang="less">

.loading_container {
    width: 100%;
    height: 100%;
    position: absolute;
    top: 0;
    left: 0;
    z-index: 200;

    background-color: rgba(0, 0, 0, 0.4);

    box-sizing: border-box;
    padding-bottom: 20vh;
    justify-content: center;
    align-items: center;
    display: flex;
}


.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

.fade-enter-to,
.fade-leave-from {
    opacity: 1;
}

.loading,
.loading > div {
    position: relative;
    box-sizing: border-box;
}

.loading {
    display: block;
    font-size: 0;
    color: var(--accent);
}

.loading > div {
    display: inline-block;
    float: none;
    background-color: currentColor;
    border: 0 solid currentColor;
}

.loading {
    width: 42px;
    height: 32px;
}

.loading > div:nth-child(1) {
    position: absolute;
    bottom: 32%;
    left: 18%;
    width: 14px;
    height: 14px;
    border-radius: 100%;
    transform-origin: center bottom;
    animation: ball-climbing-dot-jump 0.6s ease-in-out infinite;
}

.loading > div:not(:nth-child(1)) {
    position: absolute;
    top: 0;
    right: 0;
    width: 14px;
    height: 2px;
    border-radius: 0;
    transform: translate(60%, 0);
    animation: ball-climbing-dot-steps 1.8s linear infinite;
}

.loading > div:not(:nth-child(1)):nth-child(2) {
    animation-delay: 0ms;
}

.loading > div:not(:nth-child(1)):nth-child(3) {
    animation-delay: -600ms;
}

.loading > div:not(:nth-child(1)):nth-child(4) {
    animation-delay: -1200ms;
}

@keyframes ball-climbing-dot-jump {
    0% {
        transform: scale(1, 0.7);
    }

    20% {
        transform: scale(0.7, 1.2);
    }

    40% {
        transform: scale(1, 1);
    }

    50% {
        bottom: 125%;
    }

    46% {
        transform: scale(1, 1);
    }

    80% {
        transform: scale(0.7, 1.2);
    }

    90% {
        transform: scale(0.7, 1.2);
    }

    100% {
        transform: scale(1, 0.7);
    }
}

@keyframes ball-climbing-dot-steps {
    0% {
        top: 0;
        right: 0;
        opacity: 0;
    }

    50% {
        opacity: 1;
    }

    100% {
        top: 100%;
        right: 100%;
        opacity: 0;
    }
}

</style>