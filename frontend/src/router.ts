import { createRouter, createWebHistory } from 'vue-router'
import { auth } from './auth'
import LoginView from './views/LoginView.vue'
import CallbackView from './views/CallbackView.vue'
import HomeView from './views/HomeView.vue'
import UsersView from './views/UsersView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', component: LoginView, meta: { public: true } },
    { path: '/auth/callback', component: CallbackView, meta: { public: true } },
    { path: '/', component: HomeView },
    { path: '/admin/users', component: UsersView, meta: { admin: true } },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})

router.beforeEach(async (to) => {
  if (!auth.loaded.value) await auth.load()
  if (!to.meta.public && !auth.user.value) return '/login'
  if (to.meta.admin && !auth.isAdmin.value) return '/'
  if (to.meta.public && auth.user.value && to.path === '/login') return '/'
})

export default router
