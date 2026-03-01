import {createRouter, createWebHashHistory} from 'vue-router';


const routes = [
    {
        path: "/",
        redirect: {
            name: "Home"
        }
    }, {
        path: "/read",
        name: "Read",
        component: () => import('../views/PageView/PageView.vue'),
        meta: {
            auth: true
        }
    }, {
        path: "/login",
        component: () => import('../views/Login/Login.vue'),
        name: "Login"
    }, {
        path: "/home",
        component: () => import('../views/Home/Home.vue'),
        name: "Home"
    }
];

export default createRouter({
    history: createWebHashHistory(),
    routes
});

